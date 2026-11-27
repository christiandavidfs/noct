//! `noct compat`: capa de compatibilidad con Umbriel y Noctalia.
//!
//! Objetivo: que una actualización de Umbriel o Noctalia no deje el escritorio
//! roto ni exija tocar archivos a mano. La capa es conservadora:
//!
//! - valida `~/.config/umbriel/config.toml` con el binario instalado
//!   (`umbriel config validate --json`);
//! - repara solo lo que el propio Umbriel marca como retirado: alias
//!   conocidos, claves desconocidas y atajos con acciones que ya no existen;
//! - nunca reescribe valores del usuario que siguen siendo válidos;
//! - guarda una copia del último estado válido y la restaura solo si el
//!   archivo deja de ser TOML válido;
//! - deja un informe en `~/.local/state/noct/compat.json` y un registro en
//!   `compat.log`.
//!
//! Además, `noct msg` actúa como puente de IPC: los atajos llaman a
//! `noct msg <verbo>` y, si Noctalia renombra el verbo, Noct resuelve el alias
//! o falla en silencio dejando el resto de atajos intactos.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use toml_edit::{DocumentMut, Item, Value};

use crate::env;

/// Diagnóstico devuelto por `umbriel config validate --json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    #[serde(default)]
    pub line: u64,
    #[serde(default)]
    pub column: u64,
    #[serde(default)]
    pub severity: String,
    pub message: String,
}

/// Resultado de una pasada de compatibilidad.
#[derive(Debug, Clone, Default, Serialize)]
pub struct Outcome {
    pub config: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_before: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_after: Option<bool>,
    pub repairs: Vec<String>,
    pub pending: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backup: Option<String>,
    pub restored: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub umbriel_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub noctalia_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub noct_binary: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
}

impl Outcome {
    /// `true` si el escritorio queda en un estado que Umbriel acepta.
    pub fn healthy(&self) -> bool {
        self.valid_after.unwrap_or(false) && self.pending.is_empty()
    }
}

pub fn config_path() -> PathBuf {
    env::app_config("umbriel").join("config.toml")
}

pub fn state_dir() -> PathBuf {
    env::app_state("noct")
}

fn snapshot_path() -> PathBuf {
    state_dir().join("umbriel-config.good.toml")
}

fn report_path() -> PathBuf {
    state_dir().join("compat.json")
}

fn log_path() -> PathBuf {
    state_dir().join("compat.log")
}

/// Alias de acciones de Umbriel retiradas o renombradas.
///
/// Solo se aplican cuando la acción original ya no existe y el destino sí.
/// Se mantiene corto a propósito: cada entrada debe ser equivalente funcional.
const KEYBIND_ALIASES: &[(&str, &str)] = &[];

/// Verbo de Noctalia retirado → verbo equivalente actual.
///
/// `noct msg` los usa como red de seguridad en tiempo de ejecución, de modo
/// que un atajo sigue funcionando aunque Noctalia renombre el comando.
const IPC_ALIASES: &[(&str, &[&str])] = &[
    ("launcher-toggle", &["panel-toggle", "launcher"]),
    ("launcher-open", &["panel-toggle", "launcher"]),
    ("control-center-toggle", &["panel-toggle", "control-center"]),
    ("control-center-open", &["panel-toggle", "control-center"]),
    ("wallpaper-toggle", &["panel-toggle", "wallpaper"]),
    ("wallpaper-picker", &["panel-toggle", "wallpaper"]),
    ("lock-screen", &["session", "lock"]),
    ("screen-lock", &["session", "lock"]),
    ("screenshot", &["screenshot-region"]),
];

/// Valida un `config.toml` con el Umbriel instalado.
///
/// Devuelve `None` si Umbriel no está disponible o la salida no es JSON.
pub fn validate(path: &Path) -> Option<(bool, Vec<Diagnostic>)> {
    let out = env::run(
        "umbriel",
        &[
            "config",
            "validate",
            "--json",
            "-c",
            &path.display().to_string(),
        ],
    )?;
    let raw = if out.stdout.trim_start().starts_with('{') {
        out.stdout.as_str()
    } else {
        out.stderr.as_str()
    };
    parse_validation(raw)
}

