//! Lectura, validación y migración de atajos de Umbriel y Niri.

use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde::Serialize;

use crate::env;
use crate::kdl;

#[derive(Debug, Clone, Serialize)]
pub struct Keybind {
    pub chord: String,
    pub action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repeat: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub submap: Option<String>,
    pub compositor: String,
    pub file: String,
    pub line: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct Conflict {
    pub compositor: String,
    pub chord: String,
    pub normalized: String,
    pub bindings: Vec<Keybind>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LegacyRef {
    pub file: String,
    pub line: usize,
    pub command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MigrationOutcome {
    pub file: String,
    pub replacements: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backup: Option<String>,
}

pub fn umbriel_config_path() -> PathBuf {
    env::app_config("umbriel").join("config.toml")
}

pub fn niri_config_dir() -> PathBuf {
    env::app_config("niri")
}

/// Atajos definidos en un `config.toml` de Umbriel.
pub fn load_umbriel(path: &Path) -> Result<Vec<Keybind>> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("no se pudo leer {}", path.display()))?;
    Ok(parse_umbriel_keybinds(&text, path))
}

pub fn load_umbriel_default() -> Result<Vec<Keybind>> {
    let path = umbriel_config_path();
    let mut binds = if path.exists() {
        load_umbriel(&path)?
    } else {
        Vec::new()
    };

    let managed = managed_path();
    for bind in load_managed()? {
        binds.push(Keybind {
            chord: bind.chord,
            action: bind.action,
            repeat: bind.repeat,
            submap: None,
            compositor: "umbriel".to_string(),
            file: managed.display().to_string(),
            line: 0,
        });
    }

    Ok(binds)
}

/// Atajos definidos en los KDL de Niri.
pub fn load_niri_default() -> Result<Vec<Keybind>> {
    let dir = niri_config_dir();
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut files = Vec::new();
    collect_files(&dir, &["kdl"], 3, &mut files);
    let mut binds = Vec::new();
    for file in files {
        let Ok(text) = std::fs::read_to_string(&file) else {
            continue;
        };
        for (idx, raw) in text.lines().enumerate() {
            if let Some((chord, action)) = parse_niri_line(raw) {
                binds.push(Keybind {
                    chord,
                    action,
                    repeat: None,
                    submap: None,
                    compositor: "niri".to_string(),
                    file: file.display().to_string(),
                    line: idx + 1,
                });
            }
        }
    }
    Ok(binds)
}

fn collect_files(dir: &Path, extensions: &[&str], depth: usize, out: &mut Vec<PathBuf>) {
    if depth == 0 {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_files(&path, extensions, depth - 1, out);
        } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if extensions.contains(&ext) {
                out.push(path);
            }
        }
    }
}

fn extract_toml_action(value: &str) -> String {
    if let Some(rest) = value.strip_prefix('"') {
        if let Some(end) = rest.find('"') {
            return rest[..end].to_string();
        }
    }
    if let Some(pos) = value.find("action") {
        let after = &value[pos..];
        if let Some(eq) = after.find('=') {
            let tail = after[eq + 1..].trim();
            if let Some(rest) = tail.strip_prefix('"') {
                if let Some(end) = rest.find('"') {
                    return rest[..end].to_string();
                }
            }
        }
    }
    value.trim().trim_matches('"').to_string()
}

fn parse_umbriel_keybinds(text: &str, path: &Path) -> Vec<Keybind> {
    let mut out = Vec::new();
    let mut in_section = false;

    for (idx, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.starts_with('[') {
            in_section = line == "[keybinds]";
            continue;
        }
        if !in_section || line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let chord = key.trim().trim_matches('"').to_string();
        if chord.is_empty() {
            continue;
        }
        let action = extract_toml_action(value.trim());
        if action.is_empty() {
            continue;
        }
        let repeat = if value.contains("repeat = false") {
            Some(false)
        } else if value.contains("repeat = true") {
            Some(true)
        } else {
            None
        };
        let submap = value.find("submap").and_then(|pos| {
            let tail = &value[pos..];
            let eq = tail.find('=')?;
            let rest = tail[eq + 1..].trim().strip_prefix('"')?;
            let end = rest.find('"')?;
            Some(rest[..end].to_string())
        });

        out.push(Keybind {
            chord,
            action,
            repeat,
            submap,
            compositor: "umbriel".to_string(),
            file: path.display().to_string(),
            line: idx + 1,
        });
    }

    out
}

