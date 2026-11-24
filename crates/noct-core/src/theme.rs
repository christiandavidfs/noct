//! Modelo de tema de Noct y aplicación transaccional sobre Noctalia.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use serde::{Deserialize, Serialize};

use crate::env;

pub const SCHEMA_VERSION: u32 = 1;

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Convierte un nombre legible en un identificador estable.
pub fn slugify(input: &str) -> String {
    let mut out = String::new();
    let mut last_dash = false;
    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            last_dash = false;
        } else if !last_dash && !out.is_empty() {
            out.push('-');
            last_dash = true;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    if out.is_empty() {
        "noct".to_string()
    } else {
        out
    }
}

/// Los 74 roles de Material You, en variante dark y light.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PaletteSet {
    pub dark: BTreeMap<String, String>,
    pub light: BTreeMap<String, String>,
}

impl PaletteSet {
    pub fn role_count(&self) -> usize {
        self.dark.len()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Wallpapers {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub desktop: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mobile: Option<String>,
}

/// Tema canónico que comparten escritorio, greeter y Android.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub schema: u32,
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generated_at: Option<u64>,
    /// Preferencia configurada: auto, dark o light.
    pub mode: String,
    /// Modo resuelto por Noctalia en este momento.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_mode: Option<String>,
    pub source: String,
    pub scheme: String,
    pub wallpapers: Wallpapers,
    pub palette: PaletteSet,
}

impl Theme {
    /// Lee el tema activo de Noctalia y genera un bundle canónico.
    pub fn from_noctalia(name: Option<&str>, scheme: Option<&str>) -> Result<Self> {
        let wallpaper = env::capture("noctalia", &["msg", "wallpaper-get"])
            .filter(|s| !s.is_empty())
            .ok_or_else(|| anyhow!("Noctalia no está corriendo o no hay wallpaper activo"))?;

        let active = env::capture("noctalia", &["msg", "color-scheme-get"]).unwrap_or_default();
        let mut parts = active.split_whitespace();
        let source = parts.next().unwrap_or("wallpaper").to_string();
        let active_scheme = parts.next().unwrap_or("").to_string();

        let scheme = scheme
            .map(|s| s.to_string())
            .or_else(|| (!active_scheme.is_empty()).then_some(active_scheme))
            .unwrap_or_else(|| "m3-tonal-spot".to_string());

        let args = [
            "theme",
            wallpaper.as_str(),
            "--both",
            "--scheme",
            scheme.as_str(),
        ];
        let out = env::run("noctalia", &args)
            .ok_or_else(|| anyhow!("no se pudo ejecutar `noctalia theme`"))?;
        if !out.success {
            bail!("`noctalia theme` falló: {}", out.stderr);
        }

        let palette: PaletteSet = serde_json::from_str(&out.stdout)
            .context("`noctalia theme` no devolvió una paleta JSON válida")?;

        let mode = env::capture("noctalia", &["msg", "config-get", "theme.mode"])
            .filter(|s| !s.is_empty())
            .map(|s| s.trim_matches(|c| c == '\'' || c == '"').to_string())
            .unwrap_or_else(|| "auto".to_string());
        let resolved_mode =
            env::capture("noctalia", &["msg", "theme-mode-get"]).filter(|s| !s.is_empty());

        let name = name
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("Noct {scheme}"));

        Ok(Self {
            schema: SCHEMA_VERSION,
            id: format!("noct/{}", slugify(&name)),
            name,
            description: None,
            generated_at: Some(now()),
            mode,
            resolved_mode,
            source,
            scheme,
            wallpapers: Wallpapers {
                desktop: Some(wallpaper),
                mobile: None,
            },
            palette,
        })
    }

