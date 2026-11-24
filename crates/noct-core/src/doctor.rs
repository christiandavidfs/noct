//! `noct doctor`: diagnóstico del escritorio y de su configuración.

use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::env::{self, Compositor};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Ok,
    Info,
    Warn,
    Error,
}

impl Severity {
    pub fn label(self) -> &'static str {
        match self {
            Severity::Ok => "ok",
            Severity::Info => "info",
            Severity::Warn => "warn",
            Severity::Error => "error",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Check {
    pub id: String,
    pub title: String,
    pub severity: Severity,
    pub detail: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SystemSummary {
    pub session_desktop: Option<String>,
    pub wayland_display: Option<String>,
    pub compositor: String,
    pub noctalia_running: bool,
    pub noctalia_pids: Vec<u32>,
    pub noctalia_exe: Option<String>,
    pub noctalia_version: Option<String>,
    pub noctalia_shell_installed: bool,
    pub niri_version: Option<String>,
    pub umbriel_version: Option<String>,
    pub greeter_command: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct Summary {
    pub ok: usize,
    pub info: usize,
    pub warn: usize,
    pub error: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct Report {
    pub generated_at: u64,
    pub system: SystemSummary,
    pub checks: Vec<Check>,
    pub summary: Summary,
}

impl Report {
    pub fn exit_code(&self) -> i32 {
        if self.summary.error > 0 {
            1
        } else {
            0
        }
    }
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn check(
    id: &str,
    title: &str,
    severity: Severity,
    detail: impl Into<String>,
    fix: Option<&str>,
) -> Check {
    Check {
        id: id.to_string(),
        title: title.to_string(),
        severity,
        detail: detail.into(),
        fix: fix.map(|s| s.to_string()),
    }
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

/// Cuenta las apariciones del comando obsoleto del shell Quickshell.
fn legacy_shell_hits(files: &[PathBuf]) -> Vec<(PathBuf, usize, usize)> {
    let mut hits = Vec::new();
    for file in files {
        let Ok(text) = std::fs::read_to_string(file) else {
            continue;
        };
        let mut count = 0;
        for line in text.lines() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') || trimmed.starts_with("//") {
                continue;
            }
            if line.contains("qs -c noctalia-shell") {
                count += 1;
            }
        }
        if count > 0 {
            hits.push((file.clone(), count, text.lines().count()));
        }
    }
    hits
}

fn greeter_command() -> Option<String> {
    let text = std::fs::read_to_string("/etc/greetd/config.toml").ok()?;
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("command") {
            if let Some(eq) = rest.find('=') {
                return Some(rest[eq + 1..].trim().trim_matches('"').to_string());
            }
        }
    }
    None
}

/// Archivos que Noctalia reescribe al aplicar plantillas de tema.
pub fn template_outputs() -> Vec<PathBuf> {
    vec![
        env::app_config("umbriel").join("noctalia.toml"),
        env::app_config("gtk-3.0").join("gtk.css"),
        env::app_config("kitty").join("noctalia.conf"),
        env::app_config("ghostty").join("themes").join("noctalia"),
    ]
}

/// Devuelve los archivos generados que no se pueden reescribir.
pub fn readonly_template_outputs() -> Vec<PathBuf> {
    template_outputs()
        .into_iter()
        .filter(|path| {
            path.exists()
                && path
                    .metadata()
                    .map(|m| m.permissions().readonly())
                    .unwrap_or(false)
        })
        .collect()
}

/// Añade permiso de escritura a las plantillas generadas. Devuelve las rutas corregidas.
#[cfg(unix)]
pub fn fix_template_permissions() -> Vec<String> {
    use std::os::unix::fs::PermissionsExt;

    let mut fixed = Vec::new();
    for path in readonly_template_outputs() {
        let Ok(metadata) = path.metadata() else {
            continue;
        };
        let mut permissions = metadata.permissions();
        permissions.set_mode(permissions.mode() | 0o200);
        if std::fs::set_permissions(&path, permissions).is_ok() {
            fixed.push(path.display().to_string());
        }
    }
    fixed
}

/// Ejecuta todas las comprobaciones del sistema.
pub fn run() -> Report {
    let mut checks = Vec::new();

    let noctalia_bins = env::find_all("noctalia");
    let noctalia_pids = env::running_pids("noctalia");
    let noctalia_exe = noctalia_pids
        .first()
        .and_then(|pid| env::pid_exe(*pid))
        .map(|p| p.display().to_string());
    let noctalia_version = env::capture("noctalia", &["--version"]);
    let noctalia_running = !noctalia_pids.is_empty();

    let compositor = env::detect_compositor();
    let niri_version = env::capture("niri", &["msg", "version"])
        .and_then(|s| s.lines().next().map(|l| l.to_string()));
    let umbriel_version =
        env::capture("umbriel", &["--version"]).or_else(|| env::capture("umbriel", &["version"]));

    let shell_installed =
        env::find("qs").is_some() && Path::new("/etc/xdg/quickshell/noctalia-shell").exists();
    let greeter_command = greeter_command();

    let system = SystemSummary {
        session_desktop: std::env::var("XDG_CURRENT_DESKTOP").ok(),
        wayland_display: std::env::var("WAYLAND_DISPLAY").ok(),
        compositor: compositor.as_str().to_string(),
        noctalia_running,
        noctalia_pids: noctalia_pids.clone(),
        noctalia_exe: noctalia_exe.clone(),
        noctalia_version: noctalia_version.clone(),
        noctalia_shell_installed: shell_installed,
        niri_version,
        umbriel_version,
        greeter_command: greeter_command.clone(),
    };

    // 1. Binarios de Noctalia duplicados.
    if noctalia_bins.len() > 1 {
        let list = noctalia_bins
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
            .join(", ");
        checks.push(check(
            "noctalia.binaries",
            "Binarios de Noctalia duplicados",
            Severity::Warn,
            format!("Se encontraron {} binarios: {list}", noctalia_bins.len()),
            Some("Deja una sola instalación en PATH o fija la ruta en ~/.config/noct/noct.toml"),
        ));
    } else if noctalia_bins.is_empty() {
        checks.push(check(
            "noctalia.binaries",
            "Noctalia no está instalado",
            Severity::Error,
            "No se encontró `noctalia` en PATH.",
            Some("Instala Noctalia o añade su binario al PATH"),
        ));
    } else {
        checks.push(check(
            "noctalia.binaries",
            "Binario de Noctalia",
            Severity::Ok,
            noctalia_bins[0].display().to_string(),
            None,
        ));
    }

    // 2. Instancia en ejecución.
    if noctalia_running {
        let exe = noctalia_exe.clone().unwrap_or_else(|| "desconocido".into());
        checks.push(check(
            "noctalia.running",
            "Noctalia en ejecución",
            Severity::Ok,
            format!("PID(s) {:?} · {exe}", noctalia_pids),
            None,
        ));
    } else {
        checks.push(check(
            "noctalia.running",
            "Noctalia no está corriendo",
            Severity::Error,
            "Sin instancia activa, `noct theme apply` no puede aplicar la paleta ni el wallpaper.",
            Some("Arranca Noctalia desde la sesión del compositor"),
        ));
    }

    // 3. Shell Quickshell heredado.
    if shell_installed {
        checks.push(check(
            "noctalia.quickshell",
            "Noctalia Shell (Quickshell) instalado",
            Severity::Info,
            "El paquete `noctalia-shell` sigue presente. El shell activo es el binario nativo `noctalia`.",
            Some("Si no lo usas, desinstala `noctalia-shell` para evitar confusiones"),
        ));
    }

    // 4. Config heredada en JSON.
    let legacy_settings = env::app_config("noctalia").join("settings.json");
    if legacy_settings.exists() {
        checks.push(check(
            "noctalia.legacy-config",
            "Configuración heredada en JSON",
            Severity::Info,
            format!(
                "{} existe; el shell nativo usa settings.toml.",
                legacy_settings.display()
            ),
            Some("Archiva o elimina settings.json si ya no usas el shell Quickshell"),
        ));
    }

    // 5. Atajos con el comando obsoleto `qs -c noctalia-shell`.
    let mut config_files = Vec::new();
    collect_files(&env::app_config("umbriel"), &["toml"], 3, &mut config_files);
    collect_files(&env::app_config("niri"), &["kdl"], 3, &mut config_files);
    let hits = legacy_shell_hits(&config_files);
    if !hits.is_empty() {
        let total: usize = hits.iter().map(|(_, count, _)| count).sum();
        let files = hits
            .iter()
            .map(|(path, count, _)| format!("{} ({count})", path.display()))
            .collect::<Vec<_>>()
            .join(", ");
        checks.push(check(
            "keybinds.legacy-shell",
            "Atajos apuntan al shell Quickshell",
            Severity::Warn,
            format!("{total} llamadas a `qs -c noctalia-shell`: {files}"),
            Some("Ejecuta `noct keys migrate --apply` para migrarlas a `noctalia msg`"),
        ));
    } else if !config_files.is_empty() {
        checks.push(check(
            "keybinds.legacy-shell",
            "Atajos sin comandos obsoletos",
            Severity::Ok,
            "No se encontraron llamadas a `qs -c noctalia-shell`.",
            None,
        ));
    }

    // 6. Config de Umbriel e include de Noctalia.
    let umbriel_config = env::app_config("umbriel").join("config.toml");
    if umbriel_config.exists() {
        let text = std::fs::read_to_string(&umbriel_config).unwrap_or_default();
        let has_include = text.contains("noctalia.toml");
        checks.push(check(
            "umbriel.include",
            "Include de Noctalia en Umbriel",
            if has_include {
                Severity::Ok
            } else {
                Severity::Warn
            },
            if has_include {
                "config.toml incluye noctalia.toml (tema generado por Noctalia).".to_string()
            } else {
                "config.toml no incluye noctalia.toml; el tema no llegará a Umbriel.".to_string()
            },
            if has_include {
                None
            } else {
                Some("Añade noctalia.toml a [include.optional] o usa `noct theme apply`")
            },
        ));

        let effects_active = text
            .lines()
            .any(|l| l.trim() == "[effects]" || l.trim().starts_with("[effects."));
        checks.push(check(
            "umbriel.effects",
            "Efectos GLSL de Umbriel",
            if effects_active {
                Severity::Ok
            } else {
                Severity::Info
            },
            if effects_active {
                "Hay una sección [effects] activa.".to_string()
            } else {
                "Los efectos están desactivados o comentados.".to_string()
            },
            if effects_active {
                None
            } else {
                Some("Prueba `noct theme apply` con un tema que incluya presets de efectos")
            },
        ));

        let validate = env::run("umbriel", &["config", "validate"]);
        match validate {
            Some(out) if out.success => checks.push(check(
                "umbriel.validate",
                "Configuración de Umbriel válida",
                Severity::Ok,
                out.stdout,
                None,
            )),
            Some(out) => checks.push(check(
                "umbriel.validate",
                "Configuración de Umbriel inválida",
                Severity::Error,
                out.stderr,
                Some("Corrige el error antes de recargar el compositor"),
            )),
            None => checks.push(check(
                "umbriel.validate",
                "No se pudo validar Umbriel",
                Severity::Warn,
                "`umbriel config validate` no está disponible.",
                None,
            )),
        }

        if let Some(issue) = crate::config::duplicate_includes() {
            checks.push(check(
                "umbriel.includes",
                "Includes duplicados",
                Severity::Warn,
                issue,
                Some("Ejecuta `noct doctor --fix` para deduplicar"),
            ));
        } else {
            checks.push(check(
                "umbriel.includes",
                "Includes sin duplicados",
                Severity::Ok,
                "Las rutas de include son únicas.",
                None,
            ));
        }

        if let Some(issue) = crate::config::umbriel_theme_issue() {
            checks.push(check(
                "umbriel.theme-compat",
                "Tema de Umbriel incompatible",
                Severity::Warn,
                issue,
                Some("Ejecuta `noct doctor --fix` para renombrar las claves"),
            ));
        } else {
            checks.push(check(
                "umbriel.theme-compat",
                "Tema de Umbriel compatible",
                Severity::Ok,
                "Las claves del tema generado existen en el esquema instalado.",
                None,
            ));
        }
    } else {
        checks.push(check(
            "umbriel.config",
            "Sin configuración de Umbriel",
            Severity::Info,
            format!("{} no existe.", umbriel_config.display()),
            Some("Copia la configuración de ejemplo o ejecuta Umbriel una vez"),
        ));
    }

    // 7. Plantillas generadas: deben poder reescribirse.
    let readonly: Vec<String> = readonly_template_outputs()
        .iter()
        .map(|path| path.display().to_string())
        .collect();
    if readonly.is_empty() {
        checks.push(check(
            "templates.writable",
            "Plantillas generadas escribibles",
            Severity::Ok,
            "Noctalia puede actualizar los archivos de tema de las apps.",
            None,
        ));
    } else {
        checks.push(check(
            "templates.writable",
            "Plantillas generadas de solo lectura",
            Severity::Warn,
            format!(
                "Estos archivos no se pueden reescribir: {}. El tema dejará de actualizarse.",
                readonly.join(", ")
            ),
            Some("Ejecuta `chmod u+w` sobre esos archivos"),
        ));
    }

    // 8. Compositor activo.
    match compositor {
        Compositor::Umbriel => checks.push(check(
            "compositor.running",
            "Compositor Umbriel activo",
            Severity::Ok,
            "La sesión actual es Umbriel; los efectos y atajos se pueden aplicar en vivo.",
            None,
        )),
        Compositor::Niri => checks.push(check(
            "compositor.running",
            "Sesión Niri activa",
            Severity::Info,
            "Noct puede gestionar Niri, pero los shaders por ventana requieren Umbriel.",
            Some("Prueba Umbriel en una sesión anidada antes de migrar"),
        )),
        Compositor::Other => checks.push(check(
            "compositor.running",
            "Compositor no reconocido",
            Severity::Warn,
            "No se detectó Niri ni Umbriel.",
            None,
        )),
        Compositor::None => checks.push(check(
            "compositor.running",
            "Sin compositor Wayland",
            Severity::Warn,
            "No hay sesión Wayland activa.",
            None,
        )),
    }

    // 9. Greeter.
    match &greeter_command {
        Some(cmd) if cmd.contains("noctalia-greeter-session") => checks.push(check(
            "greeter.session",
            "Greeter de Noctalia configurado",
            Severity::Ok,
            cmd.clone(),
            None,
        )),
        Some(cmd) => checks.push(check(
            "greeter.session",
            "Greeter distinto de Noctalia",
            Severity::Info,
            cmd.clone(),
            None,
        )),
        None => checks.push(check(
            "greeter.session",
            "Sin configuración de greetd",
            Severity::Info,
            "/etc/greetd/config.toml no existe o no define command.",
            None,
        )),
    }

    // 10. Config propia de Noct.
    let noct_dir = env::app_config("noct");
    checks.push(check(
        "noct.config",
        "Configuración de Noct",
        if noct_dir.exists() {
            Severity::Ok
        } else {
            Severity::Info
        },
        if noct_dir.exists() {
            format!("{}", noct_dir.display())
        } else {
            "Todavía no hay ~/.config/noct; se creará al aplicar un tema.".to_string()
        },
        None,
    ));

    // 11. Puente de configuración Niri ↔ Umbriel.
    let profile_file = crate::profile::profile_path();
    if profile_file.exists() {
        match crate::profile::Profile::load(&profile_file) {
            Ok(profile) => checks.push(check(
                "bridge.profile",
                "Perfil canónico disponible",
                Severity::Ok,
                format!(
                    "{} · origen: {} · {} regla(s) de ventana",
                    profile_file.display(),
                    if profile.meta.source.is_empty() {
                        "?"
                    } else {
                        &profile.meta.source
                    },
                    profile.window_rules.len()
                ),
                None,
            )),
            Err(err) => checks.push(check(
                "bridge.profile",
                "Perfil canónico inválido",
                Severity::Warn,
                format!("{err:#}"),
                Some("Vuelve a exportar con `noct bridge export`"),
            )),
        }
    } else {
        checks.push(check(
            "bridge.profile",
            "Sin perfil canónico",
            Severity::Info,
            "Todavía no se ha exportado la configuración para el puente.",
            Some("Ejecuta `noct bridge export`"),
        ));
    }

    let (niri_exists, niri_included) = crate::profile::niri_bridge_status();
    checks.push(check(
        "bridge.niri",
        "Fragmento de Niri",
        if niri_exists && niri_included {
            Severity::Ok
        } else if niri_exists || niri_included {
            Severity::Warn
        } else {
            Severity::Info
        },
        match (niri_exists, niri_included) {
            (true, true) => "Fragmento presente e incluido en config.kdl.".to_string(),
            (true, false) => "El fragmento existe pero config.kdl no lo incluye.".to_string(),
            (false, true) => {
                "config.kdl incluye noct-bridge.kdl pero el archivo no existe.".to_string()
            }
            (false, false) => "Aún no se ha aplicado el perfil a Niri.".to_string(),
        },
        Some("Ejecuta `noct bridge apply --to niri`"),
    ));

    let (umbriel_exists, umbriel_included) = crate::profile::umbriel_bridge_status();
    checks.push(check(
        "bridge.umbriel",
        "Fragmento de Umbriel",
        if umbriel_exists && umbriel_included {
            Severity::Ok
        } else if umbriel_exists || umbriel_included {
            Severity::Warn
        } else {
            Severity::Info
        },
        match (umbriel_exists, umbriel_included) {
            (true, true) => "Fragmento presente e incluido en config.toml.".to_string(),
            (true, false) => "El fragmento existe pero config.toml no lo incluye.".to_string(),
            (false, true) => {
                "config.toml incluye noct/bridge.toml pero el archivo no existe.".to_string()
            }
            (false, false) => "Aún no se ha aplicado el perfil a Umbriel.".to_string(),
        },
        Some("Ejecuta `noct bridge apply --to umbriel`"),
    ));

    let mut summary = Summary::default();
    for item in &checks {
        match item.severity {
            Severity::Ok => summary.ok += 1,
            Severity::Info => summary.info += 1,
            Severity::Warn => summary.warn += 1,
            Severity::Error => summary.error += 1,
        }
    }

    Report {
        generated_at: now(),
        system,
        checks,
        summary,
    }
}