fn extract_spawn(rest: &str) -> Option<String> {
    for marker in ["spawn-sh \"", "spawn \""] {
        if let Some(pos) = rest.find(marker) {
            let after = &rest[pos + marker.len()..];
            if let Some(end) = after.find('"') {
                return Some(after[..end].to_string());
            }
        }
    }
    None
}

fn parse_niri_line(raw: &str) -> Option<(String, String)> {
    let line = raw.trim();
    if line.is_empty() || line.starts_with('#') || line.starts_with('}') {
        return None;
    }
    let chord_end = line.find(char::is_whitespace)?;
    let chord = &line[..chord_end];
    if chord.is_empty() || chord.contains('=') || chord == "binds" {
        return None;
    }
    let rest = &line[chord_end..];
    let action = extract_spawn(rest)?;
    Some((chord.to_string(), action))
}

/// Normaliza un chord para comparar `Mod`/`Super` y el orden de modificadores.
pub fn normalize_chord(chord: &str) -> String {
    let mut mods: Vec<String> = Vec::new();
    let mut key = String::new();

    for part in chord.split('+') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        match part.to_ascii_lowercase().as_str() {
            "mod" | "super" | "logo" | "win" => mods.push("mod".to_string()),
            "ctrl" | "control" => mods.push("ctrl".to_string()),
            "alt" | "option" => mods.push("alt".to_string()),
            "shift" => mods.push("shift".to_string()),
            other => key = other.to_string(),
        }
    }

    mods.sort();
    mods.dedup();
    mods.push(key);
    mods.join("+")
}

/// Agrupa los atajos que comparten el mismo chord normalizado.
pub fn find_conflicts(binds: &[Keybind]) -> Vec<Conflict> {
    let mut groups: Vec<(String, String, String, Vec<Keybind>)> = Vec::new();

    for bind in binds {
        let normalized = normalize_chord(&bind.chord);
        let key = format!("{}|{}", bind.compositor, normalized);
        if let Some(group) = groups.iter_mut().find(|(k, _, _, _)| *k == key) {
            group.3.push(bind.clone());
        } else {
            groups.push((
                key,
                bind.compositor.clone(),
                bind.chord.clone(),
                vec![bind.clone()],
            ));
        }
    }

    groups
        .into_iter()
        .filter(|(_, _, _, bindings)| bindings.len() > 1)
        .map(|(_, compositor, chord, bindings)| Conflict {
            compositor,
            chord,
            normalized: normalize_chord(&bindings[0].chord),
            bindings,
        })
        .collect()
}

fn extract_legacy(line: &str) -> Option<String> {
    let idx = line.find("qs -c noctalia-shell")?;
    let rest = &line[idx..];
    let end = rest.find(['"', '\'', ';']).unwrap_or(rest.len());
    Some(rest[..end].trim().to_string())
}

/// Busca llamadas al shell Quickshell en configs de Umbriel y Niri.
pub fn scan_legacy() -> Vec<LegacyRef> {
    let mut files = Vec::new();
    collect_files(&env::app_config("umbriel"), &["toml"], 3, &mut files);
    collect_files(&env::app_config("niri"), &["kdl"], 3, &mut files);

    let mut refs = Vec::new();
    for file in files {
        let Ok(text) = std::fs::read_to_string(&file) else {
            continue;
        };
        for (idx, line) in text.lines().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') || trimmed.starts_with("//") {
                continue;
            }
            if let Some(command) = extract_legacy(line) {
                refs.push(LegacyRef {
                    file: file.display().to_string(),
                    line: idx + 1,
                    suggestion: migrate_command(&command),
                    command,
                });
            }
        }
    }
    refs
}