    pub fn load(path: &Path) -> Result<Self> {
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("no se pudo leer {}", path.display()))?;
        let theme: Self = serde_json::from_str(&text)
            .with_context(|| format!("{} no es un tema de Noct válido", path.display()))?;
        if theme.schema != SCHEMA_VERSION {
            bail!(
                "el tema usa el esquema {} y Noct espera {}",
                theme.schema,
                SCHEMA_VERSION
            );
        }
        Ok(theme)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("no se pudo crear {}", parent.display()))?;
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, format!("{json}\n"))
            .with_context(|| format!("no se pudo escribir {}", path.display()))
    }

    pub fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    /// Convierte rutas relativas de wallpaper en absolutas respecto a `base`.
    pub fn resolve_paths(&mut self, base: &Path) {
        for slot in [&mut self.wallpapers.desktop, &mut self.wallpapers.mobile] {
            if let Some(value) = slot {
                let path = Path::new(value);
                if path.is_relative() {
                    *slot = Some(base.join(path).display().to_string());
                }
            }
        }
    }

    /// Modo a usar al renderizar plantillas: el resuelto por Noctalia.
    pub fn render_mode(&self) -> &str {
        match self.resolved_mode.as_deref() {
            Some("dark") => "dark",
            Some("light") => "light",
            _ => match self.mode.as_str() {
                "dark" => "dark",
                "light" => "light",
                _ => "dark",
            },
        }
    }

    /// Aplica el tema sobre Noctalia. Con `dry_run` solo describe los pasos.
    pub fn apply(&self, opts: &ApplyOptions) -> Result<Vec<Step>> {
        let mut steps = Vec::new();

        // 1. Persistir la selección de paleta.
        exec_step(
            &mut steps,
            opts.dry_run,
            "Seleccionar paleta",
            "noctalia",
            &[
                "msg",
                "color-scheme-set",
                self.source.as_str(),
                self.scheme.as_str(),
            ],
        )?;

        // 2. Renderizar plantillas desde el wallpaper y el esquema.
        //    `templates-apply` reutiliza la paleta en memoria y no reacciona a
        //    `color-scheme-set`; `noctalia theme --builtin-config` sí lo hace.
        if self.source == "wallpaper" {
            match self.wallpapers.desktop.as_deref() {
                Some(desktop) if Path::new(desktop).exists() => {
                    exec_step(
                        &mut steps,
                        opts.dry_run,
                        "Renderizar plantillas del tema",
                        "noctalia",
                        &[
                            "theme",
                            desktop,
                            "--scheme",
                            self.scheme.as_str(),
                            "--builtin-config",
                            "--default-mode",
                            self.render_mode(),
                        ],
                    )?;
                }
                Some(desktop) => steps.push(Step {
                    description: "Renderizar plantillas del tema".to_string(),
                    command: None,
                    executed: false,
                    ok: false,
                    output: Some(format!("no existe el wallpaper: {desktop}")),
                }),
                None => {}
            }
        }

        // 3. Compatibilidad de claves con el Umbriel instalado.
        compat_step(&mut steps, opts.dry_run);

        // 4. Hacer que el shell recargue la configuración y la paleta.
        exec_step(
            &mut steps,
            opts.dry_run,
            "Recargar Noctalia",
            "noctalia",
            &["msg", "config-reload"],
        )?;

        // 5. Persistir el wallpaper (shell, greeter y sesión).
        if self.source == "wallpaper" {
            if let Some(desktop) = self.wallpapers.desktop.as_deref() {
                if Path::new(desktop).exists() {
                    exec_step(
                        &mut steps,
                        opts.dry_run,
                        "Aplicar wallpaper",
                        "noctalia",
                        &["msg", "wallpaper-set", desktop],
                    )?;
                }
            }
        }

        // 6. Red de seguridad para plantillas community/custom.
        exec_step(
            &mut steps,
            opts.dry_run,
            "Aplicar plantillas restantes",
            "noctalia",
            &["msg", "templates-apply"],
        )?;

        // 7. `templates-apply` vuelve a escribir claves con el nombre antiguo;
        //    se corrigen otra vez para que la config quede válida al reiniciar.
        compat_step(&mut steps, opts.dry_run);

        if opts.greeter {
            exec_step(
                &mut steps,
                opts.dry_run,
                "Sincronizar greeter",
                "noctalia",
                &["msg", "greeter-sync"],
            )?;
        }

        let target = env::app_config("noct").join("current.json");
        if opts.dry_run {
            steps.push(Step {
                description: "Guardar tema activo".to_string(),
                command: Some(format!("write {}", target.display())),
                executed: false,
                ok: true,
                output: None,
            });
        } else {
            self.save(&target)?;
            steps.push(Step {
                description: "Guardar tema activo".to_string(),
                command: Some(format!("write {}", target.display())),
                executed: true,
                ok: true,
                output: None,
            });
        }

        Ok(steps)
    }
}

#[derive(Debug, Clone)]
pub struct ApplyOptions {
    pub dry_run: bool,
    pub greeter: bool,
}

