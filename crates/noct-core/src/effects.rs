//! Presets visuales de Umbriel: efectos de borde, cursor, pantalla y ventana.
//!
//! Noct escribe un fragmento gestionado (`~/.config/umbriel/noct/effects.toml`)
//! con las selecciones y añade los `effect.toml` de Umbriel al include del
//! usuario. Al cambiar de preset se retiran solo las entradas de efectos, nunca
//! el resto de la configuración.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use toml_edit::{Array, DocumentMut, Item, Value};

use crate::env;

pub fn effects_file() -> PathBuf {
    env::app_config("umbriel").join("noct").join("effects.toml")
}

fn config_path() -> PathBuf {
    env::app_config("umbriel").join("config.toml")
}

fn effect_roots() -> Vec<PathBuf> {
    vec![
        PathBuf::from("/usr/local/share/umbriel/effects"),
        PathBuf::from("/usr/share/umbriel/effects"),
    ]
}

/// Efectos propios de Noct, escritos bajo la config del usuario.
fn custom_root() -> PathBuf {
    env::app_config("umbriel").join("noct").join("effects")
}

const CUSTOM_MONO_TOML: &str = include_str!("../assets/effects/screen/noct-mono/effect.toml");
const CUSTOM_MONO_SHADER: &str = include_str!("../assets/effects/screen/noct-mono/shader.glsl");

fn custom_asset(kind: &str, name: &str) -> Option<(&'static str, &'static str)> {
    match (kind, name) {
        ("screen", "noct-mono") => Some((CUSTOM_MONO_TOML, CUSTOM_MONO_SHADER)),
        _ => None,
    }
}

fn tint_path() -> PathBuf {
    custom_root().join("screen").join("noct-mono").join("tint")
}

fn intensity_path() -> PathBuf {
    custom_root()
        .join("screen")
        .join("noct-mono")
        .join("intensity")
}

/// Intensidad activa del preset Mono (0–100, por defecto 100).
pub fn current_intensity() -> u8 {
    std::fs::read_to_string(intensity_path())
        .ok()
        .and_then(|text| text.trim().parse::<u8>().ok())
        .map(|value| value.min(100))
        .unwrap_or(100)
}

/// Tinte activo del preset Mono (`None` = gris puro).
pub fn current_tint() -> Option<String> {
    let text = std::fs::read_to_string(tint_path()).ok()?;
    let value = text.trim().to_ascii_lowercase();
    if value.is_empty() || value == "none" || value == "gris" {
        None
    } else {
        Some(value)
    }
}

fn parse_hex(color: &str) -> Option<(u8, u8, u8)> {
    let hex = color.trim().trim_start_matches('#');
    let expanded = match hex.len() {
        3 => {
            let mut out = String::new();
            for ch in hex.chars() {
                out.push(ch);
                out.push(ch);
            }
            out
        }
        6 => hex.to_string(),
        _ => return None,
    };
    let r = u8::from_str_radix(&expanded[0..2], 16).ok()?;
    let g = u8::from_str_radix(&expanded[2..4], 16).ok()?;
    let b = u8::from_str_radix(&expanded[4..6], 16).ok()?;
    Some((r, g, b))
}

fn normalize_tint(color: &str) -> Result<Option<String>> {
    let value = color.trim().to_ascii_lowercase();
    if value.is_empty() || value == "none" || value == "off" || value == "gris" || value == "gray" {
        return Ok(None);
    }
    let (r, g, b) = parse_hex(&value)
        .with_context(|| format!("color inválido: {color} (usa #RRGGBB o none)"))?;
    Ok(Some(format!("#{r:02x}{g:02x}{b:02x}")))
}

/// Slots de efecto que se pueden mezclar sobre un preset.
pub const EFFECT_SLOTS: &[&str] = &["border", "cursor", "screen", "window"];

/// Slots de animación que se pueden mezclar sobre un preset.
pub const ANIMATION_SLOTS: &[&str] = &["windows_in", "windows_move", "windows_out", "workspaces"];

fn overrides_path() -> PathBuf {
    env::app_config("umbriel")
        .join("noct")
        .join("overrides.toml")
}

/// Overrides por slot: cada clave sustituye el valor del preset.
/// El valor `none` desactiva ese slot; si falta, se usa el preset.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Overrides {
    #[serde(flatten)]
    pub values: BTreeMap<String, String>,
}

impl Overrides {
    fn get(&self, slot: &str) -> Option<&str> {
        self.values.get(slot).map(String::as_str)
    }

    fn set(&mut self, slot: &str, value: &str) {
        self.values.insert(slot.to_string(), value.to_string());
    }

    fn clear(&mut self, slot: &str) {
        self.values.remove(slot);
    }
}

/// Carga los overrides; un archivo ausente o inválido equivale a ninguno.
pub fn load_overrides() -> Overrides {
    let path = overrides_path();
    let Ok(text) = std::fs::read_to_string(&path) else {
        return Overrides::default();
    };
    let mut overrides: Overrides = toml::from_str(&text).unwrap_or_default();
    overrides.values.retain(|slot, value| {
        if slot_kind(slot).is_none() {
            return false;
        }
        let normalized = normalize_override_value(value);
        if normalized.is_empty() {
            return false;
        }
        *value = normalized;
        true
    });
    overrides
}

fn save_overrides(overrides: &Overrides) -> Result<()> {
    let path = overrides_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("no se pudo crear {}", parent.display()))?;
    }
    let text = toml::to_string_pretty(overrides)
        .with_context(|| format!("no se pudo serializar {}", path.display()))?;
    std::fs::write(&path, text).with_context(|| format!("no se pudo escribir {}", path.display()))
}