/// Traduce `qs -c noctalia-shell ipc call ...` al equivalente `noctalia msg ...`.
pub fn migrate_command(input: &str) -> Option<String> {
    let idx = input.find("qs -c noctalia-shell")?;
    let tail = input[idx + "qs -c noctalia-shell".len()..].trim();
    let tail = tail.strip_prefix("ipc").unwrap_or(tail).trim();
    let tail = tail.strip_prefix("call").unwrap_or(tail).trim();

    let mut parts = tail.split_whitespace();
    let target = parts.next()?;
    let action = parts.next().unwrap_or("");

    let mapped = match (target, action) {
        ("launcher", "toggle") | ("launcher", "open") => "noctalia msg panel-toggle launcher",
        ("launcher", "close") => "noctalia msg panel-close launcher",
        ("controlCenter", "toggle") | ("control-center", "toggle") => {
            "noctalia msg panel-toggle control-center"
        }
        ("wallpaper", "toggle") => "noctalia msg panel-toggle wallpaper",
        ("settings", "toggle") => "noctalia msg settings-toggle",
        ("settings", "open") => "noctalia msg settings-open",
        ("sessionMenu", "toggle") => "noctalia msg panel-toggle session",
        ("lockScreen", "lock") => "noctalia msg session lock",
        ("lockScreen", "unlock") => "noctalia msg session lock",
        ("volume", "up") => "noctalia msg volume-up",
        ("volume", "down") => "noctalia msg volume-down",
        ("volume", "increase") => "noctalia msg volume-up",
        ("volume", "decrease") => "noctalia msg volume-down",
        ("volume", "mute") | ("volume", "muteOutput") => "noctalia msg volume-mute",
        ("volume", "muteInput") => "noctalia msg mic-mute",
        ("brightness", "up") => "noctalia msg brightness-up",
        ("brightness", "down") => "noctalia msg brightness-down",
        ("brightness", "increase") => "noctalia msg brightness-up",
        ("brightness", "decrease") => "noctalia msg brightness-down",
        ("notifications", "toggleDnd") => "noctalia msg notification-dnd-toggle",
        ("media", "playPause") | ("media", "toggle") => "noctalia msg media toggle",
        ("media", other) => return Some(format!("noctalia msg media {other}")),
        _ => return None,
    };

    Some(mapped.to_string())
}

/// Reescribe las llamadas obsoletas conservando una copia de seguridad.
pub fn apply_migration(refs: &[LegacyRef]) -> Result<Vec<MigrationOutcome>> {
    let mut by_file: Vec<(String, Vec<&LegacyRef>)> = Vec::new();
    for item in refs {
        if item.suggestion.is_none() {
            continue;
        }
        if let Some(entry) = by_file.iter_mut().find(|(f, _)| *f == item.file) {
            entry.1.push(item);
        } else {
            by_file.push((item.file.clone(), vec![item]));
        }
    }

    let mut outcomes = Vec::new();
    for (file, items) in by_file {
        let path = Path::new(&file);
        let Ok(mut text) = std::fs::read_to_string(path) else {
            continue;
        };
        let mut replacements = 0;
        for item in &items {
            let Some(suggestion) = &item.suggestion else {
                continue;
            };
            if text.contains(&item.command) {
                text = text.replace(&item.command, suggestion);
                replacements += 1;
            }
        }
        if replacements == 0 {
            continue;
        }
        let backup = path.with_extension(format!(
            "{}.noct.bak",
            path.extension().and_then(|e| e.to_str()).unwrap_or("cfg")
        ));
        std::fs::copy(path, &backup).with_context(|| {
            format!(
                "no se pudo crear la copia de seguridad {}",
                backup.display()
            )
        })?;
        std::fs::write(path, text)
            .with_context(|| format!("no se pudo escribir {}", path.display()))?;
        outcomes.push(MigrationOutcome {
            file,
            replacements,
            backup: Some(backup.display().to_string()),
        });
    }
    Ok(outcomes)
}

/// Ruta del archivo de atajos gestionado por Noct.
pub fn managed_path() -> PathBuf {
    env::app_config("umbriel")
        .join("noct")
        .join("keybinds.toml")
}

/// Atajo escrito en el archivo gestionado.
#[derive(Debug, Clone, Serialize)]
pub struct ManagedBind {
    pub chord: String,
    pub action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repeat: Option<bool>,
}