fn parse_validation(raw: &str) -> Option<(bool, Vec<Diagnostic>)> {
    let value: serde_json::Value = serde_json::from_str(raw).ok()?;
    let valid = value.get("valid")?.as_bool()?;
    let diagnostics = value
        .get("diagnostics")
        .and_then(|d| d.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| serde_json::from_value(item.clone()).ok())
                .collect()
        })
        .unwrap_or_default();
    Some((valid, diagnostics))
}

/// Qué hacer con un diagnóstico emitido por Umbriel.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Fix {
    /// Retira una clave por ruta (`unknown key a.b.c`).
    RemovePath(String),
    /// Retira un atajo cuya acción ya no existe.
    RemoveKeybind(String),
    /// No se puede reparar sin criterio humano.
    Unfixable(String),
}

fn plan(diag: &Diagnostic) -> Fix {
    let message = diag.message.trim();
    if let Some(path) = message.strip_prefix("unknown key ") {
        return Fix::RemovePath(path.trim().to_string());
    }
    if let Some(rest) = message.strip_prefix("ignoring keybind ") {
        if let Some(chord) = quoted(rest, '\'') {
            return Fix::RemoveKeybind(chord);
        }
    }
    if let Some(rest) = message.strip_prefix("ignoring ") {
        if let Some((path, _)) = rest.split_once(" (") {
            return Fix::RemovePath(path.trim().to_string());
        }
    }
    Fix::Unfixable(message.to_string())
}

fn quoted(text: &str, quote: char) -> Option<String> {
    let start = text.find(quote)?;
    let rest = &text[start + 1..];
    let end = rest.find(quote)?;
    Some(rest[..end].to_string())
}

/// Separa una ruta TOML en segmentos, respetando claves entre comillas.
fn split_path(path: &str) -> Vec<String> {
    let mut segments = Vec::new();
    let mut current = String::new();
    let mut quote: Option<char> = None;
    let mut push = |current: &mut String| {
        if !current.is_empty() {
            segments.push(current.clone());
            current.clear();
        }
    };
    for ch in path.chars() {
        match quote {
            Some(q) => {
                if ch == q {
                    quote = None;
                } else {
                    current.push(ch);
                }
            }
            None => match ch {
                '"' | '\'' => quote = Some(ch),
                '.' | '[' | ']' => push(&mut current),
                _ => current.push(ch),
            },
        }
    }
    push(&mut current);
    segments
}

fn remove_in_table(table: &mut toml_edit::Table, segments: &[String]) -> bool {
    let Some((head, tail)) = segments.split_first() else {
        return false;
    };
    if tail.is_empty() {
        return table.remove(head).is_some();
    }
    match table.get_mut(head) {
        Some(next) => remove_in_item(next, tail),
        None => false,
    }
}

fn remove_in_item(item: &mut Item, segments: &[String]) -> bool {
    match item {
        Item::Table(table) => remove_in_table(table, segments),
        Item::ArrayOfTables(array) => {
            let Some((head, tail)) = segments.split_first() else {
                return false;
            };
            let Ok(index) = head.parse::<usize>() else {
                return false;
            };
            match array.get_mut(index) {
                Some(table) => remove_in_table(table, tail),
                None => false,
            }
        }
        Item::Value(Value::InlineTable(inline)) => {
            let Some((head, tail)) = segments.split_first() else {
                return false;
            };
            // Las tablas inline anidadas no se tocan: se dejan como pendiente.
            tail.is_empty() && inline.remove(head).is_some()
        }
        _ => false,
    }
}

fn remove_path(doc: &mut DocumentMut, path: &str) -> bool {
    let segments = split_path(path);
    let Some((head, tail)) = segments.split_first() else {
        return false;
    };
    if tail.is_empty() {
        return doc.as_table_mut().remove(head).is_some();
    }
    match doc.as_table_mut().get_mut(head) {
        Some(item) => remove_in_item(item, tail),
        None => false,
    }
}

fn remove_keybind(doc: &mut DocumentMut, chord: &str) -> bool {
    let Some(table) = doc
        .as_table_mut()
        .get_mut("keybinds")
        .and_then(Item::as_table_mut)
    else {
        return false;
    };
    table.remove(chord).is_some()
}