fn normalize_override_value(value: &str) -> String {
    let value = value.trim().to_ascii_lowercase();
    match value.as_str() {
        "" | "auto" | "preset" => String::new(),
        "off" | "none" | "disabled" | "false" => "none".to_string(),
        _ => value,
    }
}

fn slot_kind(slot: &str) -> Option<&'static str> {
    match slot {
        "border" => Some("border"),
        "cursor" => Some("cursor"),
        "screen" => Some("screen"),
        "window" => Some("window"),
        "windows_in" | "windows_move" | "windows_out" | "workspaces" => Some("animation"),
        _ => None,
    }
}

fn all_slots() -> Vec<&'static str> {
    EFFECT_SLOTS
        .iter()
        .chain(ANIMATION_SLOTS.iter())
        .copied()
        .collect()
}

fn installed_names(kind: &str) -> Vec<String> {
    let prefix = format!("{kind}/");
    installed_effects()
        .into_iter()
        .filter_map(|entry| entry.strip_prefix(&prefix).map(str::to_string))
        .collect()
}

fn render_mono_shader(tint: Option<&str>, intensity: u8) -> String {
    let (r, g, b, amount) = match tint.and_then(parse_hex) {
        Some((r, g, b)) => {
            let max = r.max(g).max(b).max(1) as f64;
            ((r as f64 / max), (g as f64 / max), (b as f64 / max), 0.82)
        }
        None => (1.0, 1.0, 1.0, 0.0),
    };
    CUSTOM_MONO_SHADER
        .replace("__R__", &format!("{r:.4}"))
        .replace("__G__", &format!("{g:.4}"))
        .replace("__B__", &format!("{b:.4}"))
        .replace("__MIX__", &format!("{amount:.2}"))
        .replace(
            "__INTENSITY__",
            &format!("{:.2}", intensity.min(100) as f64 / 100.0),
        )
}

fn write_if_changed(path: &Path, content: &str) -> Result<()> {
    if std::fs::read_to_string(path).ok().as_deref() == Some(content) {
        return Ok(());
    }
    std::fs::write(path, content).with_context(|| format!("no se pudo escribir {}", path.display()))
}

/// Escribe los shaders propios de Noct antes de aplicarlos.
fn ensure_custom_assets() -> Result<()> {
    for (kind, name) in [("screen", "noct-mono")] {
        let Some((effect, shader)) = custom_asset(kind, name) else {
            continue;
        };
        let dir = custom_root().join(kind).join(name);
        std::fs::create_dir_all(&dir)
            .with_context(|| format!("no se pudo crear {}", dir.display()))?;
        write_if_changed(&dir.join("effect.toml"), effect)?;
        let tint = current_tint();
        let intensity = current_intensity();
        write_if_changed(
            &dir.join("shader.glsl"),
            &render_mono_shader(tint.as_deref(), intensity),
        )?;
        let _ = shader;
    }
    Ok(())
}

/// Busca el `effect.toml` instalado para `<kind>/<name>`.
fn find_effect(kind: &str, name: &str) -> Option<PathBuf> {
    if custom_asset(kind, name).is_some() {
        return Some(custom_root().join(kind).join(name).join("effect.toml"));
    }
    for root in effect_roots() {
        let candidate = root.join(kind).join(name).join("effect.toml");
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

/// Lista los efectos instalados como `kind/name`.
pub fn installed_effects() -> Vec<String> {
    let mut out = Vec::new();
    for root in effect_roots() {
        let Ok(kinds) = std::fs::read_dir(&root) else {
            continue;
        };
        for kind in kinds.flatten() {
            if !kind.path().is_dir() {
                continue;
            }
            let kind_name = kind.file_name().to_string_lossy().to_string();
            let Ok(names) = std::fs::read_dir(kind.path()) else {
                continue;
            };
            for name in names.flatten() {
                if name.path().join("effect.toml").is_file() {
                    let entry = format!("{kind_name}/{}", name.file_name().to_string_lossy());
                    if !out.contains(&entry) {
                        out.push(entry);
                    }
                }
            }
        }
    }
    if custom_asset("screen", "noct-mono").is_some() {
        out.push("screen/noct-mono".to_string());
    }
    out.sort();
    out.dedup();
    out
}

#[derive(Debug, Clone, Serialize)]
pub struct PresetInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub effects: Vec<String>,
    pub animations: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub missing: Vec<String>,
}

struct Preset {
    id: &'static str,
    name: &'static str,
    description: &'static str,
    border: Option<&'static str>,
    cursor: Option<&'static str>,
    screen: Option<&'static str>,
    window: Option<&'static str>,
    animations: &'static [(&'static str, &'static str)],
}

const PRESETS: &[Preset] = &[
    Preset {
        id: "neon",
        name: "Noct Neon",
        description: "Borde pulsante, cursor con glow, viñeta y animaciones con rebote.",
        border: Some("pulse"),
        cursor: Some("glow"),
        screen: Some("vignette"),
        window: None,
        animations: &[("windows_move", "squash"), ("workspaces", "workspace_wipe")],
    },
    Preset {
        id: "cinema",
        name: "Noct Cinema",
        description: "Viñeta de pantalla, scanlines por ventana y estela de cursor.",
        border: None,
        cursor: Some("trail"),
        screen: Some("vignette"),
        window: Some("scanlines"),
        animations: &[("windows_move", "squash")],
    },
    Preset {
        id: "matrix",
        name: "Noct Matrix",
        description: "Scanlines, borde pulsante, estela curva y barrido de workspaces.",
        border: Some("pulse"),
        cursor: Some("trail-path"),
        screen: Some("vignette"),
        window: Some("scanlines"),
        animations: &[("workspaces", "workspace_wipe")],
    },
    Preset {
        id: "minimal",
        name: "Noct Minimal",
        description: "Solo un borde vivo alrededor de la ventana enfocada.",
        border: Some("pulse"),
        cursor: None,
        screen: None,
        window: None,
        animations: &[],
    },
    Preset {
        id: "mono",
        name: "Noct Mono",
        description: "Todo el escritorio en blanco y negro o duotono, con scanlines y viñeta.",
        border: None,
        cursor: None,
        screen: Some("noct-mono"),
        window: None,
        animations: &[("workspaces", "workspace_wipe")],
    },
    Preset {
        id: "custom",
        name: "Noct Libre",
        description: "Empieza sin efectos y mezcla solo los que elijas.",
        border: None,
        cursor: None,
        screen: None,
        window: None,
        animations: &[],
    },
    Preset {
        id: "off",
        name: "Sin efectos",
        description: "Desactiva los efectos y retira sus includes.",
        border: None,
        cursor: None,
        screen: None,
        window: None,
        animations: &[],
    },
];

fn preset(id: &str) -> Option<&'static Preset> {
    PRESETS.iter().find(|preset| preset.id == id)
}

