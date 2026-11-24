//! Presets visuales de Umbriel: efectos de borde, cursor, pantalla y ventana.
//!
//! Noct escribe un fragmento gestionado (`~/.config/umbriel/noct/effects.toml`)
//! con las selecciones y añade los `effect.toml` de Umbriel al include del
//! usuario. Al cambiar de preset se retiran solo las entradas de efectos, nunca
//! el resto de la configuración.

use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde::Serialize;
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
        write_if_changed(&dir.join("shader.glsl"), shader)?;
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
        description: "Todo el escritorio en blanco y negro, con scanlines y viñeta.",
        border: None,
        cursor: None,
        screen: Some("noct-mono"),
        window: None,
        animations: &[("workspaces", "workspace_wipe")],
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

fn render_effects(preset_id: &str, preset: &Preset) -> String {
    let mut out = String::new();
    out.push_str("# Generado por Noct (noct effects). No editar a mano.\n");
    out.push_str(&format!("# preset: {preset_id}\n\n"));
    let effects = preset_effects(preset);
    if !effects.is_empty() {
        out.push_str("[effects]\n");
        for (kind, name) in &effects {
            out.push_str(&format!("{kind} = \"{name}\"\n"));
        }
        out.push('\n');
    }
    for (group, name) in preset.animations {
        out.push_str(&format!("[animation.{group}]\n"));
        out.push_str(&format!("effect = \"{name}\"\n\n"));
    }
    out
}

fn effect_includes(preset: &Preset) -> Result<Vec<String>> {
    let mut out = Vec::new();
    for (kind, name) in preset_effects(preset) {
        let path = find_effect(kind, name).with_context(|| {
            format!("no está instalado el efecto {kind}/{name} (mira `noct effects list`)")
        })?;
        out.push(path.display().to_string());
    }
    for (_, name) in preset.animations {
        let path = find_effect("animation", name)
            .with_context(|| format!("no está instalada la animación {name}"))?;
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

/// Aplica un preset. Con `dry_run` solo devuelve el fragmento generado.
pub fn apply(id: &str, dry_run: bool) -> Result<(ApplyReport, String)> {
    let Some(preset) = preset(id) else {
        let ids: Vec<&str> = PRESETS.iter().map(|p| p.id).collect();
        bail!("preset desconocido: {id}. Disponibles: {}", ids.join(", "));
    };

    ensure_custom_assets()?;
    let includes = effect_includes(preset)?;
    let content = render_effects(preset.id, preset);
    let file = effects_file();

    let mut report = ApplyReport {
        preset: preset.id.to_string(),
        name: preset.name.to_string(),
        file: file.display().to_string(),
        effects: preset_effects(preset)
            .into_iter()
            .map(|(kind, name)| format!("{kind}={name}"))
            .collect(),
        animations: preset
            .animations
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
pub struct CurrentEffects {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preset: Option<String>,
    pub effects: Vec<String>,
    pub animations: Vec<String>,
}

/// Lee las selecciones activas del fragmento gestionado.
pub fn current() -> Result<CurrentEffects> {
    let path = effects_file();
    if !path.exists() {
        return Ok(CurrentEffects {
            preset: None,
            effects: Vec::new(),
            animations: Vec::new(),
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
        effects,
        animations,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_effects_and_animations() {
        let preset = preset("neon").unwrap();
        let text = render_effects("neon", preset);
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
        let preset = preset("off").unwrap();
        let text = render_effects("off", preset);
        let value: toml::Value = toml::from_str(&text).unwrap();
        assert!(value.get("effects").is_none());
    }

    #[test]
    fn known_presets_have_effects() {
        assert_eq!(presets().len(), 6);
        assert!(presets().iter().any(|p| p.id == "matrix"));
        assert!(presets().iter().any(|p| p.id == "mono"));
    }
}