/// Aplica alias de acciones cuando el original ya no existe.
fn apply_keybind_aliases(doc: &mut DocumentMut) -> Vec<String> {
    if KEYBIND_ALIASES.is_empty() {
        return Vec::new();
    }
    let mut applied = Vec::new();
    let Some(table) = doc
        .as_table_mut()
        .get_mut("keybinds")
        .and_then(Item::as_table_mut)
    else {
        return applied;
    };
    for (_, item) in table.iter_mut() {
        match item {
            Item::Value(value) => {
                let Some(text) = value.as_str().map(|s| s.to_string()) else {
                    continue;
                };
                if let Some((from, to)) = KEYBIND_ALIASES.iter().find(|(from, _)| *from == text) {
                    *value = Value::from(*to);
                    applied.push(format!("atajo {from} → {to}"));
                }
            }
            Item::Table(entry) => {
                if let Some(Item::Value(value)) = entry.get_mut("action") {
                    let Some(text) = value.as_str().map(|s| s.to_string()) else {
                        continue;
                    };
                    if let Some((from, to)) = KEYBIND_ALIASES.iter().find(|(from, _)| *from == text)
                    {
                        *value = Value::from(*to);
                        applied.push(format!("atajo {from} → {to}"));
                    }
                }
            }
            _ => {}
        }
    }
    applied
}

/// Reescribe `spawn noctalia msg …` por `spawn noct msg …` en `[keybinds]`.
///
/// Es idempotente. Se salta el cambio si `noct` no está en PATH para no crear
/// atajos que no podrían ejecutarse.
pub fn migrate_ipc_calls(path: &Path) -> Result<Option<String>> {
    if env::find("noct").is_none() {
        return Ok(None);
    }
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("no se pudo leer {}", path.display()))?;
    let mut doc: DocumentMut = text
        .parse()
        .with_context(|| format!("{} no es TOML válido", path.display()))?;

    let mut changed = 0usize;
    {
        let Some(table) = doc
            .as_table_mut()
            .get_mut("keybinds")
            .and_then(Item::as_table_mut)
        else {
            return Ok(None);
        };
        for (_, item) in table.iter_mut() {
            if rewrite_bind_value(item) {
                changed += 1;
            }
        }
    }

    if changed == 0 {
        return Ok(None);
    }
    std::fs::write(path, doc.to_string())
        .with_context(|| format!("no se pudo escribir {}", path.display()))?;
    Ok(Some(format!(
        "{changed} atajo(s) ahora usan `noct msg` (compatible con renombres de Noctalia)"
    )))
}

fn rewrite_bind_value(item: &mut Item) -> bool {
    let value = match item {
        Item::Value(value) => value,
        Item::Table(table) => match table.get_mut("action") {
            Some(Item::Value(value)) => value,
            _ => return false,
        },
        _ => return false,
    };
    let Some(text) = value.as_str() else {
        return false;
    };
    let Some(new) = text.strip_prefix("spawn noctalia msg ") else {
        return false;
    };
    *value = Value::from(format!("spawn noct msg {new}"));
    true
}

/// Reescribe `spawn-sh "noctalia msg …"` en el `config.kdl` de Niri.
fn migrate_niri_ipc_calls() -> Option<String> {
    env::find("noct")?;
    let path = env::app_config("niri").join("config.kdl");
    let text = std::fs::read_to_string(&path).ok()?;
    let updated = text.replace("spawn-sh \"noctalia msg ", "spawn-sh \"noct msg ");
    if updated == text {
        return None;
    }
    let changed = text.matches("spawn-sh \"noctalia msg ").count();
    std::fs::write(&path, updated).ok()?;
    Some(format!("{changed} atajo(s) de Niri ahora usan `noct msg`"))
}

fn timestamp() -> String {
    let seconds = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    seconds.to_string()
}

fn log_line(line: &str) {
    let dir = state_dir();
    if std::fs::create_dir_all(&dir).is_err() {
        return;
    }
    use std::io::Write;
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path())
    {
        let _ = writeln!(file, "[{}] {line}", timestamp());
    }
}