fn preset_effects(preset: &Preset) -> Vec<(&'static str, &'static str)> {
    let mut out = Vec::new();
    if let Some(name) = preset.border {
        out.push(("border", name));
    }
    if let Some(name) = preset.cursor {
        out.push(("cursor", name));
    }
    if let Some(name) = preset.screen {
        out.push(("screen", name));
    }
    if let Some(name) = preset.window {
        out.push(("window", name));
    }
    out
}

/// Aplica los overrides de slot sobre los efectos del preset.
fn effective_effects<'a>(preset: &'a Preset, overrides: &'a Overrides) -> Vec<(&'a str, &'a str)> {
    let mut out: Vec<(&str, &str)> = Vec::new();
    for (kind, name) in preset_effects(preset) {
        let value = overrides.get(kind).unwrap_or(name);
        if value != "none" {
            out.push((kind, value));
        }
    }
    // Un override puede añadir un efecto que el preset no tenía (p. ej. cursor en minimal).
    for slot in EFFECT_SLOTS {
        if out.iter().any(|(kind, _)| kind == slot) {
            continue;
        }
        if let Some(value) = overrides.get(slot) {
            if value != "none" {
                out.push((slot, value));
            }
        }
    }
    out
}

/// Aplica los overrides de animación sobre las animaciones del preset.
fn effective_animations<'a>(
    preset: &'a Preset,
    overrides: &'a Overrides,
) -> Vec<(&'a str, &'a str)> {
    let mut out: Vec<(&str, &str)> = Vec::new();
    for (group, name) in preset.animations {
        let value = overrides.get(group).unwrap_or(name);
        if value != "none" {
            out.push((group, value));
        }
    }
    for slot in ANIMATION_SLOTS {
        if out.iter().any(|(group, _)| group == slot) {
            continue;
        }
        if let Some(value) = overrides.get(slot) {
            if value != "none" {
                out.push((slot, value));
            }
        }
    }
    out
}

/// Presets curados con el estado de sus archivos de efecto.
pub fn presets() -> Vec<PresetInfo> {
    PRESETS
        .iter()
        .map(|preset| {
            let mut missing = Vec::new();
            for (kind, name) in preset_effects(preset) {
                if find_effect(kind, name).is_none() {
                    missing.push(format!("{kind}/{name}"));
                }
            }
            for (_, name) in preset.animations {
                if find_effect("animation", name).is_none() {
                    missing.push(format!("animation/{name}"));
                }
            }
            PresetInfo {
                id: preset.id.to_string(),
                name: preset.name.to_string(),
                description: preset.description.to_string(),
                effects: preset_effects(preset)
                    .into_iter()
                    .map(|(kind, name)| format!("{kind}={name}"))
                    .collect(),
                animations: preset
                    .animations
                    .iter()
                    .map(|(group, name)| format!("{group}={name}"))
                    .collect(),
                missing,
            }
        })
        .collect()
}

fn render_effects(
    preset_id: &str,
    effects: &[(&str, &str)],
    animations: &[(&str, &str)],
) -> String {
    let mut out = String::new();
    out.push_str("# Generado por Noct (noct effects). No editar a mano.\n");
    out.push_str(&format!("# preset: {preset_id}\n\n"));
    if !effects.is_empty() {
        out.push_str("[effects]\n");
        // Las capturas y grabaciones deben mostrar el mismo look que la pantalla.
        out.push_str("in_capture = true\n");
        for (kind, name) in effects {
            out.push_str(&format!("{kind} = \"{name}\"\n"));
        }
        out.push('\n');
    }
    for (group, name) in animations {
        out.push_str(&format!("[animation.{group}]\n"));
        out.push_str(&format!("effect = \"{name}\"\n\n"));
    }
    out
}