impl Default for ApplyOptions {
    fn default() -> Self {
        Self {
            dry_run: false,
            greeter: true,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Step {
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    pub executed: bool,
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
}

fn exec_step(
    steps: &mut Vec<Step>,
    dry_run: bool,
    description: &str,
    cmd: &str,
    args: &[&str],
) -> Result<()> {
    let rendered = if args.is_empty() {
        cmd.to_string()
    } else {
        format!("{cmd} {}", args.join(" "))
    };

    if dry_run {
        steps.push(Step {
            description: description.to_string(),
            command: Some(rendered),
            executed: false,
            ok: true,
            output: None,
        });
        return Ok(());
    }

    let out = env::run(cmd, args)
        .ok_or_else(|| anyhow!("no se pudo ejecutar `{cmd}` (¿está en PATH?)"))?;
    let ok = out.success;
    let output = if ok { out.stdout } else { out.stderr };
    steps.push(Step {
        description: description.to_string(),
        command: Some(rendered),
        executed: true,
        ok,
        output: if output.is_empty() {
            None
        } else {
            Some(output)
        },
    });

    if !ok {
        bail!("falló `{cmd} {}`", args.join(" "));
    }
    Ok(())
}

fn compat_step(steps: &mut Vec<Step>, dry_run: bool) {
    if dry_run {
        return;
    }
    match crate::config::normalize_umbriel_theme() {
        Ok(Some(message)) => steps.push(Step {
            description: "Compatibilidad con Umbriel".to_string(),
            command: Some(message),
            executed: true,
            ok: true,
            output: None,
        }),
        Ok(None) => {}
        Err(err) => steps.push(Step {
            description: "Compatibilidad con Umbriel".to_string(),
            command: None,
            executed: true,
            ok: false,
            output: Some(format!("{err:#}")),
        }),
    }
}

/// Estado de Noctalia antes de aplicar un tema, para poder deshacer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Backup {
    pub created_at: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wallpaper: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color_scheme: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme_mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_mode: Option<String>,
}

impl Backup {
    pub fn snapshot() -> Self {
        Self {
            created_at: now(),
            wallpaper: env::capture("noctalia", &["msg", "wallpaper-get"])
                .filter(|s| !s.is_empty()),
            color_scheme: env::capture("noctalia", &["msg", "color-scheme-get"])
                .filter(|s| !s.is_empty()),
            theme_mode: env::capture("noctalia", &["msg", "config-get", "theme.mode"])
                .map(|s| s.trim_matches(|c| c == '\'' || c == '"').to_string())
                .filter(|s| !s.is_empty()),
            resolved_mode: env::capture("noctalia", &["msg", "theme-mode-get"])
                .filter(|s| !s.is_empty()),
        }
    }

    pub fn save(&self) -> Result<PathBuf> {
        let dir = backup_dir();
        std::fs::create_dir_all(&dir)
            .with_context(|| format!("no se pudo crear {}", dir.display()))?;
        let path = dir.join(format!("{}.json", self.created_at));
        std::fs::write(&path, format!("{}\n", serde_json::to_string_pretty(self)?))
            .with_context(|| format!("no se pudo escribir {}", path.display()))?;
        Ok(path)
    }

    pub fn load_latest() -> Result<Option<Self>> {
        let dir = backup_dir();
        let Ok(entries) = std::fs::read_dir(&dir) else {
            return Ok(None);
        };
        let mut files: Vec<PathBuf> = entries
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("json"))
            .collect();
        files.sort();
        let Some(latest) = files.last() else {
            return Ok(None);
        };
        let text = std::fs::read_to_string(latest)?;
        Ok(serde_json::from_str(&text).ok())
    }

    /// Devuelve el estado guardado a Noctalia.
    pub fn restore(&self) -> Result<Vec<Step>> {
        let mut steps = Vec::new();

        if let Some(mode) = &self.theme_mode {
            exec_step(
                &mut steps,
                false,
                "Restaurar modo de tema",
                "noctalia",
                &["msg", "theme-mode-set", mode],
            )?;
        }

        let mut scheme_pair = None;
        if let Some(scheme) = &self.color_scheme {
            let mut parts = scheme.split_whitespace();
            if let (Some(source), Some(name)) = (parts.next(), parts.next()) {
                scheme_pair = Some((source.to_string(), name.to_string()));
                exec_step(
                    &mut steps,
                    false,
                    "Restaurar paleta",
                    "noctalia",
                    &["msg", "color-scheme-set", source, name],
                )?;
            }
        }

        if let (Some(wallpaper), Some((source, name))) = (&self.wallpaper, &scheme_pair) {
            if source == "wallpaper" && Path::new(wallpaper).exists() {
                let mode = self.resolved_mode.as_deref().unwrap_or("dark");
                exec_step(
                    &mut steps,
                    false,
                    "Renderizar plantillas restauradas",
                    "noctalia",
                    &[
                        "theme",
                        wallpaper,
                        "--scheme",
                        name,
                        "--builtin-config",
                        "--default-mode",
                        mode,
                    ],
                )?;
            }
        }

        exec_step(
            &mut steps,
            false,
            "Recargar Noctalia",
            "noctalia",
            &["msg", "config-reload"],
        )?;

        if let Some(wallpaper) = &self.wallpaper {
            exec_step(
                &mut steps,
                false,
                "Restaurar wallpaper",
                "noctalia",
                &["msg", "wallpaper-set", wallpaper],
            )?;
        }

        exec_step(
            &mut steps,
            false,
            "Restaurar plantillas restantes",
            "noctalia",
            &["msg", "templates-apply"],
        )?;

        compat_step(&mut steps, false);

        Ok(steps)
    }
}

pub fn backup_dir() -> PathBuf {
    env::app_config("noct").join("backups")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugify_handles_spaces_and_symbols() {
        assert_eq!(slugify("Noct Umbral"), "noct-umbral");
        assert_eq!(slugify("  M3 / Mono  "), "m3-mono");
        assert_eq!(slugify("!!!"), "noct");
    }
}