fn notify(summary: &str, body: &str) {
    if std::env::var_os("DBUS_SESSION_BUS_ADDRESS").is_none() {
        return;
    }
    let _ = Command::new("notify-send")
        .args(["-a", "Rakia", "-i", "rakia-icon", summary, body])
        .status();
}

fn write_report(outcome: &Outcome) {
    let dir = state_dir();
    if std::fs::create_dir_all(&dir).is_err() {
        return;
    }
    #[derive(Serialize)]
    struct Report<'a> {
        generated_at: String,
        #[serde(flatten)]
        outcome: &'a Outcome,
    }
    let report = Report {
        generated_at: timestamp(),
        outcome,
    };
    if let Ok(json) = serde_json::to_string_pretty(&report) {
        let _ = std::fs::write(report_path(), json + "\n");
    }
}

/// Ejecuta la pasada de compatibilidad.
///
/// Con `apply = false` solo valida e informa. Con `apply = true` repara,
/// migra los atajos a `noct msg` y mantiene la copia del último estado válido.
pub fn run(apply: bool) -> Result<Outcome> {
    let mut outcome = Outcome {
        config: config_path().display().to_string(),
        umbriel_version: env::capture("umbriel", &["--version"]),
        noctalia_version: env::capture("noctalia", &["--version"]),
        noct_binary: env::find("noct").map(|p| p.display().to_string()),
        ..Outcome::default()
    };

    let config = config_path();
    if !config.exists() {
        outcome
            .pending
            .push(format!("no existe {}", config.display()));
        write_report(&outcome);
        return Ok(outcome);
    }

    let previous_pending: Vec<String> = last_report()
        .and_then(|report| {
            report
                .get("pending")
                .and_then(|value| value.as_array())
                .cloned()
        })
        .map(|items| {
            items
                .iter()
                .filter_map(|value| value.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    let mut migration_error: Option<String> = None;
    if apply {
        for message in crate::config::fix_all() {
            outcome.repairs.push(message);
        }
        match migrate_ipc_calls(&config) {
            Ok(Some(message)) => outcome.repairs.push(message),
            Ok(None) => {}
            Err(err) => {
                migration_error = Some(format!("no se pudieron migrar los atajos: {err:#}"))
            }
        }
        if let Some(message) = migrate_niri_ipc_calls() {
            outcome.repairs.push(message);
        }
    }

    let before = validate(&config);
    outcome.valid_before = before.as_ref().map(|(valid, _)| *valid);

    if apply {
        if let Some((false, diagnostics)) = before.as_ref() {
            let mut repaired = Vec::new();
            let mut pending = Vec::new();
            for diag in diagnostics {
                if diag.severity == "error" && diag.message.contains("parsing") {
                    // Se resuelve restaurando el último estado válido, más abajo.
                    continue;
                }
                match plan(diag) {
                    Fix::RemovePath(path) => match repair(&config, |doc| remove_path(doc, &path)) {
                        Ok(true) => repaired.push(format!("clave retirada: {path}")),
                        Ok(false) => pending.push(diag.message.clone()),
                        Err(err) => {
                            pending.push(format!("{} ({err:#})", diag.message));
                        }
                    },
                    Fix::RemoveKeybind(chord) => {
                        match repair(&config, |doc| remove_keybind(doc, &chord)) {
                            Ok(true) => repaired
                                .push(format!("atajo retirado: {chord} (acción inexistente)")),
                            Ok(false) => pending.push(diag.message.clone()),
                            Err(err) => {
                                pending.push(format!("{} ({err:#})", diag.message));
                            }
                        }
                    }
                    Fix::Unfixable(message) => pending.push(message),
                }
            }
            outcome.repairs.extend(repaired);
            outcome.pending.extend(pending);
        }

        if let Ok(mut doc) = std::fs::read_to_string(&config)
            .unwrap_or_default()
            .parse::<DocumentMut>()
        {
            let applied = apply_keybind_aliases(&mut doc);
            if !applied.is_empty() {
                let _ = std::fs::write(&config, doc.to_string());
                outcome.repairs.extend(applied);
            }
        }
    }

    let mut after = validate(&config);
    outcome.valid_after = after.as_ref().map(|(valid, _)| *valid);

    let fatal_parse = after
        .as_ref()
        .map(|(_, diagnostics)| {
            diagnostics
                .iter()
                .any(|d| d.severity == "error" && d.message.contains("parsing"))
        })
        .unwrap_or(false);
    if apply && fatal_parse && snapshot_path().exists() {
        let broken = state_dir().join(format!("umbriel-config.broken-{}.toml", timestamp()));
        if std::fs::copy(&config, &broken).is_ok()
            && std::fs::copy(snapshot_path(), &config).is_ok()
        {
            outcome.restored = true;
            outcome.backup = Some(broken.display().to_string());
            outcome.repairs.push(format!(
                "configuración TOML ilegible restaurada desde {} (copia rota en {})",
                snapshot_path().display(),
                broken.display()
            ));
            after = validate(&config);
            outcome.valid_after = after.as_ref().map(|(valid, _)| *valid);
        }
    }

    if let Some((false, diagnostics)) = after.as_ref() {
        for diag in diagnostics {
            let entry = format!("{}:{}: {}", diag.line, diag.column, diag.message);
            if outcome
                .pending
                .iter()
                .any(|existing| existing == &entry || existing == &diag.message)
            {
                continue;
            }
            outcome.pending.push(entry);
        }
    }

    if apply && outcome.valid_after == Some(true) {
        if let Ok(text) = std::fs::read_to_string(&config) {
            let _ = std::fs::create_dir_all(state_dir());
            let _ = std::fs::write(snapshot_path(), text);
        }
    }

    if outcome.repairs.is_empty() && outcome.pending.is_empty() {
        outcome
            .notes
            .push("sin cambios: la configuración ya es válida".into());
    }

    if let Some(message) = migration_error {
        if outcome.valid_after != Some(true) && !outcome.pending.contains(&message) {
            outcome.pending.push(message);
        }
    }

    write_report(&outcome);
    if !outcome.repairs.is_empty() || previous_pending != outcome.pending {
        let mut parts = Vec::new();
        if !outcome.repairs.is_empty() {
            parts.push(format!("{} reparado(s)", outcome.repairs.len()));
        }
        if !outcome.pending.is_empty() {
            parts.push(format!("{} pendiente(s)", outcome.pending.len()));
        }
        log_line(&format!("{} · {}", parts.join(" · "), outcome.config));
    }

    if previous_pending != outcome.pending {
        if !outcome.pending.is_empty() {
            notify(
                "Rakia: revisa la compatibilidad",
                &format!(
                    "{} cambio(s) sin reparar automáticamente. Detalle en ~/.local/state/noct/compat.log",
                    outcome.pending.len()
                ),
            );
        } else if !outcome.repairs.is_empty() {
            notify(
                "Rakia: configuración reparada",
                &format!(
                    "{} ajuste(s) aplicados tras la actualización de Umbriel/Noctalia.",
                    outcome.repairs.len()
                ),
            );
        }
    }

    Ok(outcome)
}

fn repair<F>(path: &Path, change: F) -> Result<bool>
where
    F: FnOnce(&mut DocumentMut) -> bool,
{
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("no se pudo leer {}", path.display()))?;
    let mut doc: DocumentMut = text
        .parse()
        .with_context(|| format!("{} no es TOML válido", path.display()))?;
    if !change(&mut doc) {
        return Ok(false);
    }
    std::fs::write(path, doc.to_string())
        .with_context(|| format!("no se pudo escribir {}", path.display()))?;
    Ok(true)
}

/// Estado de la última pasada, si existe.
pub fn last_report() -> Option<serde_json::Value> {
    let raw = std::fs::read_to_string(report_path()).ok()?;
    serde_json::from_str(&raw).ok()
}

// ---------------------------------------------------------------------------
// Puente de IPC (`noct msg`)
// ---------------------------------------------------------------------------

/// Lista de comandos que acepta el Noctalia instalado, con caché invalidada
/// cuando cambia el binario.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct IpcCache {
    binary: String,
    fingerprint: u64,
    commands: Vec<String>,
}

fn ipc_cache_path() -> PathBuf {
    state_dir().join("noctalia-commands.json")
}

fn fingerprint(path: &Path) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    path.hash(&mut hasher);
    if let Ok(meta) = std::fs::metadata(path) {
        meta.len().hash(&mut hasher);
        if let Ok(modified) = meta.modified() {
            if let Ok(duration) = modified.duration_since(std::time::UNIX_EPOCH) {
                duration.as_nanos().hash(&mut hasher);
            }
        }
    }
    hasher.finish()
}