fn effect_includes(effects: &[(&str, &str)], animations: &[(&str, &str)]) -> Result<Vec<String>> {
    let mut out = Vec::new();
    for (kind, name) in effects {
        let path = find_effect(kind, name).with_context(|| {
            format!(
                "no está instalado el efecto {kind}/{name} (mira `noct effects list` o limpia la mezcla con `noct effects mix clear`)"
            )
        })?;
        out.push(path.display().to_string());
    }
    for (_, name) in animations {
        let path = find_effect("animation", name).with_context(|| {
            format!(
                "no está instalada la animación {name} (mira `noct effects list` o limpia la mezcla con `noct effects mix clear`)"
            )
        })?;
        out.push(path.display().to_string());
    }
    Ok(out)
}

fn is_effect_include(value: &str) -> bool {
    value.ends_with("/effect.toml")
        && (value.contains("/umbriel/effects/") || value.contains("/umbriel/noct/effects/"))
}

/// Reescribe el include de Umbriel: quita efectos antiguos y añade los nuevos.
fn update_includes(add: &[String]) -> Result<bool> {
    let path = config_path();
    if !path.exists() {
        bail!("no se encontró {}", path.display());
    }
    let text = std::fs::read_to_string(&path)
        .with_context(|| format!("no se pudo leer {}", path.display()))?;
    let mut doc: DocumentMut = text
        .parse()
        .with_context(|| format!("{} no es TOML válido", path.display()))?;

    let include = doc
        .entry("include")
        .or_insert(Item::Table(toml_edit::Table::new()));
    let optional = include
        .as_table_like_mut()
        .context("include no es una tabla")?
        .entry("optional")
        .or_insert(Item::Table(toml_edit::Table::new()));
    let files = optional
        .as_table_like_mut()
        .context("include.optional no es una tabla")?
        .entry("files")
        .or_insert(Item::Value(Value::Array(Array::new())));
    if !files.is_array() {
        *files = Item::Value(Value::Array(Array::new()));
    }
    let array = files.as_array_mut().context("files no es un array")?;

    let before: Vec<String> = array
        .iter()
        .filter_map(|value| value.as_str().map(str::to_string))
        .collect();

    // Conserva lo que no sea un efecto y el fragmento propio de Noct.
    let mut kept: Vec<String> = before
        .iter()
        .filter(|value| !is_effect_include(value))
        .cloned()
        .collect();
    if !kept.iter().any(|value| value == "noct/effects.toml") {
        kept.push("noct/effects.toml".to_string());
    }

    // Los efectos definidos deben incluirse antes de la selección.
    let insertion = kept
        .iter()
        .position(|value| value == "noct/effects.toml")
        .unwrap_or(kept.len());
    for entry in add {
        if !kept.contains(entry) {
            kept.insert(insertion, entry.clone());
        }
    }

    if kept == before {
        return Ok(false);
    }

    array.clear();
    for value in &kept {
        array.push(value.as_str());
    }
    std::fs::write(&path, doc.to_string())
        .with_context(|| format!("no se pudo escribir {}", path.display()))?;
    Ok(true)
}

#[derive(Debug, Clone, Serialize)]
pub struct ApplyReport {
    pub preset: String,
    pub name: String,
    pub file: String,
    pub effects: Vec<String>,
    pub animations: Vec<String>,
    pub includes: Vec<String>,
    pub written: bool,
    pub validated: bool,
    pub reloaded: bool,
    pub message: String,
}

fn validate_umbriel() -> Result<()> {
    let _ = crate::config::fix_all();
    let Some(out) = env::run("umbriel", &["config", "validate"]) else {
        return Ok(());
    };
    if out.success {
        Ok(())
    } else {
        let detail = if out.stderr.is_empty() {
            out.stdout
        } else {
            out.stderr
        };
        bail!("umbriel config validate falló:\n{detail}");
    }
}

/// Aplica un preset con los overrides activos. Con `dry_run` solo devuelve el fragmento.
pub fn apply(id: &str, dry_run: bool) -> Result<(ApplyReport, String)> {
    let overrides = load_overrides();
    apply_with_overrides(id, &overrides, dry_run)
}

fn apply_with_overrides(
    id: &str,
    overrides: &Overrides,
    dry_run: bool,
) -> Result<(ApplyReport, String)> {
    let Some(preset) = preset(id) else {
        let ids: Vec<&str> = PRESETS.iter().map(|p| p.id).collect();
        bail!("preset desconocido: {id}. Disponibles: {}", ids.join(", "));
    };

    // "off" apaga todo de verdad: no se mezcla con overrides.
    let empty = Overrides::default();
    let active = if preset.id == "off" {
        &empty
    } else {
        overrides
    };
    let effects = effective_effects(preset, active);
    let animations = effective_animations(preset, active);
    let includes = effect_includes(&effects, &animations)?;
    let content = render_effects(preset.id, &effects, &animations);
    let file = effects_file();

    let mut report = ApplyReport {
        preset: preset.id.to_string(),
        name: preset.name.to_string(),
        file: file.display().to_string(),
        effects: effects
            .iter()
            .map(|(kind, name)| format!("{kind}={name}"))
            .collect(),
        animations: animations
            .iter()
            .map(|(group, name)| format!("{group}={name}"))
            .collect(),
        includes: includes.clone(),
        written: false,
        validated: false,
        reloaded: false,
        message: String::new(),
    };

    if dry_run {
        report.message = "dry-run: no se escribió nada".into();
        return Ok((report, content));
    }

    ensure_custom_assets()?;

    let previous_effects = std::fs::read_to_string(&file).ok();
    let config = config_path();
    let previous_config = std::fs::read_to_string(&config).ok();

    if let Some(parent) = file.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("no se pudo crear {}", parent.display()))?;
    }
    std::fs::write(&file, &content)
        .with_context(|| format!("no se pudo escribir {}", file.display()))?;
    report.written = true;

    if let Err(err) = update_includes(&includes) {
        restore(&file, previous_effects);
        return Err(err);
    }

    if let Err(err) = validate_umbriel() {
        restore(&file, previous_effects);
        restore(&config, previous_config);
        return Err(err);
    }
    report.validated = true;

    let reloaded = env::run("umbriel", &["msg", "config-reload"])
        .map(|out| out.success)
        .unwrap_or(false);
    report.reloaded = reloaded;
    report.message = if preset.id == "off" {
        "efectos desactivados".to_string()
    } else if preset.id == "custom" && effects.is_empty() && animations.is_empty() {
        "sin efectos: elige qué mezclar".to_string()
    } else if reloaded {
        "preset aplicado y Umbriel recargado".to_string()
    } else {
        "preset escrito (Umbriel no está en ejecución; se aplicará al arrancar)".to_string()
    };

    Ok((report, content))
}