pub fn load_managed() -> Result<Vec<ManagedBind>> {
    let path = managed_path();
    if !path.exists() {
        return Ok(Vec::new());
    }
    let text = std::fs::read_to_string(&path)
        .with_context(|| format!("no se pudo leer {}", path.display()))?;
    let value: toml::Value =
        toml::from_str(&text).with_context(|| format!("{} no es TOML válido", path.display()))?;
    let mut out = Vec::new();
    if let Some(binds) = value.get("keybinds").and_then(|v| v.as_table()) {
        for (chord, raw) in binds {
            let (action, repeat) = match raw {
                toml::Value::String(s) => (s.clone(), None),
                toml::Value::Table(t) => {
                    let action = t
                        .get("action")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default()
                        .to_string();
                    let repeat = t.get("repeat").and_then(|v| v.as_bool());
                    (action, repeat)
                }
                _ => continue,
            };
            if !action.is_empty() {
                out.push(ManagedBind {
                    chord: chord.clone(),
                    action,
                    repeat,
                });
            }
        }
    }
    out.sort_by(|a, b| a.chord.cmp(&b.chord));
    Ok(out)
}

fn toml_quote(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

fn write_managed(binds: &[ManagedBind]) -> Result<()> {
    let path = managed_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("no se pudo crear {}", parent.display()))?;
    }

    let mut text = String::from(
        "# Gestionado por Noct. Este archivo se incluye desde ~/.config/umbriel/config.toml.\n\
         # Cambia los atajos con `noct keys set` para mantener la validación y el rollback.\n\n\
         [keybinds]\n",
    );
    for bind in binds {
        let mut fields = vec![format!("action = {}", toml_quote(&bind.action))];
        if let Some(repeat) = bind.repeat {
            fields.push(format!("repeat = {repeat}"));
        }
        text.push_str(&format!(
            "{} = {{ {} }}\n",
            toml_quote(&bind.chord),
            fields.join(", ")
        ));
    }

    std::fs::write(&path, text).with_context(|| format!("no se pudo escribir {}", path.display()))
}

/// Añade o reemplaza un atajo en el archivo gestionado.
pub fn set_managed(chord: &str, action: &str, repeat: Option<bool>) -> Result<()> {
    let mut binds = load_managed()?;
    if let Some(existing) = binds.iter_mut().find(|b| b.chord == chord) {
        existing.action = action.to_string();
        existing.repeat = repeat;
    } else {
        binds.push(ManagedBind {
            chord: chord.to_string(),
            action: action.to_string(),
            repeat,
        });
    }
    binds.sort_by(|a, b| a.chord.cmp(&b.chord));
    write_managed(&binds)
}

/// Elimina un atajo gestionado. Devuelve si existía.
pub fn unset_managed(chord: &str) -> Result<bool> {
    let mut binds = load_managed()?;
    let before = binds.len();
    binds.retain(|b| b.chord != chord);
    if binds.len() == before {
        return Ok(false);
    }
    write_managed(&binds)?;
    Ok(true)
}

/// Resultado de asegurar el include de Noct en el config de Umbriel.
#[derive(Debug, Clone, Serialize)]
pub struct IncludeOutcome {
    pub file: String,
    pub added: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backup: Option<String>,
}

/// Añade el archivo gestionado a `[include.optional].files` si no está.
pub fn ensure_include() -> Result<IncludeOutcome> {
    let config_path = umbriel_config_path();
    if !config_path.exists() {
        bail!(
            "no existe {}; ejecuta Umbriel una vez o copia su configuración de ejemplo",
            config_path.display()
        );
    }

    let wanted = managed_path();
    let wanted_str = wanted.display().to_string();
    let text = std::fs::read_to_string(&config_path)
        .with_context(|| format!("no se pudo leer {}", config_path.display()))?;

    if text.contains(&wanted_str) {
        return Ok(IncludeOutcome {
            file: config_path.display().to_string(),
            added: false,
            backup: None,
        });
    }

    let mut doc: toml_edit::DocumentMut = text
        .parse()
        .with_context(|| format!("{} no es TOML válido", config_path.display()))?;

    let include = doc
        .as_table_mut()
        .entry("include")
        .or_insert(toml_edit::Item::Table(toml_edit::Table::new()));
    let include_table = include
        .as_table_mut()
        .ok_or_else(|| anyhow::anyhow!("`include` no es una tabla en config.toml"))?;
    let optional = include_table
        .entry("optional")
        .or_insert(toml_edit::Item::Table(toml_edit::Table::new()));
    let optional_table = optional
        .as_table_mut()
        .ok_or_else(|| anyhow::anyhow!("`include.optional` no es una tabla en config.toml"))?;
    let files = optional_table
        .entry("files")
        .or_insert(toml_edit::Item::Value(toml_edit::Value::Array(
            toml_edit::Array::new(),
        )));
    let array = files
        .as_array_mut()
        .ok_or_else(|| anyhow::anyhow!("`include.optional.files` no es una lista"))?;
    array.push(wanted_str.as_str());

    let backup = config_path.with_extension("toml.noct.bak");
    std::fs::copy(&config_path, &backup)
        .with_context(|| format!("no se pudo crear la copia {}", backup.display()))?;
    std::fs::write(&config_path, doc.to_string())
        .with_context(|| format!("no se pudo escribir {}", config_path.display()))?;

    Ok(IncludeOutcome {
        file: config_path.display().to_string(),
        added: true,
        backup: Some(backup.display().to_string()),
    })
}