/// Binario de Noctalia en uso.
pub fn noctalia_binary() -> Option<PathBuf> {
    env::find("noctalia")
}

fn parse_help_commands(text: &str) -> Vec<String> {
    let mut commands = Vec::new();
    for line in text.lines() {
        let Some(rest) = line.strip_prefix("  ") else {
            continue;
        };
        if rest.starts_with(' ') || rest.starts_with('-') {
            continue;
        }
        let name: String = rest
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '-')
            .collect();
        if name.is_empty() {
            continue;
        }
        if !commands.contains(&name) {
            commands.push(name);
        }
    }
    commands
}

/// Comandos del Noctalia instalado (cacheado por huella del binario).
pub fn noctalia_commands() -> Option<Vec<String>> {
    let binary = noctalia_binary()?;
    let current = fingerprint(&binary);

    if let Ok(raw) = std::fs::read_to_string(ipc_cache_path()) {
        if let Ok(cache) = serde_json::from_str::<IpcCache>(&raw) {
            if cache.binary == binary.display().to_string() && cache.fingerprint == current {
                return Some(cache.commands);
            }
        }
    }

    let out = env::run(binary.to_string_lossy().as_ref(), &["msg", "--help"])?;
    if !out.success && out.stdout.is_empty() {
        return None;
    }
    let commands = parse_help_commands(&out.stdout);
    if commands.is_empty() {
        return None;
    }

    let cache = IpcCache {
        binary: binary.display().to_string(),
        fingerprint: current,
        commands: commands.clone(),
    };
    if std::fs::create_dir_all(state_dir()).is_ok() {
        if let Ok(json) = serde_json::to_string_pretty(&cache) {
            let _ = std::fs::write(ipc_cache_path(), json);
        }
    }
    Some(commands)
}