fn restore(path: &Path, previous: Option<String>) {
    match previous {
        Some(text) => {
            let _ = std::fs::write(path, text);
        }
        None => {
            let _ = std::fs::remove_file(path);
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct MixReport {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slot: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    pub preset: String,
    pub overrides: BTreeMap<String, String>,
    pub effects: Vec<String>,
    pub animations: Vec<String>,
    pub file: String,
    pub written: bool,
    pub validated: bool,
    pub reloaded: bool,
    pub message: String,
}

/// Preset base para una mezcla: el activo, o `custom` si estamos en off/nada.
fn base_preset_for_mix() -> String {
    match current().ok().and_then(|value| value.preset) {
        Some(id) if id != "off" => id,
        _ => "custom".to_string(),
    }
}

fn check_override_value(slot: &str, value: &str) -> Result<()> {
    if value == "none" {
        return Ok(());
    }
    let kind = slot_kind(slot)
        .with_context(|| format!("slot desconocido: {slot}. Usa: {}", all_slots().join(", ")))?;
    let names = installed_names(kind);
    if names.iter().any(|name| name == value) {
        return Ok(());
    }
    let available = if names.is_empty() {
        "ninguno".to_string()
    } else {
        names.join(", ")
    };
    bail!("no está instalado {kind}/{value}. Disponibles: {available}")
}

/// Cambia un slot de la mezcla y reaplica el preset activo.
pub fn set_override(slot: &str, value: &str, dry_run: bool) -> Result<(MixReport, String)> {
    let slot = slot.trim().to_ascii_lowercase();
    let normalized = normalize_override_value(value);
    if normalized.is_empty() {
        return clear_override(Some(&slot), dry_run);
    }
    check_override_value(&slot, &normalized)?;

    let mut overrides = load_overrides();
    overrides.set(&slot, &normalized);
    let preset_id = base_preset_for_mix();
    let previous = std::fs::read_to_string(overrides_path()).ok();

    if !dry_run {
        save_overrides(&overrides)?;
    }

    let result = apply_with_overrides(&preset_id, &overrides, dry_run);
    let (apply_report, content) = match result {
        Ok(value) => value,
        Err(err) => {
            if !dry_run {
                restore(&overrides_path(), previous);
            }
            return Err(err);
        }
    };

    Ok((
        MixReport {
            slot: Some(slot),
            value: Some(normalized),
            preset: apply_report.preset,
            overrides: overrides.values,
            effects: apply_report.effects,
            animations: apply_report.animations,
            file: apply_report.file,
            written: apply_report.written,
            validated: apply_report.validated,
            reloaded: apply_report.reloaded,
            message: apply_report.message,
        },
        content,
    ))
}

/// Quita un override (o todos) y reaplica el preset activo.
pub fn clear_override(slot: Option<&str>, dry_run: bool) -> Result<(MixReport, String)> {
    let mut overrides = load_overrides();
    let slot = slot.map(|value| value.trim().to_ascii_lowercase());
    if let Some(slot) = &slot {
        if slot_kind(slot).is_none() {
            bail!("slot desconocido: {slot}. Usa: {}", all_slots().join(", "));
        }
        overrides.clear(slot);
    } else {
        overrides.values.clear();
    }

    let mut preset_id = base_preset_for_mix();
    // Si la mezcla queda vacía y estábamos en custom, apaga del todo.
    if overrides.values.is_empty() && preset_id == "custom" {
        preset_id = "off".to_string();
    }

    let previous = std::fs::read_to_string(overrides_path()).ok();
    if !dry_run {
        save_overrides(&overrides)?;
    }

    let result = apply_with_overrides(&preset_id, &overrides, dry_run);
    let (apply_report, content) = match result {
        Ok(value) => value,
        Err(err) => {
            if !dry_run {
                restore(&overrides_path(), previous);
            }
            return Err(err);
        }
    };

    Ok((
        MixReport {
            slot,
            value: None,
            preset: apply_report.preset,
            overrides: overrides.values,
            effects: apply_report.effects,
            animations: apply_report.animations,
            file: apply_report.file,
            written: apply_report.written,
            validated: apply_report.validated,
            reloaded: apply_report.reloaded,
            message: apply_report.message,
        },
        content,
    ))
}

fn animation_choices(slot: &str, installed: &[String]) -> Vec<String> {
    let wanted: &[&str] = match slot {
        "windows_in" | "windows_out" => &["reveal"],
        "windows_move" => &["squash"],
        "workspaces" => &["workspace_wipe"],
        _ => return installed.to_vec(),
    };
    installed
        .iter()
        .filter(|name| wanted.contains(&name.as_str()))
        .cloned()
        .collect()
}

#[derive(Debug, Clone, Serialize)]
pub struct MixShow {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preset: Option<String>,
    pub overrides: BTreeMap<String, String>,
    pub effective: BTreeMap<String, Option<String>>,
    pub choices: BTreeMap<String, Vec<String>>,
}

/// Estado de la mezcla: overrides, valores efectivos y opciones instaladas.
pub fn mix_show() -> Result<MixShow> {
    let current = current()?;
    let mut effective = BTreeMap::new();
    for slot in all_slots() {
        effective.insert(slot.to_string(), None);
    }
    for entry in &current.effects {
        if let Some((kind, name)) = entry.split_once('=') {
            effective.insert(kind.to_string(), Some(name.to_string()));
        }
    }
    for entry in &current.animations {
        if let Some((group, name)) = entry.split_once('=') {
            effective.insert(group.to_string(), Some(name.to_string()));
        }
    }

    let mut choices = BTreeMap::new();
    for slot in EFFECT_SLOTS {
        choices.insert(slot.to_string(), installed_names(slot));
    }
    let animations = installed_names("animation");
    for slot in ANIMATION_SLOTS {
        choices.insert(slot.to_string(), animation_choices(slot, &animations));
    }

    Ok(MixShow {
        preset: current.preset,
        overrides: current.overrides,
        effective,
        choices,
    })
}

#[derive(Debug, Clone, Serialize)]
pub struct TintReport {
    /// Tinte normalizado, o `None` para gris puro.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tint: Option<String>,
    pub preset: String,
    pub file: String,
    pub written: bool,
    pub validated: bool,
    pub reloaded: bool,
    pub message: String,
}

/// Aplica un ajuste de Mono. Si el slot `screen` no es `noct-mono`, lo activa
/// con un override para que el tinte/intensidad se vean de verdad.
fn apply_mono_adjustment(preset_id: &str) -> Result<(ApplyReport, bool)> {
    let Some(preset) = preset(preset_id) else {
        bail!("preset desconocido: {preset_id}");
    };
    let mut overrides = load_overrides();
    let previous = std::fs::read_to_string(overrides_path()).ok();
    let has_mono_screen = effective_effects(preset, &overrides)
        .iter()
        .any(|(kind, name)| *kind == "screen" && *name == "noct-mono");
    let forced = !has_mono_screen;
    if forced {
        overrides.set("screen", "noct-mono");
        if let Err(err) = save_overrides(&overrides) {
            restore(&overrides_path(), previous);
            return Err(err);
        }
    }

    match apply_with_overrides(preset_id, &overrides, false) {
        Ok((report, _)) => Ok((report, forced)),
        Err(err) => {
            if forced {
                restore(&overrides_path(), previous);
            }
            Err(err)
        }
    }
}

/// Cambia el tinte del preset Mono y lo deja aplicado.
pub fn set_tint(color: &str, dry_run: bool) -> Result<(TintReport, String)> {
    let normalized = normalize_tint(color)?;
    let shader = render_mono_shader(normalized.as_deref(), current_intensity());
    let file = effects_file();

    if dry_run {
        return Ok((
            TintReport {
                tint: normalized,
                preset: "mono".into(),
                file: file.display().to_string(),
                written: false,
                validated: false,
                reloaded: false,
                message: "dry-run: no se escribió nada".into(),
            },
            shader,
        ));
    }

    let path = tint_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("no se pudo crear {}", parent.display()))?;
    }
    let value = normalized.clone().unwrap_or_else(|| "none".to_string());
    std::fs::write(&path, value)
        .with_context(|| format!("no se pudo escribir {}", path.display()))?;

    let preset_id = base_preset_for_mix();
    let (apply_report, forced) = apply_mono_adjustment(&preset_id)?;
    let message = if forced {
        format!("Mono activado. {}", apply_report.message)
    } else {
        apply_report.message
    };
    Ok((
        TintReport {
            tint: normalized,
            preset: apply_report.preset,
            file: apply_report.file,
            written: true,
            validated: apply_report.validated,
            reloaded: apply_report.reloaded,
            message,
        },
        shader,
    ))
}

#[derive(Debug, Clone, Serialize)]
pub struct IntensityReport {
    pub intensity: u8,
    pub preset: String,
    pub file: String,
    pub written: bool,
    pub validated: bool,
    pub reloaded: bool,
    pub message: String,
}

/// Cambia la intensidad del preset Mono (0–100) y lo deja aplicado.
pub fn set_intensity(value: u8, dry_run: bool) -> Result<(IntensityReport, String)> {
    let value = value.min(100);
    let shader = render_mono_shader(current_tint().as_deref(), value);
    let file = effects_file();

    if dry_run {
        return Ok((
            IntensityReport {
                intensity: value,
                preset: "mono".into(),
                file: file.display().to_string(),
                written: false,
                validated: false,
                reloaded: false,
                message: "dry-run: no se escribió nada".into(),
            },
            shader,
        ));
    }

    let path = intensity_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("no se pudo crear {}", parent.display()))?;
    }
    std::fs::write(&path, value.to_string())
        .with_context(|| format!("no se pudo escribir {}", path.display()))?;

    let preset_id = base_preset_for_mix();
    let (apply_report, forced) = apply_mono_adjustment(&preset_id)?;
    let message = if forced {
        format!("Mono activado. {}", apply_report.message)
    } else {
        apply_report.message
    };
    Ok((
        IntensityReport {
            intensity: value,
            preset: apply_report.preset,
            file: apply_report.file,
            written: true,
            validated: apply_report.validated,
            reloaded: apply_report.reloaded,
            message,
        },
        shader,
    ))
}

