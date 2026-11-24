//! Lanzador con fallback para aplicaciones Qt/GL que fallan bajo Wayland.
//!
//! El entorno de Noctalia exporta `QT_QPA_PLATFORM=wayland`, pero muchas apps
//! (Anaconda Navigator, Qt antiguos de conda, etc.) no traen el plugin Wayland.
//! `noct run` ejecuta la app una vez con el entorno heredado; si muere con un
//! error reconocible de Qt/GLX, reintenta con `xcb` y renderizado software.

use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{bail, Result};

const FALLBACK_ENV: &[(&str, &str)] = &[
    ("QT_QPA_PLATFORM", "xcb"),
    ("QT_XCB_GL_INTEGRATION", "none"),
    ("LIBGL_ALWAYS_SOFTWARE", "1"),
];

const FALLBACK_SIGNATURES: &[&str] = &[
    "could not find the qt platform plugin",
    "no qt platform plugin could be initialized",
    "could not initialize glx",
    "cannot create platform opengl context",
    "failed to create display",
    "qt.qpa.plugin",
];

pub struct RunOptions {
    pub force_fallback: bool,
    pub dry_run: bool,
}

fn print_plan(command: &[String]) {
    eprintln!(
        "noct run: {}",
        command
            .iter()
            .map(|part| {
                if part.contains(' ') {
                    format!("{part:?}")
                } else {
                    part.clone()
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    );
    eprintln!(
        "  fallback: {}",
        FALLBACK_ENV
            .iter()
            .map(|(key, value)| format!("{key}={value}"))
            .collect::<Vec<_>>()
            .join(" ")
    );
}

fn run_with_env(command: &[String], fallback: bool) -> Result<i32> {
    let mut process = Command::new(&command[0]);
    process.args(&command[1..]);
    if fallback {
        for (key, value) in FALLBACK_ENV {
            process.env(key, value);
        }
    }
    let status = process
        .status()
        .map_err(|err| anyhow::anyhow!("no se pudo ejecutar {}: {err}", command[0]))?;
    Ok(status.code().unwrap_or(1))
}

fn run_capture(command: &[String]) -> Result<(i32, String)> {
    // stderr va a un archivo temporal en vez de un pipe: si el proceso lanza
    // hijos que heredan el descriptor, un pipe podría quedar abierto y colgar
    // el lanzador esperando EOF.
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    let log_path =
        std::env::temp_dir().join(format!("noct-run-{}-{unique}.log", std::process::id()));
    let log = std::fs::File::create(&log_path)
        .map_err(|err| anyhow::anyhow!("no se pudo crear {}: {err}", log_path.display()))?;

    let mut process = Command::new(&command[0]);
    process
        .args(&command[1..])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::from(log));

    let mut child = process
        .spawn()
        .map_err(|err| anyhow::anyhow!("no se pudo ejecutar {}: {err}", command[0]))?;
    let status = child.wait()?;
    let text = std::fs::read_to_string(&log_path).unwrap_or_default();
    let _ = std::fs::remove_file(&log_path);

    if !text.is_empty() {
        eprint!("{text}");
        if !text.ends_with('\n') {
            eprintln!();
        }
    }
    Ok((status.code().unwrap_or(1), text))
}

fn looks_like_qt_failure(stderr: &str) -> bool {
    let lower = stderr.to_ascii_lowercase();
    FALLBACK_SIGNATURES
        .iter()
        .any(|signature| lower.contains(signature))
}

pub fn run(command: &[String], options: &RunOptions) -> Result<i32> {
    if command.is_empty() {
        bail!("falta el comando a ejecutar");
    }
    if options.dry_run {
        print_plan(command);
        return Ok(0);
    }
    if options.force_fallback {
        eprintln!("noct run: usando xcb + GL software");
        return run_with_env(command, true);
    }

    let (code, stderr) = run_capture(command)?;
    if code == 0 || !looks_like_qt_failure(&stderr) {
        return Ok(code);
    }

    eprintln!("noct run: la app falló con Qt/GL en Wayland; reintentando con xcb + GL software");
    run_with_env(command, true)
}