/// Traduce un verbo retirado al equivalente actual.
pub fn ipc_alias(command: &str) -> Option<Vec<String>> {
    IPC_ALIASES
        .iter()
        .find(|(from, _)| *from == command)
        .map(|(_, to)| to.iter().map(|s| s.to_string()).collect())
}

/// Ejecuta `noctalia msg` resolviendo alias.
///
/// Reemplaza el proceso actual con `exec`, así que no deja procesos extra.
/// Si el verbo no existe, registra el caso y devuelve 0: un atajo roto no debe
/// mostrar un error del compositor ni afectar a los demás.
pub fn exec_noctalia_msg(args: &[String]) -> Result<i32> {
    let Some(binary) = noctalia_binary() else {
        eprintln!("noct msg: no se encontró `noctalia` en PATH");
        return Ok(0);
    };

    let commands = noctalia_commands();
    let first = args.first().cloned().unwrap_or_default();
    let resolved: Vec<String> = match &commands {
        Some(list) if list.iter().any(|c| c == &first) => args.to_vec(),
        _ => match ipc_alias(&first) {
            Some(mapped) => mapped
                .into_iter()
                .chain(args.iter().skip(1).cloned())
                .collect(),
            None => {
                let detail = format!(
                    "noct msg: Noctalia no reconoce «{first}» ({} comandos disponibles)",
                    commands.as_ref().map(|c| c.len()).unwrap_or(0)
                );
                eprintln!("{detail}");
                log_line(&detail);
                return Ok(0);
            }
        },
    };

    if resolved != args {
        log_line(&format!(
            "alias IPC: {} → {}",
            args.join(" "),
            resolved.join(" ")
        ));
    }

    use std::os::unix::process::CommandExt;
    let error = Command::new(&binary).arg("msg").args(&resolved).exec();
    Err(anyhow::Error::new(error).context("no se pudo ejecutar noctalia msg"))
}

