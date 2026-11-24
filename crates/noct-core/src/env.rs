//! Detección del entorno: rutas XDG, binarios, procesos y compositor.

use std::path::PathBuf;
use std::process::Command;

/// Directorio home del usuario.
pub fn home() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/"))
}

fn xdg_dir(var: &str, fallback: &str) -> PathBuf {
    std::env::var_os(var)
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .unwrap_or_else(|| home().join(fallback))
}

pub fn config_home() -> PathBuf {
    xdg_dir("XDG_CONFIG_HOME", ".config")
}

pub fn state_home() -> PathBuf {
    xdg_dir("XDG_STATE_HOME", ".local/state")
}

pub fn data_home() -> PathBuf {
    xdg_dir("XDG_DATA_HOME", ".local/share")
}

/// `~/.config/<app>`
pub fn app_config(app: &str) -> PathBuf {
    config_home().join(app)
}

/// `~/.local/state/<app>`
pub fn app_state(app: &str) -> PathBuf {
    state_home().join(app)
}

#[derive(Debug, Clone)]
pub struct CmdOutput {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
}

/// Ejecuta un comando y devuelve su salida completa.
pub fn run(cmd: &str, args: &[&str]) -> Option<CmdOutput> {
    let output = Command::new(cmd).args(args).output().ok()?;
    Some(CmdOutput {
        success: output.status.success(),
        stdout: String::from_utf8_lossy(&output.stdout).trim().to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
    })
}

/// Ejecuta un comando y devuelve stdout solo si terminó con éxito.
pub fn capture(cmd: &str, args: &[&str]) -> Option<String> {
    let out = run(cmd, args)?;
    if out.success {
        Some(out.stdout)
    } else {
        None
    }
}

/// Busca todos los binarios con ese nombre en `PATH` (sin duplicados).
pub fn find_all(name: &str) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = Vec::new();
    if let Some(path) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&path) {
            let candidate = dir.join(name);
            if candidate.is_file() {
                let resolved = candidate.canonicalize().unwrap_or(candidate);
                if !found.contains(&resolved) {
                    found.push(resolved);
                }
            }
        }
    }
    found
}

/// Primer binario encontrado en `PATH`.
pub fn find(name: &str) -> Option<PathBuf> {
    find_all(name).into_iter().next()
}

/// PIDs de procesos cuyo `comm` coincide exactamente.
pub fn running_pids(name: &str) -> Vec<u32> {
    let mut pids = Vec::new();
    let Ok(entries) = std::fs::read_dir("/proc") else {
        return pids;
    };
    for entry in entries.flatten() {
        let file_name = entry.file_name();
        let Some(pid) = file_name.to_str().and_then(|s| s.parse::<u32>().ok()) else {
            continue;
        };
        let comm_path = entry.path().join("comm");
        let Ok(comm) = std::fs::read_to_string(comm_path) else {
            continue;
        };
        if comm.trim() == name {
            pids.push(pid);
        }
    }
    pids.sort_unstable();
    pids
}

pub fn pid_exe(pid: u32) -> Option<PathBuf> {
    std::fs::read_link(format!("/proc/{pid}/exe")).ok()
}

pub fn pid_cmdline(pid: u32) -> Option<String> {
    let raw = std::fs::read(format!("/proc/{pid}/cmdline")).ok()?;
    let parts: Vec<String> = raw
        .split(|b| *b == 0)
        .filter(|s| !s.is_empty())
        .map(|s| String::from_utf8_lossy(s).into_owned())
        .collect();
    if parts.is_empty() {
        None
    } else {
        Some(parts.join(" "))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Compositor {
    Umbriel,
    Niri,
    Other,
    None,
}

impl Compositor {
    pub fn as_str(self) -> &'static str {
        match self {
            Compositor::Umbriel => "umbriel",
            Compositor::Niri => "niri",
            Compositor::Other => "other",
            Compositor::None => "none",
        }
    }
}

pub fn niri_alive() -> bool {
    capture("niri", &["msg", "version"]).is_some()
}

pub fn umbriel_alive() -> bool {
    capture("umbriel", &["outputs"]).is_some()
}

/// Compositor activo, comprobando IPC primero y variables de sesión después.
pub fn detect_compositor() -> Compositor {
    if umbriel_alive() {
        return Compositor::Umbriel;
    }
    if niri_alive() {
        return Compositor::Niri;
    }
    let desktop = std::env::var("XDG_CURRENT_DESKTOP")
        .unwrap_or_default()
        .to_ascii_lowercase();
    if desktop.is_empty() && std::env::var_os("WAYLAND_DISPLAY").is_none() {
        Compositor::None
    } else if desktop.contains("umbriel") {
        Compositor::Umbriel
    } else if desktop.contains("niri") {
        Compositor::Niri
    } else {
        Compositor::Other
    }
}
