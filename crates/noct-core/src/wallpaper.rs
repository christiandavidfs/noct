//! Modos de wallpaper: fijo (protegido) y Bing diario con sincronización del greeter.

use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

use crate::env;

pub const BING_URL: &str = "https://www.bing.com/HPImageArchive.aspx?format=js&idx=0&n=1&mkt=en-US";
const BING_PREFIX: &str = "https://www.bing.com";
const OLD_DAILY_PLUGIN: &str = "nzlov/daily-wallpaper";
const BING_TIMER: &str = "noct-bing-wallpaper.timer";
const GREETER_TIMER: &str = "noct-wallpaper-greeter.timer";

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WallpaperMode {
    /// El wallpaper elegido no se toca hasta que el usuario lo cambie.
    #[default]
    Fixed,
    /// Noct descarga el Bing del día y actualiza escritorio y greeter.
    BingDaily,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WallpaperConfig {
    #[serde(default)]
    pub mode: WallpaperMode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_bing: Option<String>,
    #[serde(default)]
    pub updated_at: u64,
}

impl Default for WallpaperConfig {
    fn default() -> Self {
        Self {
            mode: WallpaperMode::Fixed,
            path: None,
            last_bing: None,
            updated_at: now(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct WallpaperStatus {
    pub mode: WallpaperMode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_bing: Option<String>,
    pub bing_timer: bool,
    pub greeter_timer: bool,
    pub daily_plugin: bool,
    pub applied: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    pub config_file: String,
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn config_path() -> PathBuf {
    env::app_config("noct").join("wallpaper.toml")
}

fn state_path() -> PathBuf {
    env::app_state("noct").join("greeter-wallpaper")
}

fn wallpapers_dir() -> PathBuf {
    env::data_home().join("noct").join("wallpapers")
}

fn systemd_dir() -> PathBuf {
    env::config_home().join("systemd").join("user")
}

/// Wallpaper activo según Noctalia.
pub fn current_wallpaper() -> Option<String> {
    env::capture("noctalia", &["msg", "wallpaper-get"])
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

/// Carga la configuración; si no existe, propone el wallpaper actual como fijo.
pub fn load_config() -> WallpaperConfig {
    let path = config_path();
    let mut config = std::fs::read_to_string(&path)
        .ok()
        .and_then(|text| toml::from_str::<WallpaperConfig>(&text).ok())
        .unwrap_or_default();
    if config.path.is_none() {
        config.path = current_wallpaper();
    }
    config
}

fn save_config(config: &WallpaperConfig) -> Result<()> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("no se pudo crear {}", parent.display()))?;
    }
    let mut config = config.clone();
    config.updated_at = now();
    let text = toml::to_string_pretty(&config)
        .with_context(|| format!("no se pudo serializar {}", path.display()))?;
    std::fs::write(&path, text).with_context(|| format!("no se pudo escribir {}", path.display()))
}

fn is_enabled(unit: &str) -> bool {
    env::capture("systemctl", &["--user", "is-enabled", unit])
        .map(|value| value.trim() == "enabled" || value.trim() == "enabled-runtime")
        .unwrap_or(false)
}

fn daily_plugin_enabled() -> bool {
    env::capture("noctalia", &["msg", "plugins", "list"])
        .map(|text| {
            text.lines()
                .any(|line| line.contains(OLD_DAILY_PLUGIN) && line.trim_end().ends_with("enabled"))
        })
        .unwrap_or(false)
}

fn set_daily_plugin(enabled: bool) -> bool {
    let action = if enabled { "enable" } else { "disable" };
    env::run("noctalia", &["msg", "plugins", action, OLD_DAILY_PLUGIN])
        .map(|out| out.success)
        .unwrap_or(false)
}

fn set_bing_timer(enabled: bool) -> bool {
    let action = if enabled { "enable" } else { "disable" };
    let _ = env::run("systemctl", &["--user", action, "--now", BING_TIMER]);
    is_enabled(BING_TIMER)
}

fn ensure_greeter_timer() {
    let _ = env::run("systemctl", &["--user", "enable", "--now", GREETER_TIMER]);
}

/// Aplica el wallpaper al shell y sincroniza el greeter.
pub fn apply(path: &Path) -> Result<()> {
    let path = path
        .canonicalize()
        .with_context(|| format!("no existe el wallpaper {}", path.display()))?;
    let path = path.display().to_string();
    let out = env::run("noctalia", &["msg", "wallpaper-set", &path])
        .context("no se pudo ejecutar noctalia")?;
    if !out.success {
        bail!("noctalia wallpaper-set falló: {}", out.stderr);
    }
    let _ = env::run("noctalia", &["msg", "greeter-sync"]);
    Ok(())
}

fn finish(mut status: WallpaperStatus, applied: Result<()>) -> Result<WallpaperStatus> {
    if let Err(err) = applied {
        status.applied = false;
        status.message = Some(format!("{err:#}"));
    }
    Ok(status)
}

/// Fija un wallpaper y desactiva cualquier actualización automática.
pub fn set_fixed(path: Option<&str>) -> Result<WallpaperStatus> {
    let chosen = match path {
        Some(value) => value.to_string(),
        None => current_wallpaper().context(
            "no hay un wallpaper activo; indica una ruta con `noct wallpaper fixed <ruta>`",
        )?,
    };
    let chosen_path = PathBuf::from(&chosen)
        .canonicalize()
        .with_context(|| format!("no existe el wallpaper {chosen}"))?;

    ensure_units()?;
    let mut config = load_config();
    config.mode = WallpaperMode::Fixed;
    config.path = Some(chosen_path.display().to_string());
    config.last_bing = None;
    save_config(&config)?;

    set_daily_plugin(false);
    set_bing_timer(false);
    ensure_greeter_timer();
    finish(status(), apply(&chosen_path))
}

/// Descarga el Bing del día y lo aplica sin activar la rotación diaria.
pub fn bing_once() -> Result<WallpaperStatus> {
    let path = fetch_bing()?;
    ensure_units()?;
    let mut config = load_config();
    config.mode = WallpaperMode::Fixed;
    config.path = Some(path.display().to_string());
    config.last_bing = Some(path.display().to_string());
    save_config(&config)?;
    set_daily_plugin(false);
    set_bing_timer(false);
    ensure_greeter_timer();
    finish(status(), apply(&path))
}

/// Activa Bing diario: descarga el de hoy, instala los timers y desactiva
/// el plugin antiguo para que no compita por el wallpaper.
pub fn set_bing_daily() -> Result<WallpaperStatus> {
    ensure_units()?;
    let path = fetch_bing()?;
    let mut config = load_config();
    config.mode = WallpaperMode::BingDaily;
    config.path = None;
    config.last_bing = Some(path.display().to_string());
    save_config(&config)?;

    set_daily_plugin(false);
    let _ = env::run("systemctl", &["--user", "daemon-reload"]);
    ensure_greeter_timer();
    set_bing_timer(true);
    finish(status(), apply(&path))
}

/// Ejecuta el Bing del día. Si el modo es Bing diario, mantiene la rotación;
/// si no, lo fija como wallpaper estático.
pub fn run_bing() -> Result<WallpaperStatus> {
    let config = load_config();
    if config.mode == WallpaperMode::BingDaily {
        let path = fetch_bing()?;
        let mut config = config;
        config.last_bing = Some(path.display().to_string());
        save_config(&config)?;
        finish(status(), apply(&path))
    } else {
        bing_once()
    }
}

/// Descarga la imagen del día de Bing y devuelve la ruta local.
pub fn fetch_bing() -> Result<PathBuf> {
    let out = env::run("curl", &["-fsSL", BING_URL])
        .context("no se pudo ejecutar curl para consultar Bing")?;
    if !out.success {
        bail!("Bing no respondió: {}", out.stderr);
    }
    let value: serde_json::Value =
        serde_json::from_str(&out.stdout).context("Bing no devolvió un JSON válido")?;
    let image = value
        .get("images")
        .and_then(|images| images.get(0))
        .context("Bing no devolvió ninguna imagen")?;
    let url = image
        .get("url")
        .and_then(|value| value.as_str())
        .context("Bing no devolvió la URL de la imagen")?;
    let date = image
        .get("startdate")
        .and_then(|value| value.as_str())
        .unwrap_or("today");
    let full = if url.starts_with("http") {
        url.to_string()
    } else {
        format!("{BING_PREFIX}{url}")
    };

    let dir = wallpapers_dir();
    std::fs::create_dir_all(&dir).with_context(|| format!("no se pudo crear {}", dir.display()))?;
    let path = dir.join(format!("bing-{date}.jpg"));
    let out = env::run("curl", &["-fsSL", "-o", &path.display().to_string(), &full])
        .context("no se pudo descargar el wallpaper de Bing")?;
    if !out.success {
        bail!("la descarga de Bing falló: {}", out.stderr);
    }
    Ok(path)
}

/// Sincroniza el greeter solo si el wallpaper activo cambió.
pub fn sync_greeter() -> Result<bool> {
    let Some(current) = current_wallpaper() else {
        return Ok(false);
    };
    let state = state_path();
    let previous = std::fs::read_to_string(&state).ok();
    if previous.as_deref().map(str::trim) == Some(current.as_str()) {
        return Ok(false);
    }
    let out =
        env::run("noctalia", &["msg", "greeter-sync"]).context("no se pudo ejecutar noctalia")?;
    if !out.success {
        bail!("noctalia greeter-sync falló: {}", out.stderr);
    }
    if let Some(parent) = state.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(&state, format!("{current}\n"));
    Ok(true)
}

pub fn status() -> WallpaperStatus {
    let config = load_config();
    WallpaperStatus {
        mode: config.mode,
        path: config.path,
        current: current_wallpaper(),
        last_bing: config.last_bing,
        bing_timer: is_enabled(BING_TIMER),
        greeter_timer: is_enabled(GREETER_TIMER),
        daily_plugin: daily_plugin_enabled(),
        applied: true,
        message: None,
        config_file: config_path().display().to_string(),
    }
}

/// Escribe las unidades de systemd del updater y del greeter.
pub fn ensure_units() -> Result<()> {
    let dir = systemd_dir();
    std::fs::create_dir_all(&dir).with_context(|| format!("no se pudo crear {}", dir.display()))?;
    let exe = std::env::current_exe()
        .context("no se pudo resolver la ruta de noct")?
        .display()
        .to_string();

    let bing_service = format!(
        "[Unit]\nDescription=Noct: wallpaper diario de Bing\nAfter=network-online.target\nWants=network-online.target\n\n[Service]\nType=oneshot\nExecStart={exe} wallpaper bing --quiet\n"
    );
    let bing_timer = "[Unit]\nDescription=Noct: actualiza el wallpaper de Bing cada día\n\n[Timer]\nOnCalendar=*-*-* 06:15:00\nRandomizedDelaySec=30m\nPersistent=true\n\n[Install]\nWantedBy=timers.target\n";
    let greeter_service = format!(
        "[Unit]\nDescription=Noct: sincroniza el greeter con el wallpaper activo\n\n[Service]\nType=oneshot\nExecStart={exe} wallpaper sync-greeter --quiet\n"
    );
    let greeter_timer = "[Unit]\nDescription=Noct: revisa el wallpaper para el greeter\n\n[Timer]\nOnBootSec=2min\nOnUnitActiveSec=5min\n\n[Install]\nWantedBy=timers.target\n";

    std::fs::write(dir.join("noct-bing-wallpaper.service"), bing_service)
        .context("no se pudo escribir el servicio de Bing")?;
    std::fs::write(dir.join(BING_TIMER), bing_timer)
        .context("no se pudo escribir el timer de Bing")?;
    std::fs::write(dir.join("noct-wallpaper-greeter.service"), greeter_service)
        .context("no se pudo escribir el servicio del greeter")?;
    std::fs::write(dir.join(GREETER_TIMER), greeter_timer)
        .context("no se pudo escribir el timer del greeter")?;
    let _ = env::run("systemctl", &["--user", "daemon-reload"]);
    Ok(())
}