#[derive(Debug, Clone, Serialize)]
pub struct CurrentEffects {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preset: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tint: Option<String>,
    pub intensity: u8,
    pub effects: Vec<String>,
    pub animations: Vec<String>,
    pub overrides: BTreeMap<String, String>,
}

/// Lee las selecciones activas del fragmento gestionado.
pub fn current() -> Result<CurrentEffects> {
    let path = effects_file();
    if !path.exists() {
        return Ok(CurrentEffects {
            preset: None,
            tint: current_tint(),
            intensity: current_intensity(),
            effects: Vec::new(),
            animations: Vec::new(),
            overrides: load_overrides().values,
        });
    }
    let text = std::fs::read_to_string(&path)
        .with_context(|| format!("no se pudo leer {}", path.display()))?;
    let value: toml::Value =
        toml::from_str(&text).with_context(|| format!("{} no es TOML válido", path.display()))?;

    let preset = text
        .lines()
        .find_map(|line| line.strip_prefix("# preset: "))
        .map(|value| value.trim().to_string());

    let mut effects = Vec::new();
    if let Some(table) = value.get("effects").and_then(|v| v.as_table()) {
        for (kind, name) in table {
            if let Some(name) = name.as_str() {
                effects.push(format!("{kind}={name}"));
            }
        }
    }
    effects.sort();

    let mut animations = Vec::new();
    if let Some(table) = value.get("animation").and_then(|v| v.as_table()) {
        for (group, body) in table {
            if let Some(name) = body.get("effect").and_then(|v| v.as_str()) {
                animations.push(format!("{group}={name}"));
            }
        }
    }
    animations.sort();

    Ok(CurrentEffects {
        preset,
        tint: current_tint(),
        intensity: current_intensity(),
        effects,
        animations,
        overrides: load_overrides().values,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_effects_and_animations() {
        let preset = preset("neon").unwrap();
        let effects = preset_effects(preset);
        let text = render_effects("neon", &effects, preset.animations);
        let value: toml::Value = toml::from_str(&text).unwrap();
        assert_eq!(value["effects"]["border"].as_str(), Some("pulse"));
        assert_eq!(value["effects"]["cursor"].as_str(), Some("glow"));
        assert_eq!(
            value["animation"]["windows_move"]["effect"].as_str(),
            Some("squash")
        );
    }

    #[test]
    fn off_renders_empty_document() {
        let text = render_effects("off", &[], &[]);
        let value: toml::Value = toml::from_str(&text).unwrap();
        assert!(value.get("effects").is_none());
    }

    #[test]
    fn overrides_can_disable_replace_and_add_slots() {
        let preset = preset("matrix").unwrap();
        let mut overrides = Overrides::default();
        overrides.set("cursor", "none");
        overrides.set("screen", "noct-mono");
        overrides.set("windows_move", "none");

        let effects = effective_effects(preset, &overrides);
        assert!(!effects.iter().any(|(kind, _)| *kind == "cursor"));
        assert!(effects
            .iter()
            .any(|(kind, name)| *kind == "screen" && *name == "noct-mono"));

        let animations = effective_animations(preset, &overrides);
        assert!(!animations.iter().any(|(group, _)| *group == "windows_move"));
    }

    #[test]
    fn overrides_can_add_cursor_to_minimal() {
        let preset = preset("minimal").unwrap();
        let mut overrides = Overrides::default();
        overrides.set("cursor", "trail");

        let effects = effective_effects(preset, &overrides);
        assert!(effects
            .iter()
            .any(|(kind, name)| *kind == "cursor" && *name == "trail"));
        assert!(effects.iter().any(|(kind, _)| *kind == "border"));
    }

    #[test]
    fn known_presets_have_effects() {
        assert_eq!(presets().len(), 7);
        assert!(presets().iter().any(|p| p.id == "matrix"));
        assert!(presets().iter().any(|p| p.id == "mono"));
        assert!(presets().iter().any(|p| p.id == "custom"));
    }
}

// ---------------------------------------------------------------------------
// Widget de Noctalia
// ---------------------------------------------------------------------------

const PLUGIN_TOML: &str = include_str!("../assets/noctalia-plugin/plugin.toml");
const PLUGIN_SHORTCUT: &str = include_str!("../assets/noctalia-plugin/shortcut.luau");
const PLUGIN_PANEL: &str = include_str!("../assets/noctalia-plugin/panel.luau");
const PLUGIN_MIX: &str = include_str!("../assets/noctalia-plugin/mix.luau");
const PLUGIN_I18N_EN: &str = include_str!("../assets/noctalia-plugin/translations/en.json");
const PLUGIN_I18N_ES: &str = include_str!("../assets/noctalia-plugin/translations/es.json");

#[derive(Debug, Clone, Serialize)]
pub struct WidgetReport {
    pub plugin_dir: String,
    pub files: Vec<String>,
    pub enabled: bool,
    pub state_file: String,
    pub settings_file: String,
    pub shortcut_added: bool,
    pub noct_in_path: bool,
}

fn inline_shortcut(kind: &str) -> toml_edit::Value {
    let mut table = toml_edit::InlineTable::new();
    table.insert("type", kind.into());
    toml_edit::Value::InlineTable(table)
}

/// Añade el tile al control center si aún no está. Mantiene como máximo 6.
fn ensure_control_center_shortcut() -> Result<(bool, String)> {
    let path = env::state_home().join("noctalia").join("settings.toml");
    if !path.exists() {
        return Ok((false, path.display().to_string()));
    }
    let text = std::fs::read_to_string(&path)
        .with_context(|| format!("no se pudo leer {}", path.display()))?;
    let mut doc: DocumentMut = text
        .parse()
        .with_context(|| format!("{} no es TOML válido", path.display()))?;

    let control = doc
        .entry("control_center")
        .or_insert(Item::Table(toml_edit::Table::new()));
    let control = control
        .as_table_like_mut()
        .context("control_center no es una tabla")?;
    let entry = control.entry("shortcuts").or_insert_with(|| {
        let mut array = Array::new();
        for kind in [
            "wifi",
            "bluetooth",
            "caffeine",
            "nightlight",
            "notification",
            "power_profile",
        ] {
            array.push(inline_shortcut(kind));
        }
        Item::Value(Value::Array(array))
    });
    if !entry.is_array() {
        return Ok((false, path.display().to_string()));
    }
    let array = entry.as_array_mut().context("shortcuts no es un array")?;

    let target = "noct/effects:effects";
    let has_target = array.iter().any(|value| {
        value
            .as_inline_table()
            .and_then(|table| table.get("type"))
            .and_then(|value| value.as_str())
            == Some(target)
    });
    if has_target {
        return Ok((false, path.display().to_string()));
    }

    // El Home muestra hasta 6 atajos. Si ya hay 6, libera un hueco quitando
    // caffeine o, en su defecto, power_profile.
    if array.len() >= 6 {
        let candidates = ["caffeine", "power_profile"];
        let mut removed = false;
        for candidate in candidates {
            let index = array.iter().position(|value| {
                value
                    .as_inline_table()
                    .and_then(|table| table.get("type"))
                    .and_then(|value| value.as_str())
                    == Some(candidate)
            });
            if let Some(index) = index {
                array.remove(index);
                removed = true;
                break;
            }
        }
        if !removed && array.len() >= 6 {
            array.remove(array.len() - 1);
        }
    }
    array.insert(0, inline_shortcut(target));

    let previous = std::fs::read_to_string(&path).ok();
    if let Some(previous) = &previous {
        let backup = path.with_extension("toml.noct.bak");
        let _ = std::fs::write(&backup, previous);
    }
    std::fs::write(&path, doc.to_string())
        .with_context(|| format!("no se pudo escribir {}", path.display()))?;
    Ok((true, path.display().to_string()))
}

/// Instala o actualiza el widget de Noctalia y lo deja habilitado.
pub fn install_widget() -> Result<WidgetReport> {
    let dir = env::data_home()
        .join("noctalia")
        .join("plugins")
        .join("effects");
    std::fs::create_dir_all(&dir).with_context(|| format!("no se pudo crear {}", dir.display()))?;

    let files = [
        ("plugin.toml", PLUGIN_TOML),
        ("shortcut.luau", PLUGIN_SHORTCUT),
        ("panel.luau", PLUGIN_PANEL),
        ("mix.luau", PLUGIN_MIX),
        ("translations/en.json", PLUGIN_I18N_EN),
        ("translations/es.json", PLUGIN_I18N_ES),
    ];
    let mut written = Vec::new();
    for (name, content) in files {
        let path = dir.join(name);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("no se pudo crear {}", parent.display()))?;
        }
        write_if_changed(&path, content)?;
        written.push(path.display().to_string());
    }

    let state_file = env::config_home().join("noctalia").join("plugins.json");
    let mut enabled = false;
    if let Some(parent) = state_file.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("no se pudo crear {}", parent.display()))?;
    }
    let mut root: serde_json::Value = std::fs::read_to_string(&state_file)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_else(|| serde_json::json!({"sources": [], "states": {}, "version": 2}));

    let states = root
        .as_object_mut()
        .context("plugins.json no es un objeto")?
        .entry("states")
        .or_insert_with(|| serde_json::json!({}));
    let states = states
        .as_object_mut()
        .context("plugins.json: states no es un objeto")?;
    let entry = states
        .entry("noct/effects")
        .or_insert_with(|| serde_json::json!({}));
    if let Some(entry) = entry.as_object_mut() {
        entry.insert("enabled".into(), serde_json::Value::Bool(true));
        enabled = true;
    }

    let rendered = serde_json::to_string_pretty(&root)?;
    let previous = std::fs::read_to_string(&state_file).ok();
    if previous.as_deref() != Some(rendered.as_str()) {
        if let Some(previous) = &previous {
            let backup = state_file.with_extension("json.noct.bak");
            let _ = std::fs::write(&backup, previous);
        }
        std::fs::write(&state_file, &rendered)
            .with_context(|| format!("no se pudo escribir {}", state_file.display()))?;
    }

    let (shortcut_added, settings_file) = ensure_control_center_shortcut()?;
    if shortcut_added {
        let _ = env::run("noctalia", &["msg", "config-reload"]);
    }

    let noct_in_path = env::capture("sh", &["-c", "command -v noct"])
        .map(|value| !value.is_empty())
        .unwrap_or(false);

    Ok(WidgetReport {
        plugin_dir: dir.display().to_string(),
        files: written,
        enabled,
        state_file: state_file.display().to_string(),
        settings_file,
        shortcut_added,
        noct_in_path,
    })
}