/// Catálogo de acciones disponibles en Umbriel.
pub fn umbriel_actions() -> Vec<String> {
    let text = env::capture("umbriel", &["msg", "--help"]).unwrap_or_default();
    parse_action_catalog(&text)
}

fn parse_action_catalog(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in text.lines() {
        if !line.starts_with("  ") {
            continue;
        }
        let trimmed = line.trim_start();
        let Some(token) = trimmed.split_whitespace().next() else {
            continue;
        };
        if token.starts_with('-') || token == "Usage:" {
            continue;
        }
        let clean = token.trim_end_matches(':');
        if !clean.is_empty() && !out.iter().any(|a| a == clean) {
            out.push(clean.to_string());
        }
    }
    out
}

/// Comprueba una acción contra el catálogo, aceptando `spawn:...` y variantes.
pub fn is_valid_action(action: &str, catalog: &[String]) -> bool {
    let base = action.split(':').next().unwrap_or(action);
    catalog
        .iter()
        .any(|entry| entry.split(':').next().unwrap_or(entry) == base)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn normalizes_mod_and_super() {
        assert_eq!(normalize_chord("Mod+Shift+T"), "mod+shift+t");
        assert_eq!(normalize_chord("Shift+Super+t"), "mod+shift+t");
        assert_eq!(normalize_chord("Ctrl+Alt+Delete"), "alt+ctrl+delete");
    }

    #[test]
    fn parses_umbriel_keybinds() {
        let text = r#"
[keybinds]
"Mod+Return" = { action = "spawn:kitty", repeat = false }
"Mod+Q" = "window-close"
"#;
        let binds = parse_umbriel_keybinds(text, Path::new("test.toml"));
        assert_eq!(binds.len(), 2);
        assert_eq!(binds[0].chord, "Mod+Return");
        assert_eq!(binds[0].action, "spawn:kitty");
        assert_eq!(binds[0].repeat, Some(false));
        assert_eq!(binds[1].action, "window-close");
    }

    #[test]
    fn migrates_quickshell_calls() {
        assert_eq!(
            migrate_command("qs -c noctalia-shell ipc call launcher toggle").as_deref(),
            Some("noctalia msg panel-toggle launcher")
        );
        assert_eq!(
            migrate_command("qs -c noctalia-shell ipc call lockScreen lock").as_deref(),
            Some("noctalia msg session lock")
        );
        assert!(migrate_command("qs -c noctalia-shell ipc call unknown thing").is_none());
    }
}

