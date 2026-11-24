//! Puente de configuración Niri ↔ Umbriel.
//!
//! Modelo canónico en `~/.config/noct/profile.toml`. Noct exporta lo que puede
//! leer del compositor activo, lo guarda y genera fragmentos gestionados para el
//! otro. Los fragmentos se incluyen al final, de modo que nunca se reescriben
//! los archivos del usuario, y lo que no se puede mapear se reporta en lugar de
//! perderse en silencio.
#![allow(clippy::field_reassign_with_default)]

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

use crate::env;
use crate::kdl;

pub const PROFILE_VERSION: u32 = 1;

pub fn profile_path() -> PathBuf {
    env::app_config("noct").join("profile.toml")
}

fn niri_dir() -> PathBuf {
    env::app_config("niri")
}

fn umbriel_dir() -> PathBuf {
    env::app_config("umbriel")
}

pub fn niri_bridge_file() -> PathBuf {
    niri_dir().join("noct-bridge.kdl")
}

pub fn umbriel_bridge_file() -> PathBuf {
    umbriel_dir().join("noct").join("bridge.toml")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub meta: Meta,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keyboard: Option<Keyboard>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub touchpad: Option<Touchpad>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mouse: Option<Mouse>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cursor: Option<Cursor>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub focus: Option<Focus>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout: Option<Layout>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub appearance: Option<Appearance>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub autostart: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub outputs: Vec<Output>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub window_rules: Vec<WindowRule>,
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            meta: Meta {
                version: PROFILE_VERSION,
                source: String::new(),
                exported_at: now_epoch(),
            },
            keyboard: None,
            touchpad: None,
            mouse: None,
            cursor: None,
            focus: None,
            layout: None,
            appearance: None,
            autostart: Vec::new(),
            outputs: Vec::new(),
            window_rules: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Meta {
    pub version: u32,
    pub source: String,
    pub exported_at: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Keyboard {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variant: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub options: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub numlock: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repeat_delay: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repeat_rate: Option<i64>,
}

impl Keyboard {
    fn is_empty(&self) -> bool {
        self.layout.is_none()
            && self.variant.is_none()
            && self.model.is_none()
            && self.options.is_none()
            && self.numlock.is_none()
            && self.repeat_delay.is_none()
            && self.repeat_rate.is_none()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Touchpad {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tap: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub natural_scroll: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_while_typing: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accel_profile: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub click_method: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub left_handed: Option<bool>,
}

impl Touchpad {
    fn is_empty(&self) -> bool {
        self.tap.is_none()
            && self.natural_scroll.is_none()
            && self.disable_while_typing.is_none()
            && self.accel_profile.is_none()
            && self.click_method.is_none()
            && self.left_handed.is_none()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Mouse {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accel_profile: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub natural_scroll: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub left_handed: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sensitivity: Option<f64>,
}

impl Mouse {
    fn is_empty(&self) -> bool {
        self.accel_profile.is_none()
            && self.natural_scroll.is_none()
            && self.left_handed.is_none()
            && self.sensitivity.is_none()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Cursor {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub theme: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<i64>,
}

impl Cursor {
    fn is_empty(&self) -> bool {
        self.theme.is_none() && self.size.is_none()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Focus {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub follows_mouse: Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Layout {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gaps: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extent_presets: Option<Vec<f64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_extent_fraction: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub center_focused: Option<String>,
}

impl Layout {
    fn is_empty(&self) -> bool {
        self.gaps.is_none()
            && self.extent_presets.is_none()
            && self.default_extent_fraction.is_none()
            && self.center_focused.is_none()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Appearance {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub corner_radius: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub border_width: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prefer_no_csd: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blur: Option<bool>,
}

impl Appearance {
    fn is_empty(&self) -> bool {
        self.corner_radius.is_none()
            && self.border_width.is_none()
            && self.prefer_no_csd.is_none()
            && self.blur.is_none()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Output {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scale: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transform: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub position: Option<[i64; 2]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vrr: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct WindowRule {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub app_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub floating: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fullscreen: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub maximized: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspace: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extent_fraction: Option<f64>,
}

fn now_epoch() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ExportReport {
    pub profile: Profile,
    pub mapped: Vec<String>,
    pub unmapped: Vec<String>,
    pub source_files: Vec<String>,
}

impl Profile {
    /// Lee el modelo canónico guardado.
    pub fn load(path: &Path) -> Result<Self> {
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("no se pudo leer {}", path.display()))?;
        let profile: Profile = toml::from_str(&text)
            .with_context(|| format!("{} no es un perfil válido", path.display()))?;
        Ok(profile)
    }

    pub fn load_default() -> Result<Self> {
        Self::load(&profile_path())
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("no se pudo crear {}", parent.display()))?;
        }
        let text = self.to_toml()?;
        std::fs::write(path, text)
            .with_context(|| format!("no se pudo escribir {}", path.display()))?;
        Ok(())
    }

    /// Serializa el perfil a TOML legible.
    pub fn to_toml(&self) -> Result<String> {
        toml::to_string_pretty(self).context("no se pudo serializar el perfil")
    }

    /// Exporta desde el compositor indicado (`niri` o `umbriel`).
    pub fn export_from(source: &str) -> Result<ExportReport> {
        match source {
            "niri" => export_niri(),
            "umbriel" => export_umbriel(),
            other => bail!("compositor desconocido: {other} (usa niri o umbriel)"),
        }
    }

    /// Genera el fragmento gestionado para Niri.
    pub fn to_niri(&self) -> String {
        let mut out = String::new();
        out.push_str("// Generado por Noct (noct bridge). No editar a mano.\n");
        out.push_str("// Cambia el perfil con `noct bridge` y vuelve a aplicar.\n\n");

        let has_input = self.keyboard.as_ref().is_some_and(kb_has_niri)
            || self.touchpad.as_ref().is_some_and(tp_has_niri)
            || self.mouse.as_ref().is_some_and(ms_has_niri)
            || self.focus.as_ref().and_then(|f| f.follows_mouse) == Some(true);
        if has_input {
            out.push_str("input {\n");
            if let Some(kb) = &self.keyboard {
                let mut block = String::new();
                let mut xkb = String::new();
                push_opt_str(&mut xkb, 12, "layout", &kb.layout);
                push_opt_str(&mut xkb, 12, "variant", &kb.variant);
                push_opt_str(&mut xkb, 12, "model", &kb.model);
                push_opt_str(&mut xkb, 12, "options", &kb.options);
                if !xkb.is_empty() {
                    block.push_str("        xkb {\n");
                    block.push_str(&xkb);
                    block.push_str("        }\n");
                }
                if kb.numlock == Some(true) {
                    block.push_str("        numlock\n");
                }
                if let Some(v) = kb.repeat_delay {
                    block.push_str(&format!("        repeat-delay {v}\n"));
                }
                if let Some(v) = kb.repeat_rate {
                    block.push_str(&format!("        repeat-rate {v}\n"));
                }
                if !block.is_empty() {
                    out.push_str("    keyboard {\n");
                    out.push_str(&block);
                    out.push_str("    }\n");
                }
            }

            if let Some(tp) = &self.touchpad {
                let mut block = String::new();
                push_flag(&mut block, 8, "tap", tp.tap);
                push_flag(&mut block, 8, "natural-scroll", tp.natural_scroll);
                push_flag(&mut block, 8, "dwt", tp.disable_while_typing);
                push_flag(&mut block, 8, "left-handed", tp.left_handed);
                push_opt_str(&mut block, 8, "accel-profile", &tp.accel_profile);
                push_opt_str(&mut block, 8, "click-method", &tp.click_method);
                if !block.is_empty() {
                    out.push_str("    touchpad {\n");
                    out.push_str(&block);
                    out.push_str("    }\n");
                }
            }

            if let Some(ms) = &self.mouse {
                let mut block = String::new();
                push_opt_str(&mut block, 8, "accel-profile", &ms.accel_profile);
                push_flag(&mut block, 8, "natural-scroll", ms.natural_scroll);
                push_flag(&mut block, 8, "left-handed", ms.left_handed);
                if !block.is_empty() {
                    out.push_str("    mouse {\n");
                    out.push_str(&block);
                    out.push_str("    }\n");
                }
            }

            if self.focus.as_ref().and_then(|f| f.follows_mouse) == Some(true) {
                out.push_str("    focus-follows-mouse\n");
            }
            out.push_str("}\n\n");
        }

        if let Some(cursor) = &self.cursor {
            out.push_str("cursor {\n");
            push_opt_str(&mut out, 4, "xcursor-theme", &cursor.theme);
            if let Some(size) = cursor.size {
                out.push_str(&format!("    xcursor-size {size}\n"));
            }
            out.push_str("}\n\n");
        }

        if let Some(layout) = &self.layout {
            out.push_str("layout {\n");
            if let Some(gaps) = layout.gaps {
                out.push_str(&format!("    gaps {gaps}\n"));
            }
            if let Some(center) = &layout.center_focused {
                out.push_str(&format!("    center-focused-column {}\n", quote(center)));
            }
            if let Some(presets) = &layout.extent_presets {
                out.push_str("    preset-column-widths {\n");
                for preset in presets {
                    out.push_str(&format!("        proportion {preset}\n"));
                }
                out.push_str("    }\n");
            }
            if let Some(fraction) = layout.default_extent_fraction {
                out.push_str("    default-column-width {\n");
                out.push_str(&format!("        proportion {fraction}\n"));
                out.push_str("    }\n");
            }
            out.push_str("}\n\n");
        }

        if let Some(appearance) = &self.appearance {
            if appearance.prefer_no_csd == Some(true) {
                out.push_str("prefer-no-csd\n\n");
            }
            if let Some(radius) = appearance.corner_radius {
                out.push_str("window-rule {\n");
                out.push_str(&format!("    geometry-corner-radius {radius}\n"));
                out.push_str("    clip-to-geometry true\n");
                out.push_str("}\n\n");
            }
        }

        for output in &self.outputs {
            out.push_str(&format!("output {} {{\n", quote(output.name.as_str())));
            push_opt_str(&mut out, 4, "mode", &output.mode);
            if let Some(scale) = output.scale {
                out.push_str(&format!("    scale {scale}\n"));
            }
            if let Some(enabled) = output.enabled {
                if !enabled {
                    out.push_str("    off\n");
                }
            }
            out.push_str("}\n\n");
        }

        for rule in &self.window_rules {
            out.push_str("window-rule {\n");
            let has_match = rule.app_id.is_some() || rule.title.is_some();
            if has_match {
                out.push_str("    match");
                if let Some(app_id) = &rule.app_id {
                    out.push_str(&format!(" app-id={}", quote(app_id.as_str())));
                }
                if let Some(title) = &rule.title {
                    out.push_str(&format!(" title={}", kdl_string(title.as_str())));
                }
                out.push('\n');
            }
            if let Some(floating) = rule.floating {
                out.push_str(&format!("    open-floating {floating}\n"));
            }
            if let Some(fullscreen) = rule.fullscreen {
                out.push_str(&format!("    open-fullscreen {fullscreen}\n"));
            }
            if let Some(maximized) = rule.maximized {
                out.push_str(&format!("    open-maximized {maximized}\n"));
            }
            if let Some(output) = &rule.output {
                out.push_str(&format!("    open-on-output {}\n", quote(output.as_str())));
            }
            if let Some(workspace) = &rule.workspace {
                out.push_str(&format!(
                    "    open-on-workspace {}\n",
                    quote(workspace.as_str())
                ));
            }
            if let Some(fraction) = rule.extent_fraction {
                out.push_str("    default-column-width {\n");
                out.push_str(&format!("        proportion {fraction}\n"));
                out.push_str("    }\n");
            }
            out.push_str("}\n\n");
        }

        // El autostart se guarda en el perfil para consulta, pero no se emite:
        // duplicaría los procesos del compositor de origen. Cada compositor
        // mantiene su propia lista de arranque.

        out
    }

    /// Genera el fragmento gestionado para Umbriel.
    pub fn to_umbriel(&self) -> String {
        let mut out = String::new();
        out.push_str("# Generado por Noct (noct bridge). No editar a mano.\n");
        out.push_str("# Cambia el perfil con `noct bridge` y vuelve a aplicar.\n\n");

        // [input.*]
        if let Some(kb) = &self.keyboard {
            let mut lines = Vec::new();
            push_kv_str(&mut lines, "layout", &kb.layout);
            push_kv_str(&mut lines, "variant", &kb.variant);
            push_kv_str(&mut lines, "model", &kb.model);
            push_kv_str(&mut lines, "options", &kb.options);
            push_kv_bool(&mut lines, "numlock_toggle", kb.numlock);
            push_kv_int(&mut lines, "repeat_delay", kb.repeat_delay);
            push_kv_int(&mut lines, "repeat_rate", kb.repeat_rate);
            emit_section(&mut out, "[input.keyboard]", &lines);
        }
        if let Some(tp) = &self.touchpad {
            let mut lines = Vec::new();
            push_kv_bool(&mut lines, "tap", tp.tap);
            push_kv_bool(&mut lines, "natural_scroll", tp.natural_scroll);
            push_kv_bool(&mut lines, "disable_while_typing", tp.disable_while_typing);
            push_kv_str(&mut lines, "accel_profile", &tp.accel_profile);
            push_kv_str(&mut lines, "click_method", &tp.click_method);
            push_kv_bool(&mut lines, "left_handed", tp.left_handed);
            emit_section(&mut out, "[input.touchpad]", &lines);
        }
        if let Some(ms) = &self.mouse {
            let mut lines = Vec::new();
            push_kv_str(&mut lines, "accel_profile", &ms.accel_profile);
            push_kv_bool(&mut lines, "natural_scroll", ms.natural_scroll);
            push_kv_bool(&mut lines, "left_handed", ms.left_handed);
            push_kv_float(&mut lines, "sensitivity", ms.sensitivity);
            emit_section(&mut out, "[input.mouse]", &lines);
        }
        if let Some(cursor) = &self.cursor {
            let mut lines = Vec::new();
            push_kv_str(&mut lines, "theme", &cursor.theme);
            push_kv_int(&mut lines, "size", cursor.size);
            emit_section(&mut out, "[input.cursor]", &lines);
        }
        if let Some(focus) = &self.focus {
            let mut lines = Vec::new();
            push_kv_bool(&mut lines, "follows_mouse", focus.follows_mouse);
            emit_section(&mut out, "[input.focus]", &lines);
        }

        // [layout] + [layout.scrolling]
        if let Some(layout) = &self.layout {
            let mut lines = Vec::new();
            push_kv_int(&mut lines, "gap", layout.gaps);
            if let Some(presets) = &layout.extent_presets {
                let values: Vec<String> = presets.iter().map(|v| toml_float(*v)).collect();
                lines.push(format!("extent_presets = [{}]", values.join(", ")));
            }
            emit_section(&mut out, "[layout]", &lines);

            let mut scrolling = Vec::new();
            push_kv_float(
                &mut scrolling,
                "default_extent_fraction",
                layout.default_extent_fraction,
            );
            if let Some(center) = &layout.center_focused {
                scrolling.push(format!(
                    "center_focused = {}",
                    toml_string(&center.replace('-', "_"))
                ));
            }
            emit_section(&mut out, "[layout.scrolling]", &scrolling);
        }

        // [appearance] + [appearance.blur]
        if let Some(appearance) = &self.appearance {
            let mut lines = Vec::new();
            push_kv_int(&mut lines, "corner_radius", appearance.corner_radius);
            push_kv_int(&mut lines, "border_width", appearance.border_width);
            push_kv_bool(&mut lines, "prefer_no_csd", appearance.prefer_no_csd);
            emit_section(&mut out, "[appearance]", &lines);

            if let Some(blur) = appearance.blur {
                emit_section(
                    &mut out,
                    "[appearance.blur]",
                    &[format!("enabled = {blur}")],
                );
            }
        }

        for output in &self.outputs {
            let mut lines = Vec::new();
            push_kv_str(&mut lines, "mode", &output.mode);
            push_kv_float(&mut lines, "scale", output.scale);
            push_kv_str(&mut lines, "transform", &output.transform);
            push_kv_bool(&mut lines, "enabled", output.enabled);
            if let Some(position) = output.position {
                lines.push(format!("position = [{}, {}]", position[0], position[1]));
            }
            let header = format!("[output.{}]", toml_key(&output.name));
            emit_section(&mut out, &header, &lines);
        }

        for rule in &self.window_rules {
            let mut lines = Vec::new();
            push_kv_bool(&mut lines, "default_floating", rule.floating);
            push_kv_bool(&mut lines, "default_fullscreen", rule.fullscreen);
            push_kv_bool(&mut lines, "default_maximize", rule.maximized);
            push_kv_str(&mut lines, "default_output", &rule.output);
            push_kv_str(&mut lines, "default_workspace", &rule.workspace);
            push_kv_float(&mut lines, "default_scrolling_extent", rule.extent_fraction);
            if lines.is_empty() && rule.app_id.is_none() && rule.title.is_none() {
                continue;
            }
            out.push_str("[[window_rule]]\n");
            for line in &lines {
                out.push_str(line);
                out.push('\n');
            }
            out.push('\n');

            if rule.app_id.is_some() || rule.title.is_some() {
                let mut m = Vec::new();
                push_kv_str(&mut m, "app_id", &rule.app_id);
                push_kv_str(&mut m, "title", &rule.title);
                emit_section(&mut out, "[window_rule.match]", &m);
            }
        }

        out
    }
}

fn toml_string(value: &str) -> String {
    let escaped = value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\t', "\\t");
    format!("\"{escaped}\"")
}

fn toml_float(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{value:.1}")
    } else {
        let text = format!("{value}");
        text
    }
}

fn emit_section(out: &mut String, header: &str, lines: &[String]) {
    if lines.is_empty() {
        return;
    }
    out.push_str(header);
    out.push('\n');
    for line in lines {
        out.push_str(line);
        out.push('\n');
    }
    out.push('\n');
}

fn push_kv_str(lines: &mut Vec<String>, key: &str, value: &Option<String>) {
    if let Some(value) = value {
        lines.push(format!("{key} = {}", toml_string(value)));
    }
}

fn push_kv_bool(lines: &mut Vec<String>, key: &str, value: Option<bool>) {
    if let Some(value) = value {
        lines.push(format!("{key} = {value}"));
    }
}

fn push_kv_int(lines: &mut Vec<String>, key: &str, value: Option<i64>) {
    if let Some(value) = value {
        lines.push(format!("{key} = {value}"));
    }
}

fn push_kv_float(lines: &mut Vec<String>, key: &str, value: Option<f64>) {
    if let Some(value) = value {
        lines.push(format!("{key} = {}", toml_float(value)));
    }
}

fn kb_has_niri(kb: &Keyboard) -> bool {
    kb.layout.is_some()
        || kb.variant.is_some()
        || kb.model.is_some()
        || kb.options.is_some()
        || kb.numlock == Some(true)
        || kb.repeat_delay.is_some()
        || kb.repeat_rate.is_some()
}

fn tp_has_niri(tp: &Touchpad) -> bool {
    tp.tap == Some(true)
        || tp.natural_scroll == Some(true)
        || tp.disable_while_typing == Some(true)
        || tp.left_handed == Some(true)
        || tp.accel_profile.is_some()
        || tp.click_method.is_some()
}

fn ms_has_niri(ms: &Mouse) -> bool {
    ms.accel_profile.is_some() || ms.natural_scroll == Some(true) || ms.left_handed == Some(true)
}

fn quote(value: &str) -> String {
    let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}

/// Cadena KDL segura: usa raw string si contiene barras invertidas (regex).
fn kdl_string(value: &str) -> String {
    if !value.contains('\\') {
        return quote(value);
    }
    let mut hashes = 1usize;
    loop {
        let terminator = format!("\"{}", "#".repeat(hashes));
        if !value.contains(&terminator) {
            break;
        }
        hashes += 1;
    }
    let hashes = "#".repeat(hashes);
    format!("r{hashes}\"{value}\"{hashes}")
}

fn toml_key(value: &str) -> String {
    if value
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        value.to_string()
    } else {
        quote(value)
    }
}

fn push_opt_str(out: &mut String, indent: usize, key: &str, value: &Option<String>) {
    if let Some(value) = value {
        out.push_str(&format!("{}{} {}\n", " ".repeat(indent), key, quote(value)));
    }
}

fn push_flag(out: &mut String, indent: usize, key: &str, value: Option<bool>) {
    if value == Some(true) {
        out.push_str(&format!("{}{}\n", " ".repeat(indent), key));
    }
}

// ---------------------------------------------------------------------------
// Exportación desde Niri
// ---------------------------------------------------------------------------

fn niri_config_files() -> Vec<PathBuf> {
    let dir = niri_dir();
    let mut files = Vec::new();
    let mut visited = Vec::new();

    fn visit(path: &Path, files: &mut Vec<PathBuf>, visited: &mut Vec<PathBuf>, depth: usize) {
        if depth == 0 || !path.is_file() {
            return;
        }
        let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        if visited.contains(&canonical) {
            return;
        }
        visited.push(canonical);
        files.push(path.to_path_buf());

        let Ok(text) = std::fs::read_to_string(path) else {
            return;
        };
        let base = path.parent().unwrap_or(Path::new("."));
        for node in kdl::parse(&text) {
            if node.name != "include" {
                continue;
            }
            let Some(value) = node.arg(0).and_then(|v| v.as_str()) else {
                continue;
            };
            let candidate = if Path::new(value).is_absolute() {
                PathBuf::from(value)
            } else {
                base.join(value)
            };
            visit(&candidate, files, visited, depth - 1);
        }
    }

    visit(&dir.join("config.kdl"), &mut files, &mut visited, 16);

    // Incluye cualquier .kdl suelto que no se haya alcanzado desde config.kdl.
    let mut extras = Vec::new();
    collect_kdl(&dir, 3, &mut extras);
    for file in extras {
        if file.file_name().and_then(|n| n.to_str()) == Some("noct-bridge.kdl") {
            continue;
        }
        let canonical = file.canonicalize().unwrap_or_else(|_| file.clone());
        if !visited.contains(&canonical) {
            visited.push(canonical);
            files.push(file);
        }
    }

    files
}

fn collect_kdl(dir: &Path, depth: usize, out: &mut Vec<PathBuf>) {
    if depth == 0 {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_kdl(&path, depth - 1, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("kdl") {
            out.push(path);
        }
    }
}

fn dedup_profile(profile: &mut Profile) {
    let mut rules: Vec<WindowRule> = Vec::new();
    for rule in profile.window_rules.drain(..) {
        if !rules.contains(&rule) {
            rules.push(rule);
        }
    }
    profile.window_rules = rules;

    let mut outputs: Vec<Output> = Vec::new();
    for output in profile.outputs.drain(..) {
        if let Some(existing) = outputs.iter_mut().find(|o| o.name == output.name) {
            *existing = output;
        } else {
            outputs.push(output);
        }
    }
    profile.outputs = outputs;

    let mut autostart = Vec::new();
    for command in profile.autostart.drain(..) {
        if !autostart.contains(&command) {
            autostart.push(command);
        }
    }
    profile.autostart = autostart;
}

fn export_niri() -> Result<ExportReport> {
    let files = niri_config_files();
    if files.is_empty() {
        bail!("no se encontró ~/.config/niri/config.kdl");
    }

    let mut nodes = Vec::new();
    for file in &files {
        if let Ok(text) = std::fs::read_to_string(file) {
            nodes.extend(kdl::parse(&text));
        }
    }

    let mut profile = Profile::default();
    profile.meta.source = "niri".to_string();
    let mut mapped = Vec::new();
    let mut unmapped = Vec::new();

    for node in &nodes {
        match node.name.as_str() {
            "input" => {
                if let Some(kb) = node.child("keyboard") {
                    let mut keyboard = Keyboard::default();
                    if let Some(xkb) = kb.child("xkb") {
                        keyboard.layout = xkb.child("layout").and_then(arg_string);
                        keyboard.variant = xkb.child("variant").and_then(arg_string);
                        keyboard.model = xkb.child("model").and_then(arg_string);
                        keyboard.options = xkb.child("options").and_then(arg_string);
                    }
                    keyboard.numlock = Some(kb.has_child("numlock"));
                    keyboard.repeat_delay = kb.child("repeat-delay").and_then(arg_int);
                    keyboard.repeat_rate = kb.child("repeat-rate").and_then(arg_int);
                    if !keyboard.is_empty() {
                        mapped.push("keyboard".into());
                        profile.keyboard = Some(keyboard);
                    }
                }
                if let Some(tp) = node.child("touchpad") {
                    let mut touchpad = Touchpad::default();
                    touchpad.tap = Some(tp.has_child("tap"));
                    touchpad.natural_scroll = Some(tp.has_child("natural-scroll"));
                    touchpad.disable_while_typing = Some(tp.has_child("dwt"));
                    touchpad.left_handed = tp.has_child("left-handed").then_some(true);
                    touchpad.accel_profile = tp.child("accel-profile").and_then(arg_string);
                    touchpad.click_method = tp.child("click-method").and_then(arg_string);
                    if !touchpad.is_empty() {
                        mapped.push("touchpad".into());
                        profile.touchpad = Some(touchpad);
                    }
                }
                if let Some(ms) = node.child("mouse") {
                    let mut mouse = Mouse::default();
                    mouse.accel_profile = ms.child("accel-profile").and_then(arg_string);
                    mouse.natural_scroll = Some(ms.has_child("natural-scroll"));
                    mouse.left_handed = ms.has_child("left-handed").then_some(true);
                    if !mouse.is_empty() {
                        mapped.push("mouse".into());
                        profile.mouse = Some(mouse);
                    }
                }
                let follows = node.has_child("focus-follows-mouse");
                profile.focus = Some(Focus {
                    follows_mouse: Some(follows),
                });
                mapped.push("input.focus".into());
                if node.has_child("workspace-auto-back-and-forth") {
                    unmapped.push("input.workspace-auto-back-and-forth".into());
                }
            }
            "cursor" => {
                let mut cursor = Cursor::default();
                cursor.theme = node.child("xcursor-theme").and_then(arg_string);
                cursor.size = node.child("xcursor-size").and_then(arg_int);
                if !cursor.is_empty() {
                    mapped.push("cursor".into());
                    profile.cursor = Some(cursor);
                }
            }
            "layout" => {
                let mut layout = Layout::default();
                layout.gaps = node.child("gaps").and_then(arg_int);
                layout.center_focused = node.child("center-focused-column").and_then(arg_string);
                if let Some(presets) = node.child("preset-column-widths") {
                    let values: Vec<f64> = presets
                        .children_named("proportion")
                        .filter_map(arg_float)
                        .collect();
                    if !values.is_empty() {
                        layout.extent_presets = Some(values);
                    }
                }
                if let Some(width) = node.child("default-column-width") {
                    layout.default_extent_fraction = width.child("proportion").and_then(arg_float);
                }
                if !layout.is_empty() {
                    mapped.push("layout".into());
                    profile.layout = Some(layout);
                }
                for child in &node.children {
                    if !matches!(
                        child.name.as_str(),
                        "gaps"
                            | "center-focused-column"
                            | "preset-column-widths"
                            | "default-column-width"
                            | "struts"
                    ) {
                        unmapped.push(format!("layout.{}", child.name));
                    }
                }
            }
            "prefer-no-csd" => {
                let appearance = profile.appearance.get_or_insert_with(Appearance::default);
                appearance.prefer_no_csd = Some(true);
                mapped.push("appearance.prefer_no_csd".into());
            }
            "window-rule" => {
                let mut rule = WindowRule::default();
                if let Some(m) = node.child("match") {
                    rule.app_id = m
                        .prop("app-id")
                        .and_then(|v| v.as_str().map(str::to_string));
                    rule.title = m.prop("title").and_then(|v| v.as_str().map(str::to_string));
                }
                rule.floating = node.child("open-floating").and_then(arg_bool);
                rule.fullscreen = node.child("open-fullscreen").and_then(arg_bool);
                rule.maximized = node.child("open-maximized").and_then(arg_bool);
                rule.output = node.child("open-on-output").and_then(arg_string);
                rule.workspace = node.child("open-on-workspace").and_then(arg_string);
                rule.extent_fraction = node
                    .child("default-column-width")
                    .and_then(|n| n.child("proportion"))
                    .and_then(arg_float);

                if rule.app_id.is_none() && rule.title.is_none() {
                    if let Some(radius) = node.child("geometry-corner-radius").and_then(arg_int) {
                        let appearance = profile.appearance.get_or_insert_with(Appearance::default);
                        appearance.corner_radius = Some(radius);
                        mapped.push("appearance.corner_radius".into());
                        continue;
                    }
                }

                if rule.app_id.is_some()
                    || rule.title.is_some()
                    || rule.floating.is_some()
                    || rule.fullscreen.is_some()
                    || rule.maximized.is_some()
                    || rule.output.is_some()
                    || rule.workspace.is_some()
                    || rule.extent_fraction.is_some()
                {
                    mapped.push(format!(
                        "window_rule[{}]",
                        rule.app_id.clone().unwrap_or_else(|| "?".into())
                    ));
                    profile.window_rules.push(rule);
                }
            }
            "output" => {
                let Some(name) = node.arg(0).and_then(|v| v.as_str()) else {
                    continue;
                };
                let mut output = Output {
                    name: name.to_string(),
                    ..Output::default()
                };
                output.mode = node.child("mode").and_then(arg_string);
                output.scale = node.child("scale").and_then(arg_float);
                output.transform = node.child("transform").and_then(arg_string);
                output.vrr = node.child("variable-refresh-rate").and_then(arg_string);
                output.enabled = Some(!node.has_child("off"));
                if let Some(position) = node.child("position") {
                    let x = position.prop("x").and_then(|v| v.as_i64());
                    let y = position.prop("y").and_then(|v| v.as_i64());
                    if let (Some(x), Some(y)) = (x, y) {
                        output.position = Some([x, y]);
                    }
                }
                mapped.push(format!("output.{name}"));
                profile.outputs.push(output);
            }
            "spawn-at-startup" => {
                let args: Vec<String> = node
                    .args
                    .iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .collect();
                if !args.is_empty() {
                    let command = shell_join(&args);
                    if command.to_ascii_lowercase().contains("niri") {
                        unmapped.push("autostart específico de niri".into());
                    } else {
                        profile.autostart.push(command);
                        mapped.push("autostart".into());
                    }
                }
            }
            "spawn-sh-at-startup" => {
                if let Some(command) = arg_string(node) {
                    if command.to_ascii_lowercase().contains("niri") {
                        unmapped.push("autostart específico de niri".into());
                    } else {
                        profile.autostart.push(command);
                        mapped.push("autostart".into());
                    }
                }
            }
            "animations" => unmapped.push("animations".into()),
            "environment" => unmapped.push("environment".into()),
            "hotkey-overlay" => unmapped.push("hotkey-overlay".into()),
            "screenshot-path" => unmapped.push("screenshot-path".into()),
            "debug" => unmapped.push("debug".into()),
            "binds" => unmapped.push("keybinds (usa `noct keys`)".into()),
            "include" => {}
            other => unmapped.push(other.to_string()),
        }
    }

    dedup_profile(&mut profile);

    mapped.sort();
    mapped.dedup();
    unmapped.sort();
    unmapped.dedup();

    Ok(ExportReport {
        profile,
        mapped,
        unmapped,
        source_files: files.iter().map(|p| p.display().to_string()).collect(),
    })
}

fn arg_string(node: &kdl::Node) -> Option<String> {
    node.arg(0).and_then(|v| v.as_str().map(str::to_string))
}

fn arg_int(node: &kdl::Node) -> Option<i64> {
    node.arg(0).and_then(|v| v.as_i64())
}

fn arg_float(node: &kdl::Node) -> Option<f64> {
    node.arg(0).and_then(|v| v.as_f64())
}

fn arg_bool(node: &kdl::Node) -> Option<bool> {
    node.arg(0).and_then(|v| v.as_bool())
}

fn shell_quote(arg: &str) -> String {
    let safe = !arg.is_empty()
        && arg
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "-_./:@%+=,".contains(c));
    if safe {
        arg.to_string()
    } else {
        format!("'{}'", arg.replace('\'', "'\\''"))
    }
}

fn shell_join(args: &[String]) -> String {
    args.iter()
        .map(|arg| shell_quote(arg))
        .collect::<Vec<_>>()
        .join(" ")
}

// ---------------------------------------------------------------------------
// Exportación desde Umbriel
// ---------------------------------------------------------------------------

fn umbriel_config_files() -> Vec<PathBuf> {
    let root = umbriel_dir().join("config.toml");
    let mut files = Vec::new();
    let mut visited = Vec::new();
    collect_umbriel_file(&root, &mut files, &mut visited, 8);
    files
}

fn collect_umbriel_file(
    path: &Path,
    files: &mut Vec<PathBuf>,
    visited: &mut Vec<PathBuf>,
    depth: usize,
) {
    if depth == 0 || !path.is_file() {
        return;
    }
    let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    if visited.contains(&canonical) {
        return;
    }
    visited.push(canonical);

    let Ok(text) = std::fs::read_to_string(path) else {
        return;
    };
    let Ok(value) = text.parse::<toml::Value>() else {
        return;
    };
    let base = path.parent().unwrap_or(Path::new("."));

    for section in ["include", "include.optional"] {
        let parts: Vec<&str> = section.split('.').collect();
        let Some(table) = get_table(&value, &parts) else {
            continue;
        };
        let Some(list) = table.get("files").and_then(|v| v.as_array()) else {
            continue;
        };
        for entry in list.iter().filter_map(|v| v.as_str()) {
            let candidate = if Path::new(entry).is_absolute() {
                PathBuf::from(entry)
            } else {
                base.join(entry)
            };
            collect_umbriel_file(&candidate, files, visited, depth - 1);
        }
    }

    // El archivo incluido se aplica después de sus includes.
    files.push(path.to_path_buf());
}

fn deep_merge(base: &mut toml::Value, other: &toml::Value) {
    match (base, other) {
        (toml::Value::Table(a), toml::Value::Table(b)) => {
            for (key, value) in b {
                match a.get_mut(key) {
                    Some(slot) => deep_merge(slot, value),
                    None => {
                        a.insert(key.clone(), value.clone());
                    }
                }
            }
        }
        (toml::Value::Array(a), toml::Value::Array(b)) => {
            let tables = !a.is_empty()
                && !b.is_empty()
                && a.iter().all(|v| v.is_table())
                && b.iter().all(|v| v.is_table());
            if tables {
                a.extend(b.iter().cloned());
            } else {
                *a = b.clone();
            }
        }
        (slot, value) => *slot = value.clone(),
    }
}

fn export_umbriel() -> Result<ExportReport> {
    let path = umbriel_dir().join("config.toml");
    if !path.exists() {
        bail!("no se encontró ~/.config/umbriel/config.toml");
    }
    let files = umbriel_config_files();
    let mut value = toml::Value::Table(Default::default());
    for file in &files {
        let Ok(text) = std::fs::read_to_string(file) else {
            continue;
        };
        let Ok(parsed) = text.parse::<toml::Value>() else {
            continue;
        };
        deep_merge(&mut value, &parsed);
    }

    let mut profile = Profile::default();
    profile.meta.source = "umbriel".to_string();
    let mut mapped = Vec::new();
    let mut unmapped = Vec::new();

    if let Some(kb) = get_table(&value, &["input", "keyboard"]) {
        let mut keyboard = Keyboard::default();
        keyboard.layout = get_string(kb, "layout");
        keyboard.variant = get_string(kb, "variant");
        keyboard.model = get_string(kb, "model");
        keyboard.options = get_string(kb, "options");
        keyboard.numlock = get_bool(kb, "numlock_toggle");
        keyboard.repeat_delay = get_int(kb, "repeat_delay");
        keyboard.repeat_rate = get_int(kb, "repeat_rate");
        if !keyboard.is_empty() {
            mapped.push("keyboard".into());
            profile.keyboard = Some(keyboard);
        }
    }

    if let Some(tp) = get_table(&value, &["input", "touchpad"]) {
        let mut touchpad = Touchpad::default();
        touchpad.tap = get_bool(tp, "tap");
        touchpad.natural_scroll = get_bool(tp, "natural_scroll");
        touchpad.disable_while_typing = get_bool(tp, "disable_while_typing");
        touchpad.accel_profile = get_string(tp, "accel_profile");
        touchpad.click_method = get_string(tp, "click_method");
        touchpad.left_handed = get_bool(tp, "left_handed");
        if !touchpad.is_empty() {
            mapped.push("touchpad".into());
            profile.touchpad = Some(touchpad);
        }
    }

    if let Some(ms) = get_table(&value, &["input", "mouse"]) {
        let mut mouse = Mouse::default();
        mouse.accel_profile = get_string(ms, "accel_profile");
        mouse.natural_scroll = get_bool(ms, "natural_scroll");
        mouse.left_handed = get_bool(ms, "left_handed");
        mouse.sensitivity = get_float(ms, "sensitivity");
        if !mouse.is_empty() {
            mapped.push("mouse".into());
            profile.mouse = Some(mouse);
        }
    }

    if let Some(cursor) = get_table(&value, &["input", "cursor"]) {
        let mut out = Cursor::default();
        out.theme = get_string(cursor, "theme");
        out.size = get_int(cursor, "size");
        if !out.is_empty() {
            mapped.push("cursor".into());
            profile.cursor = Some(out);
        }
    }

    if let Some(focus) = get_table(&value, &["input", "focus"]) {
        if let Some(follows) = get_bool(focus, "follows_mouse") {
            profile.focus = Some(Focus {
                follows_mouse: Some(follows),
            });
            mapped.push("input.focus".into());
        }
    }

    if let Some(layout) = get_table(&value, &["layout"]) {
        let mut out = Layout::default();
        out.gaps = get_int(layout, "gap");
        out.extent_presets = get_float_array(layout, "extent_presets");
        if let Some(scrolling) = get_table(&value, &["layout", "scrolling"]) {
            out.default_extent_fraction = get_float(scrolling, "default_extent_fraction");
            out.center_focused =
                get_string(scrolling, "center_focused").map(|value| value.replace('_', "-"));
        }
        if !out.is_empty() {
            mapped.push("layout".into());
            profile.layout = Some(out);
        }
    }

    if let Some(appearance) = get_table(&value, &["appearance"]) {
        let mut out = Appearance::default();
        out.corner_radius = get_int(appearance, "corner_radius");
        out.border_width = get_int(appearance, "border_width");
        out.prefer_no_csd = get_bool(appearance, "prefer_no_csd");
        out.blur = get_table(&value, &["appearance", "blur"]).and_then(|b| get_bool(b, "enabled"));
        if !out.is_empty() {
            mapped.push("appearance".into());
            profile.appearance = Some(out);
        }
    }

    if let Some(general) = get_table(&value, &["general"]) {
        if let Some(autostart) = general.get("autostart").and_then(|v| v.as_array()) {
            for command in autostart.iter().filter_map(|v| v.as_str()) {
                if command.to_ascii_lowercase().contains("umbriel") {
                    unmapped.push("autostart específico de umbriel".into());
                } else {
                    profile.autostart.push(command.to_string());
                    mapped.push("autostart".into());
                }
            }
        }
    }

    if let Some(outputs) = value.get("output").and_then(|v| v.as_table()) {
        for (name, raw) in outputs {
            let Some(table) = raw.as_table() else {
                continue;
            };
            let mut output = Output {
                name: name.clone(),
                ..Output::default()
            };
            output.mode = get_string(table, "mode");
            output.scale = get_float(table, "scale");
            output.transform = get_string(table, "transform");
            output.vrr = get_string(table, "vrr");
            output.enabled = get_bool(table, "enabled");
            if let Some(position) = table.get("position").and_then(|v| v.as_array()) {
                let x = position.first().and_then(|v| v.as_integer());
                let y = position.get(1).and_then(|v| v.as_integer());
                if let (Some(x), Some(y)) = (x, y) {
                    output.position = Some([x, y]);
                }
            }
            mapped.push(format!("output.{name}"));
            profile.outputs.push(output);
        }
    }

    if let Some(rules) = value.get("window_rule").and_then(|v| v.as_array()) {
        for raw in rules {
            let Some(table) = raw.as_table() else {
                continue;
            };
            let mut rule = WindowRule::default();
            if let Some(m) = table.get("match").and_then(|v| v.as_table()) {
                rule.app_id = get_string(m, "app_id");
                rule.title = get_string(m, "title");
            }
            rule.floating = get_bool(table, "default_floating");
            rule.fullscreen = get_bool(table, "default_fullscreen");
            rule.maximized = get_bool(table, "default_maximize");
            rule.output = get_string(table, "default_output");
            rule.workspace = get_string(table, "default_workspace");
            rule.extent_fraction = get_float(table, "default_scrolling_extent");
            if rule.app_id.is_some()
                || rule.title.is_some()
                || rule.floating.is_some()
                || rule.fullscreen.is_some()
                || rule.maximized.is_some()
                || rule.output.is_some()
                || rule.workspace.is_some()
                || rule.extent_fraction.is_some()
            {
                mapped.push(format!(
                    "window_rule[{}]",
                    rule.app_id.clone().unwrap_or_else(|| "?".into())
                ));
                profile.window_rules.push(rule);
            }
        }
    }

    // Tablas conocidas que Noct no traduce (por ahora) se reportan.
    let known_top = [
        "include",
        "general",
        "input",
        "layout",
        "appearance",
        "output",
        "window_rule",
        "colors",
        "keybinds",
        "events",
    ];
    if let Some(table) = value.as_table() {
        for key in table.keys() {
            if !known_top.contains(&key.as_str()) {
                unmapped.push(key.clone());
            }
        }
    }
    for key in ["colors", "keybinds", "events"] {
        if value.get(key).is_some() {
            unmapped.push(format!("{key} (gestionado aparte)"));
        }
    }
    if value
        .get("layout")
        .and_then(|v| v.as_table())
        .and_then(|t| t.get("mode"))
        .is_some()
    {
        unmapped.push("layout.mode".into());
    }
    if value
        .get("layout")
        .and_then(|v| v.as_table())
        .and_then(|t| t.get("struts"))
        .is_some()
    {
        unmapped.push("layout.struts".into());
    }

    dedup_profile(&mut profile);

    mapped.sort();
    mapped.dedup();
    unmapped.sort();
    unmapped.dedup();

    Ok(ExportReport {
        profile,
        mapped,
        unmapped,
        source_files: vec![path.display().to_string()],
    })
}

fn get_table<'a>(value: &'a toml::Value, path: &[&str]) -> Option<&'a toml::value::Table> {
    let mut current = value;
    for key in path {
        current = current.get(*key)?;
    }
    current.as_table()
}

fn get_string(table: &toml::value::Table, key: &str) -> Option<String> {
    table.get(key).and_then(|v| v.as_str()).map(str::to_string)
}

fn get_bool(table: &toml::value::Table, key: &str) -> Option<bool> {
    table.get(key).and_then(|v| v.as_bool())
}

fn get_int(table: &toml::value::Table, key: &str) -> Option<i64> {
    table.get(key).and_then(|v| v.as_integer())
}

fn get_float(table: &toml::value::Table, key: &str) -> Option<f64> {
    table
        .get(key)
        .and_then(|v| v.as_float().or_else(|| v.as_integer().map(|i| i as f64)))
}

fn get_float_array(table: &toml::value::Table, key: &str) -> Option<Vec<f64>> {
    let values = table.get(key)?.as_array()?;
    Some(
        values
            .iter()
            .filter_map(|v| v.as_float().or_else(|| v.as_integer().map(|i| i as f64)))
            .collect(),
    )
}

// ---------------------------------------------------------------------------
// Aplicación
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct ApplyReport {
    pub target: String,
    pub file: String,
    pub written: bool,
    pub include_added: bool,
    pub validated: bool,
    pub message: String,
}

/// Comprueba que `niri validate` acepta la configuración actual.
fn validate_niri() -> Result<()> {
    let Some(out) = env::run("niri", &["validate"]) else {
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
        bail!("niri validate falló:\n{detail}");
    }
}

fn validate_umbriel() -> Result<()> {
    // Autocorrige claves del tema que Noctalia vuelve a escribir con el nombre
    // antiguo; si no, la validación fallaría por un problema ajeno al puente.
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

fn ensure_niri_include(config: &Path) -> Result<bool> {
    if !config.exists() {
        bail!("no existe {}", config.display());
    }
    let text = std::fs::read_to_string(config)
        .with_context(|| format!("no se pudo leer {}", config.display()))?;
    if text.contains("noct-bridge.kdl") {
        return Ok(false);
    }
    let mut updated = text;
    if !updated.ends_with('\n') {
        updated.push('\n');
    }
    updated.push_str("\n// Puente de configuración de Noct\ninclude \"noct-bridge.kdl\"\n");
    std::fs::write(config, updated)
        .with_context(|| format!("no se pudo escribir {}", config.display()))?;
    Ok(true)
}

fn ensure_umbriel_include(config: &Path) -> Result<bool> {
    if !config.exists() {
        bail!("no existe {}", config.display());
    }
    let text = std::fs::read_to_string(config)
        .with_context(|| format!("no se pudo leer {}", config.display()))?;
    let mut doc: toml_edit::DocumentMut = text
        .parse()
        .with_context(|| format!("{} no es TOML válido", config.display()))?;

    let entry = "noct/bridge.toml";
    let already = doc
        .get("include")
        .and_then(|i| i.as_table_like())
        .and_then(|t| t.get("optional"))
        .and_then(|i| i.as_table_like())
        .and_then(|t| t.get("files"))
        .and_then(|i| i.as_array())
        .map(|files| files.iter().any(|v| v.as_str() == Some(entry)))
        .unwrap_or(false);
    if already {
        return Ok(false);
    }

    let include = doc
        .entry("include")
        .or_insert(toml_edit::Item::Table(toml_edit::Table::new()));
    let optional = include
        .as_table_like_mut()
        .context("include no es una tabla")?
        .entry("optional")
        .or_insert(toml_edit::Item::Table(toml_edit::Table::new()));
    let files = optional
        .as_table_like_mut()
        .context("include.optional no es una tabla")?
        .entry("files")
        .or_insert(toml_edit::Item::Value(toml_edit::Value::Array(
            toml_edit::Array::new(),
        )));
    if !files.is_array() {
        *files = toml_edit::Item::Value(toml_edit::Value::Array(toml_edit::Array::new()));
    }
    let array = files.as_array_mut().context("files no es un array")?;
    array.push(entry);

    std::fs::write(config, doc.to_string())
        .with_context(|| format!("no se pudo escribir {}", config.display()))?;
    Ok(true)
}

fn write_with_rollback(path: &Path, contents: &str) -> Result<Option<String>> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("no se pudo crear {}", parent.display()))?;
    }
    let previous = std::fs::read_to_string(path).ok();
    std::fs::write(path, contents)
        .with_context(|| format!("no se pudo escribir {}", path.display()))?;
    Ok(previous)
}

fn restore(path: &Path, previous: Option<String>) {
    match previous {
        Some(content) => {
            let _ = std::fs::write(path, content);
        }
        None => {
            let _ = std::fs::remove_file(path);
        }
    }
}

/// Aplica el perfil al compositor indicado. Con `dry_run` solo devuelve el texto.
pub fn apply(profile: &Profile, target: &str, dry_run: bool) -> Result<(ApplyReport, String)> {
    match target {
        "niri" => apply_niri(profile, dry_run),
        "umbriel" => apply_umbriel(profile, dry_run),
        other => bail!("compositor desconocido: {other} (usa niri o umbriel)"),
    }
}

fn apply_niri(profile: &Profile, dry_run: bool) -> Result<(ApplyReport, String)> {
    let content = profile.to_niri();
    let path = niri_bridge_file();
    let mut report = ApplyReport {
        target: "niri".into(),
        file: path.display().to_string(),
        written: false,
        include_added: false,
        validated: false,
        message: String::new(),
    };

    if dry_run {
        report.message = "dry-run: no se escribió nada".into();
        return Ok((report, content));
    }

    let previous = write_with_rollback(&path, &content)?;
    report.written = true;

    let config = niri_dir().join("config.kdl");
    match ensure_niri_include(&config) {
        Ok(added) => report.include_added = added,
        Err(err) => {
            restore(&path, previous);
            return Err(err);
        }
    }

    if let Err(err) = validate_niri() {
        restore(&path, previous);
        return Err(err);
    }
    report.validated = true;

    // Niri recarga la configuración al detectar el cambio; no hace falta IPC.
    if let Some(out) = env::run("niri", &["msg", "version"]) {
        if out.success {
            report.message = if report.include_added {
                "fragmento y include escritos; Niri recargará la configuración".into()
            } else {
                "fragmento actualizado; Niri recargará la configuración".into()
            };
        } else {
            report.message = "fragmento escrito (Niri no está en ejecución)".into();
        }
    } else {
        report.message = "fragmento escrito (Niri no está en ejecución)".into();
    }

    Ok((report, content))
}

fn apply_umbriel(profile: &Profile, dry_run: bool) -> Result<(ApplyReport, String)> {
    let content = profile.to_umbriel();
    let path = umbriel_bridge_file();
    let mut report = ApplyReport {
        target: "umbriel".into(),
        file: path.display().to_string(),
        written: false,
        include_added: false,
        validated: false,
        message: String::new(),
    };

    if dry_run {
        report.message = "dry-run: no se escribió nada".into();
        return Ok((report, content));
    }

    let previous = write_with_rollback(&path, &content)?;
    report.written = true;

    let config = umbriel_dir().join("config.toml");
    match ensure_umbriel_include(&config) {
        Ok(added) => report.include_added = added,
        Err(err) => {
            restore(&path, previous);
            return Err(err);
        }
    }

    if let Err(err) = validate_umbriel() {
        restore(&path, previous);
        return Err(err);
    }
    report.validated = true;

    let reloaded = env::run("umbriel", &["msg", "config-reload"])
        .map(|out| out.success)
        .unwrap_or(false);
    report.message = if reloaded {
        "fragmento aplicado y Umbriel recargado".into()
    } else {
        "fragmento escrito (Umbriel no está en ejecución; se aplicará al arrancar)".into()
    };

    Ok((report, content))
}

/// Estado del fragmento de Niri: (existe, está incluido en config.kdl).
pub fn niri_bridge_status() -> (bool, bool) {
    let bridge = niri_bridge_file();
    let config = niri_dir().join("config.kdl");
    let included = std::fs::read_to_string(config)
        .map(|text| text.contains("noct-bridge.kdl"))
        .unwrap_or(false);
    (bridge.exists(), included)
}

/// Estado del fragmento de Umbriel: (existe, está incluido en config.toml).
pub fn umbriel_bridge_status() -> (bool, bool) {
    let bridge = umbriel_bridge_file();
    let config = umbriel_dir().join("config.toml");
    let included = std::fs::read_to_string(config)
        .map(|text| text.contains("noct/bridge.toml"))
        .unwrap_or(false);
    (bridge.exists(), included)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Profile {
        Profile {
            meta: Meta {
                version: PROFILE_VERSION,
                source: "niri".into(),
                exported_at: 0,
            },
            keyboard: Some(Keyboard {
                layout: Some("es".into()),
                variant: None,
                model: None,
                options: Some("caps:escape".into()),
                numlock: Some(true),
                repeat_delay: Some(300),
                repeat_rate: Some(50),
            }),
            touchpad: Some(Touchpad {
                tap: Some(true),
                natural_scroll: Some(true),
                ..Touchpad::default()
            }),
            mouse: None,
            cursor: Some(Cursor {
                theme: Some("capitaine-cursors".into()),
                size: Some(24),
            }),
            focus: Some(Focus {
                follows_mouse: Some(true),
            }),
            layout: Some(Layout {
                gaps: Some(16),
                extent_presets: Some(vec![0.333, 0.5, 0.667]),
                default_extent_fraction: Some(0.5),
                center_focused: Some("never".into()),
            }),
            appearance: Some(Appearance {
                corner_radius: Some(20),
                border_width: None,
                prefer_no_csd: Some(true),
                blur: None,
            }),
            autostart: vec!["/usr/local/bin/noctalia".into()],
            outputs: vec![Output {
                name: "DP-1".into(),
                mode: Some("2560x1440@144".into()),
                scale: Some(1.0),
                ..Output::default()
            }],
            window_rules: vec![WindowRule {
                app_id: Some("steam".into()),
                floating: Some(true),
                ..WindowRule::default()
            }],
        }
    }

    #[test]
    fn niri_roundtrip() {
        let profile = sample();
        let text = profile.to_niri();
        let nodes = kdl::parse(&text);

        let input = nodes.iter().find(|n| n.name == "input").unwrap();
        let keyboard = input.child("keyboard").unwrap();
        let xkb = keyboard.child("xkb").unwrap();
        assert_eq!(
            xkb.child("layout").unwrap().arg(0).unwrap().as_str(),
            Some("es")
        );
        assert_eq!(
            keyboard
                .child("repeat-delay")
                .unwrap()
                .arg(0)
                .unwrap()
                .as_i64(),
            Some(300)
        );
        assert!(input.child("touchpad").unwrap().has_child("tap"));

        let layout = nodes.iter().find(|n| n.name == "layout").unwrap();
        assert_eq!(
            layout.child("gaps").unwrap().arg(0).unwrap().as_i64(),
            Some(16)
        );
        assert_eq!(
            layout.child("preset-column-widths").unwrap().children.len(),
            3
        );
    }

    #[test]
    fn umbriel_snippet_is_valid_toml() {
        let text = sample().to_umbriel();
        let value: toml::Value = toml::from_str(&text).unwrap();
        assert_eq!(value["input"]["keyboard"]["layout"].as_str(), Some("es"));
        assert_eq!(value["layout"]["gap"].as_integer(), Some(16));
        assert_eq!(
            value["window_rule"].as_array().unwrap()[0]["match"]["app_id"].as_str(),
            Some("steam")
        );
    }

    #[test]
    fn profile_serde_roundtrip() {
        let text = toml::to_string_pretty(&sample()).unwrap();
        let parsed: Profile = toml::from_str(&text).unwrap();
        assert_eq!(parsed.keyboard.unwrap().layout.as_deref(), Some("es"));
        assert_eq!(parsed.meta.version, PROFILE_VERSION);
    }
}