/// Acciones de Umbriel disponibles (para diagnóstico y pruebas).
pub fn umbriel_actions() -> Vec<String> {
    let Some(out) = env::run("umbriel", &["msg", "--help"]) else {
        return Vec::new();
    };
    parse_help_commands(&out.stdout)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_validation_json() {
        let raw = r#"{"diagnostics":[{"column":13,"file":"bad.toml","line":13,"message":"unknown key appearance.blur.bogus_key","severity":"warning"}],"path":"bad.toml","valid":false}"#;
        let (valid, diagnostics) = parse_validation(raw).expect("json");
        assert!(!valid);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].line, 13);
    }

    #[test]
    fn classifies_diagnostics() {
        let diag = |message: &str| Diagnostic {
            line: 1,
            column: 1,
            severity: "warning".into(),
            message: message.into(),
        };
        assert_eq!(
            plan(&diag("unknown key appearance.blur.bogus_key")),
            Fix::RemovePath("appearance.blur.bogus_key".into())
        );
        assert_eq!(
            plan(&diag(
                "ignoring keybind 'Mod+Z' (unknown action 'not-a-real-action')"
            )),
            Fix::RemoveKeybind("Mod+Z".into())
        );
        assert_eq!(
            plan(&diag("ignoring colors.title_bar (expected table)")),
            Fix::RemovePath("colors.title_bar".into())
        );
        assert!(matches!(
            plan(&diag("duplicate key `a` in table `b`")),
            Fix::Unfixable(_)
        ));
    }

    #[test]
    fn splits_quoted_paths() {
        assert_eq!(split_path("a.b.c"), vec!["a", "b", "c"]);
        assert_eq!(
            split_path("window_rule.0.match.app_id"),
            vec!["window_rule", "0", "match", "app_id"]
        );
        assert_eq!(
            split_path("window_rule[0].nonexistent_rule_key"),
            vec!["window_rule", "0", "nonexistent_rule_key"]
        );
        assert_eq!(
            split_path(r#"a."weird.key".c"#),
            vec!["a", "weird.key", "c"]
        );
    }

    #[test]
    fn removes_nested_key() {
        let mut doc: DocumentMut =
            "[appearance.blur]\nradius = 3\nbogus_key = 7\n\n[keybinds]\n\"Mod+Z\" = \"x\"\n"
                .parse()
                .expect("toml");
        assert!(remove_path(&mut doc, "appearance.blur.bogus_key"));
        assert!(!doc.to_string().contains("bogus_key"));
        assert!(remove_keybind(&mut doc, "Mod+Z"));
        assert!(!doc.to_string().contains("Mod+Z"));
    }

    #[test]
    fn removes_key_from_array_of_tables() {
        let mut doc: DocumentMut = "[[window_rule]]\nmatch.title = \"x\"\nbad = 1\n"
            .parse()
            .expect("toml");
        assert!(remove_path(&mut doc, "window_rule.0.bad"));
        assert!(!doc.to_string().contains("bad = 1"));
    }

    #[test]
    fn rewrites_noctalia_binds() {
        let mut doc: DocumentMut =
            "[keybinds]\n\"Mod+S\" = \"spawn noctalia msg settings-toggle\"\n\"Mod+Return\" = { action = \"spawn foot\", repeat = false }\n"
                .parse()
                .expect("toml");
        let table = doc
            .as_table_mut()
            .get_mut("keybinds")
            .and_then(Item::as_table_mut)
            .expect("keybinds");
        let mut changed = 0;
        for (_, item) in table.iter_mut() {
            if rewrite_bind_value(item) {
                changed += 1;
            }
        }
        assert_eq!(changed, 1);
        let text = doc.to_string();
        assert!(text.contains("spawn noct msg settings-toggle"));
        assert!(text.contains("spawn foot"));
    }

    #[test]
    fn parses_help_lists() {
        let help = "Available actions:\n\nApps\n  spawn <cmd...>           Run a command\n  spawn-floating <cmd...>  Float\n\nFocus\n  window-focus-left        Focus\n";
        assert_eq!(
            parse_help_commands(help),
            vec!["spawn", "spawn-floating", "window-focus-left"]
        );
    }

    #[test]
    fn resolves_ipc_aliases() {
        assert_eq!(
            ipc_alias("launcher-toggle"),
            Some(vec!["panel-toggle".to_string(), "launcher".to_string()])
        );
        assert_eq!(ipc_alias("settings-toggle"), None);
    }
}