// ---------------------------------------------------------------------------
// Traducción de atajos Niri → Umbriel
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct TranslatedBind {
    pub chord: String,
    pub action: String,
    pub source_action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repeat: Option<bool>,
    pub note: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct UntranslatedBind {
    pub chord: String,
    pub action: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct TranslationReport {
    pub from: String,
    pub to: String,
    pub translated: Vec<TranslatedBind>,
    pub skipped: Vec<UntranslatedBind>,
}

fn niri_chord_to_umbriel(chord: &str) -> String {
    let mut mods: Vec<&str> = Vec::new();
    let mut key = String::new();
    for part in chord.split('+') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        match part.to_ascii_lowercase().as_str() {
            "mod" | "super" | "logo" | "win" => mods.push("Mod"),
            "ctrl" | "control" => mods.push("Ctrl"),
            "alt" | "option" => mods.push("Alt"),
            "shift" => mods.push("Shift"),
            _ => key = niri_key_name(part),
        }
    }
    // Orden estable: Mod, Ctrl, Alt, Shift.
    let order = ["Mod", "Ctrl", "Alt", "Shift"];
    let mut out: Vec<String> = order
        .iter()
        .filter(|m| mods.contains(m))
        .map(|m| m.to_string())
        .collect();
    if !key.is_empty() {
        out.push(key);
    }
    out.join("+")
}

fn niri_key_name(key: &str) -> String {
    let lower = key.to_ascii_lowercase();
    match lower.as_str() {
        "escape" => "Escape".into(),
        "return" => "Return".into(),
        "tab" => "Tab".into(),
        "space" => "Space".into(),
        "comma" => "Comma".into(),
        "period" => "Period".into(),
        "minus" => "Minus".into(),
        "equal" => "Equal".into(),
        "delete" => "Delete".into(),
        "backspace" => "Backspace".into(),
        "home" => "Home".into(),
        "end" => "End".into(),
        "up" => "Up".into(),
        "down" => "Down".into(),
        "left" => "Left".into(),
        "right" => "Right".into(),
        "wheelscrolldown" => "WheelDown".into(),
        "wheelscrollup" => "WheelUp".into(),
        "wheelscrollleft" => "WheelLeft".into(),
        "wheelscrollright" => "WheelRight".into(),
        "bracketleft" => "BracketLeft".into(),
        "bracketright" => "BracketRight".into(),
        _ => {
            if lower.starts_with("xf86") {
                // XF86AudioRaiseVolume conserva su forma original.
                key.to_string()
            } else if key.len() == 1 {
                key.to_ascii_uppercase()
            } else {
                let mut chars = key.chars();
                match chars.next() {
                    Some(first) => format!("{}{}", first.to_ascii_uppercase(), chars.as_str()),
                    None => String::new(),
                }
            }
        }
    }
}

fn percent_delta(value: &str) -> Option<String> {
    let value = value.trim();
    let sign = if value.starts_with('-') { -1.0 } else { 1.0 };
    let number = value.trim_start_matches(['-', '+']).trim_end_matches('%');
    let parsed: f64 = number.parse().ok()?;
    Some(format!("{}", sign * parsed / 100.0))
}

fn translate_niri_action(action: &str, args: &[String]) -> Option<String> {
    let action = action.trim();
    if action == "spawn" || action == "spawn-sh" {
        if args.is_empty() {
            return None;
        }
        // No traducir comandos con shell complejo: Umbriel no documenta cómo
        // interpreta `spawn:` y podríamos escribir un atajo roto.
        let simple = args.iter().all(|arg| {
            arg.chars()
                .all(|c| c.is_ascii_alphanumeric() || " -_.:/@%+=,".contains(c))
        });
        if !simple {
            return None;
        }
        let command = if action == "spawn-sh" {
            args.join(" ")
        } else {
            args.iter()
                .map(|arg| {
                    let safe = !arg.is_empty()
                        && arg
                            .chars()
                            .all(|c| c.is_ascii_alphanumeric() || "-_./:@%+=,".contains(c));
                    if safe {
                        arg.clone()
                    } else {
                        format!("'{}'", arg.replace('\'', "'\\''"))
                    }
                })
                .collect::<Vec<_>>()
                .join(" ")
        };
        return Some(format!("spawn:{command}"));
    }

    let first = args.first().map(String::as_str).unwrap_or_default();
    let mapped = match action {
        "close-window" => "window-close".to_string(),
        "focus-column-left" => "window-focus-left".to_string(),
        "focus-column-right" => "window-focus-right".to_string(),
        "focus-window-up" => "window-focus-up".to_string(),
        "focus-window-down" => "window-focus-down".to_string(),
        "focus-column-first" => "column-focus-first".to_string(),
        "focus-column-last" => "column-focus-last".to_string(),
        "move-column-left" => "column-move-left".to_string(),
        "move-column-right" => "column-move-right".to_string(),
        "move-window-up" => "window-move-up".to_string(),
        "move-window-down" => "window-move-down".to_string(),
        "move-column-to-first" => "column-move-to-first".to_string(),
        "move-column-to-last" => "column-move-to-last".to_string(),
        "focus-monitor-left" => "output-focus-left".to_string(),
        "focus-monitor-right" => "output-focus-right".to_string(),
        "focus-monitor-up" => "output-focus-up".to_string(),
        "focus-monitor-down" => "output-focus-down".to_string(),
        "move-column-to-monitor-left" => "column-move-to-output-left".to_string(),
        "move-column-to-monitor-right" => "column-move-to-output-right".to_string(),
        "move-column-to-monitor-up" => "column-move-to-output-up".to_string(),
        "move-column-to-monitor-down" => "column-move-to-output-down".to_string(),
        "switch-focus-between-floating-and-tiling" => "window-focus-switch-floating".to_string(),
        "consume-or-expel-window-left" => "window-consume-or-expel-left".to_string(),
        "consume-or-expel-window-right" => "window-consume-or-expel-right".to_string(),
        "consume-window-into-column" => "window-consume-right".to_string(),
        "switch-preset-column-width" => "window-cycle-primary-extent".to_string(),
        "switch-preset-column-width-back" => "window-cycle-primary-extent-back".to_string(),
        "switch-preset-window-height" => "window-cycle-secondary-extent".to_string(),
        "switch-preset-window-height-back" => "window-cycle-secondary-extent-back".to_string(),
        "expand-column-to-available-width" => "window-set-primary-extent:1.0".to_string(),
        "center-column" => "column-center".to_string(),
        "set-column-width" => format!(
            "window-modify-primary-extent:{}",
            percent_delta(first).unwrap_or_default()
        ),
        "set-window-height" => format!(
            "window-modify-secondary-extent:{}",
            percent_delta(first).unwrap_or_default()
        ),
        "toggle-window-floating" => "window-toggle-floating".to_string(),
        "maximize-column" => "window-set-primary-extent:1.0".to_string(),
        "fullscreen-window" => "window-toggle-fullscreen".to_string(),
        "toggle-column-tabbed-display" => "column-toggle-tabbed".to_string(),
        "maximize-window-to-edges" => "window-toggle-maximize-to-edges".to_string(),
        "focus-workspace-down" => "workspace-next".to_string(),
        "focus-workspace-up" => "workspace-previous".to_string(),
        "focus-workspace-previous" => "workspace-focus-last".to_string(),
        "move-column-to-workspace-down" => "column-move-to-workspace-next".to_string(),
        "move-column-to-workspace-up" => "column-move-to-workspace-previous".to_string(),
        "focus-workspace" => {
            if first.is_empty() {
                return None;
            }
            format!("workspace-switch:{first}")
        }
        "move-column-to-workspace" => {
            if first.is_empty() {
                return None;
            }
            format!("column-move-to-workspace:{first}")
        }
        "toggle-overview" => "overview-toggle".to_string(),
        "show-hotkey-overlay" => "cheatsheet-open".to_string(),
        "toggle-keyboard-shortcuts-inhibit" => "shortcuts-inhibit-toggle".to_string(),
        "quit" => "session-quit".to_string(),
        "power-off-monitors" => "dpms-off".to_string(),
        _ => return None,
    };

    if mapped.ends_with(':') {
        return None;
    }
    Some(mapped)
}

fn niri_value_string(value: &kdl::Value) -> Option<String> {
    match value {
        kdl::Value::Str(s) | kdl::Value::Ident(s) => Some(s.clone()),
        kdl::Value::Number(n) => Some(if n.fract() == 0.0 {
            format!("{}", *n as i64)
        } else {
            format!("{n}")
        }),
        kdl::Value::Bool(b) => Some(b.to_string()),
    }
}

fn load_niri_bind_nodes() -> Vec<kdl::Node> {
    let dir = niri_config_dir();
    let mut files = Vec::new();
    collect_files(&dir, &["kdl"], 3, &mut files);
    let mut binds = Vec::new();
    for file in files {
        if file.file_name().and_then(|n| n.to_str()) == Some("noct-bridge.kdl") {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&file) else {
            continue;
        };
        for node in kdl::parse(&text) {
            if node.name != "binds" {
                continue;
            }
            binds.extend(node.children);
        }
    }
    binds
}

/// Propone la traducción de los atajos de Niri al formato de Umbriel.
pub fn translate_niri_to_umbriel() -> Result<TranslationReport> {
    let catalog = umbriel_actions();
    let existing = load_umbriel_default()?;
    let existing_chords: Vec<String> = existing
        .iter()
        .map(|bind| normalize_chord(&bind.chord))
        .collect();

    let mut translated = Vec::new();
    let mut skipped = Vec::new();
    let mut seen: Vec<String> = Vec::new();

    for node in load_niri_bind_nodes() {
        let chord = niri_chord_to_umbriel(&node.name);
        let Some(action_node) = node.children.first() else {
            continue;
        };
        let args: Vec<String> = action_node
            .args
            .iter()
            .filter_map(niri_value_string)
            .collect();
        let source_action = if args.is_empty() {
            action_node.name.clone()
        } else {
            format!("{} {}", action_node.name, args.join(" "))
        };

        let Some(action) = translate_niri_action(&action_node.name, &args) else {
            let reason = if matches!(action_node.name.as_str(), "spawn" | "spawn-sh") {
                "comando con shell; tradúcelo a mano".into()
            } else {
                "sin equivalente en Umbriel".into()
            };
            skipped.push(UntranslatedBind {
                chord,
                action: source_action,
                reason,
            });
            continue;
        };
        if !is_valid_action(&action, &catalog) {
            skipped.push(UntranslatedBind {
                chord,
                action: source_action,
                reason: format!("Umbriel no reconoce la acción '{action}'"),
            });
            continue;
        }

        let normalized = normalize_chord(&chord);
        if existing_chords.contains(&normalized) {
            skipped.push(UntranslatedBind {
                chord,
                action: source_action,
                reason: "el chord ya existe en Umbriel".into(),
            });
            continue;
        }
        if seen.contains(&normalized) {
            skipped.push(UntranslatedBind {
                chord,
                action: source_action,
                reason: "chord duplicado en Niri".into(),
            });
            continue;
        }
        seen.push(normalized);

        let repeat = action_node
            .prop("repeat")
            .and_then(|v| v.as_bool())
            .or_else(|| node.prop("repeat").and_then(|v| v.as_bool()));
        let note = if node.props.iter().any(|(k, _)| k == "hotkey-overlay-title") {
            "etiqueta del overlay descartada".to_string()
        } else {
            String::new()
        };

        translated.push(TranslatedBind {
            chord,
            action,
            source_action,
            repeat,
            note,
        });
    }

    Ok(TranslationReport {
        from: "niri".into(),
        to: "umbriel".into(),
        translated,
        skipped,
    })
}

/// Escribe una lista de atajos en el archivo gestionado (mezclando con los existentes).
pub fn merge_managed(binds: &[ManagedBind]) -> Result<usize> {
    let mut existing = load_managed()?;
    let mut written = 0usize;
    for bind in binds {
        if let Some(found) = existing.iter_mut().find(|b| b.chord == bind.chord) {
            found.action = bind.action.clone();
            found.repeat = bind.repeat;
        } else {
            existing.push(bind.clone());
        }
        written += 1;
    }
    existing.sort_by(|a, b| a.chord.cmp(&b.chord));
    write_managed(&existing)?;
    Ok(written)
}

#[cfg(test)]
mod translate_tests {
    use super::*;

    #[test]
    fn translates_chords() {
        assert_eq!(niri_chord_to_umbriel("Mod+CTRL+UP"), "Mod+Ctrl+Up");
        assert_eq!(
            niri_chord_to_umbriel("Mod+Shift+ESCAPE"),
            "Mod+Shift+Escape"
        );
        assert_eq!(
            niri_chord_to_umbriel("Mod+WheelScrollDown"),
            "Mod+WheelDown"
        );
        assert_eq!(
            niri_chord_to_umbriel("XF86AudioRaiseVolume"),
            "XF86AudioRaiseVolume"
        );
    }

    #[test]
    fn translates_actions() {
        assert_eq!(
            translate_niri_action("close-window", &[]).as_deref(),
            Some("window-close")
        );
        assert_eq!(
            translate_niri_action("focus-workspace", &["3".into()]).as_deref(),
            Some("workspace-switch:3")
        );
        assert_eq!(
            translate_niri_action("set-column-width", &["-10%".into()]).as_deref(),
            Some("window-modify-primary-extent:-0.1")
        );
        assert_eq!(
            translate_niri_action("spawn", &["alacritty".into()]).as_deref(),
            Some("spawn:alacritty")
        );
        assert!(translate_niri_action(
            "spawn",
            &["sh".into(), "-c".into(), "grim -g \"$(slurp)\" -".into()]
        )
        .is_none());
    }
}
