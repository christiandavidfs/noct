use std::io::IsTerminal;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use noct_core::{config, doctor, effects, env, keys, profile, run, sync, theme, wallpaper};

#[derive(Parser)]
#[command(
    name = "noct",
    version,
    about = "Noct: tema, configuración y sync para Noctalia + Umbriel"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Diagnostica el escritorio y su configuración
    Doctor {
        /// Salida JSON para scripts
        #[arg(long)]
        json: bool,
        /// Corrige permisos de plantillas generadas de solo lectura
        #[arg(long)]
        fix: bool,
    },
    /// Gestiona temas de Noct
    Theme {
        #[command(subcommand)]
        cmd: ThemeCmd,
    },
    /// Inspecciona y migra atajos
    Keys {
        #[command(subcommand)]
        cmd: KeysCmd,
    },
    /// Puente de configuración entre Niri y Umbriel
    Bridge {
        #[command(subcommand)]
        cmd: BridgeCmd,
    },
    /// Presets visuales de Umbriel
    Effects {
        #[command(subcommand)]
        cmd: EffectsCmd,
    },
    /// Gestiona el wallpaper fijo o el Bing diario
    Wallpaper {
        #[command(subcommand)]
        cmd: WallpaperCmd,
    },
    /// Sincroniza tema y wallpaper con el launcher Android
    Sync {
        #[command(subcommand)]
        cmd: SyncCmd,
    },
    /// Ejecuta una app con fallback Qt/GL para Wayland
    Run {
        /// Fuerza xcb + GL software sin probar Wayland
        #[arg(long)]
        force_fallback: bool,
        /// Muestra el comando y el entorno sin ejecutarlo
        #[arg(long)]
        dry_run: bool,
        /// Comando y argumentos
        #[arg(
            trailing_var_arg = true,
            allow_hyphen_values = true,
            required = true,
            num_args = 1..
        )]
        command: Vec<String>,
    },
}

#[derive(Subcommand)]
enum SyncCmd {
    /// Arranca el servidor LAN de sincronización
    Serve {
        /// Puerto HTTP
        #[arg(long, default_value_t = 8787)]
        port: u16,
        /// Interfaz de escucha
        #[arg(long, default_value = "0.0.0.0")]
        bind: String,
        /// Token fijo (por defecto se genera uno aleatorio)
        #[arg(long)]
        token: Option<String>,
        /// Tema exportado a servir (por defecto, el activo de Noctalia)
        #[arg(long)]
        theme: Option<PathBuf>,
        /// Wallpaper a servir (por defecto, el del tema)
        #[arg(long)]
        wallpaper: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum WallpaperCmd {
    /// Muestra el modo, el wallpaper activo y los timers
    Status {
        #[arg(long)]
        json: bool,
    },
    /// Fija un wallpaper y desactiva cualquier rotación automática
    Fixed {
        /// Ruta del wallpaper; si se omite, usa el actual
        path: Option<PathBuf>,
        #[arg(long)]
        json: bool,
    },
    /// Activa el Bing diario y sincroniza escritorio + greeter
    BingDaily {
        #[arg(long)]
        json: bool,
    },
    /// Descarga y aplica el Bing de hoy
    Bing {
        /// No imprime nada si todo va bien (para systemd)
        #[arg(long)]
        quiet: bool,
        #[arg(long)]
        json: bool,
    },
    /// Sincroniza el greeter si el wallpaper activo cambió
    SyncGreeter {
        #[arg(long)]
        quiet: bool,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum EffectsCmd {
    /// Lista los presets y los efectos instalados
    List {
        #[arg(long)]
        json: bool,
    },
    /// Aplica un preset visual
    Apply {
        /// Identificador del preset (neon, cinema, matrix, minimal, mono, custom, off)
        preset: String,
        /// Muestra el fragmento sin escribir nada
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        json: bool,
    },
    /// Muestra los efectos activos
    Current {
        #[arg(long)]
        json: bool,
    },
    /// Cambia la intensidad del preset Mono (0-100)
    Intensity {
        /// Intensidad 0-100
        value: u8,
        /// Muestra el shader generado sin escribir nada
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        json: bool,
    },
    /// Cambia el tinte del preset Mono (#RRGGBB o none)
    Tint {
        /// Color en hex, o "none" para gris puro
        color: String,
        /// Muestra el shader generado sin escribir nada
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        json: bool,
    },
    /// Mezcla efectos por slot sobre el preset activo
    Mix {
        #[command(subcommand)]
        cmd: MixCmd,
    },
    /// Instala el widget de Noctalia para elegir efectos y tinte
    Widget {
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum MixCmd {
    /// Muestra los overrides activos y las opciones instaladas
    Show {
        #[arg(long)]
        json: bool,
    },
    /// Cambia un slot (border, cursor, screen, window, windows_move...)
    Set {
        /// Slot a cambiar
        slot: String,
        /// Efecto instalado, o "off"/"none" para desactivarlo
        value: String,
        /// Muestra el resultado sin escribir nada
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        json: bool,
    },
    /// Quita un override (sin slot, quita todos)
    Clear {
        /// Slot a limpiar; omite para limpiar toda la mezcla
        slot: Option<String>,
        /// Muestra el resultado sin escribir nada
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum BridgeCmd {
    /// Exporta la configuración del compositor a un perfil canónico
    Export {
        /// Compositor de origen (por defecto, el activo)
        #[arg(long, value_parser = ["niri", "umbriel"])]
        from: Option<String>,
        /// Ruta del perfil (por defecto ~/.config/noct/profile.toml)
        #[arg(long, short)]
        out: Option<PathBuf>,
        /// Salida JSON
        #[arg(long)]
        json: bool,
    },
    /// Muestra el perfil canónico guardado
    Show {
        /// Salida JSON
        #[arg(long)]
        json: bool,
    },
    /// Genera y aplica el fragmento gestionado en un compositor
    Apply {
        /// Compositor de destino
        #[arg(long, value_parser = ["niri", "umbriel"])]
        to: String,
        /// Muestra el fragmento sin escribir nada
        #[arg(long)]
        dry_run: bool,
        /// Salida JSON
        #[arg(long)]
        json: bool,
    },
    /// Exporta del compositor activo y aplica al otro
    Sync {
        /// Compositor de origen (por defecto, el activo)
        #[arg(long, value_parser = ["niri", "umbriel"])]
        from: Option<String>,
        /// Muestra los fragmentos sin escribir nada
        #[arg(long)]
        dry_run: bool,
        /// Aplica también al compositor de origen
        #[arg(long)]
        both: bool,
        /// Salida JSON
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum ThemeCmd {
    /// Exporta el tema activo de Noctalia a un JSON canónico
    Export {
        /// Ruta de salida (por defecto ./noct-theme.json)
        #[arg(long, short)]
        out: Option<PathBuf>,
        /// Nombre legible del tema
        #[arg(long)]
        name: Option<String>,
        /// Esquema Material You (m3-tonal-spot, m3-monochrome, ...)
        #[arg(long)]
        scheme: Option<String>,
        /// Exporta un bundle portable (theme.json + wallpapers)
        #[arg(long)]
        bundle: Option<PathBuf>,
        /// Wallpaper para móvil que se incluye en el bundle
        #[arg(long)]
        mobile_wallpaper: Option<PathBuf>,
    },
    /// Aplica un tema exportado sobre Noctalia y el greeter
    Apply {
        /// Tema a aplicar
        file: PathBuf,
        /// Muestra los pasos sin ejecutarlos
        #[arg(long)]
        dry_run: bool,
        /// No sincronizar el greeter
        #[arg(long)]
        no_greeter: bool,
        /// Salida JSON
        #[arg(long)]
        json: bool,
    },
    /// Muestra el último tema aplicado por Noct
    Show {
        #[arg(long)]
        json: bool,
    },
    /// Restaura el estado anterior a la última aplicación
    Rollback,
}

#[derive(Subcommand)]
enum KeysCmd {
    /// Lista los atajos de Umbriel y Niri
    List {
        #[arg(long)]
        json: bool,
    },
    /// Detecta conflictos y comandos obsoletos
    Check {
        #[arg(long)]
        json: bool,
    },
    /// Migra `qs -c noctalia-shell` a `noctalia msg`
    Migrate {
        /// Escribe los cambios (con copia .noct.bak)
        #[arg(long)]
        apply: bool,
        #[arg(long)]
        json: bool,
    },
    /// Añade o cambia un atajo en el archivo gestionado por Noct
    Set {
        /// Chord, por ejemplo "Mod+Return"
        chord: String,
        /// Acción de Umbriel, por ejemplo "window-close" o "spawn kitty"
        action: String,
        /// Desactiva la repetición al mantener pulsado
        #[arg(long)]
        no_repeat: bool,
        /// Muestra los cambios sin escribir
        #[arg(long)]
        dry_run: bool,
    },
    /// Elimina un atajo gestionado por Noct
    Unset { chord: String },
    /// Lista las acciones disponibles del compositor
    Actions {
        /// Filtra por texto
        search: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Traduce los atajos de Niri al formato de Umbriel
    Translate {
        /// Escribe los atajos traducidos en el archivo gestionado
        #[arg(long)]
        apply: bool,
        #[arg(long)]
        json: bool,
    },
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(err) => {
            eprintln!("noct: {err:#}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<ExitCode> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Doctor { json, fix } => cmd_doctor(json, fix),
        Commands::Theme { cmd } => cmd_theme(cmd),
        Commands::Keys { cmd } => cmd_keys(cmd),
        Commands::Bridge { cmd } => cmd_bridge(cmd),
        Commands::Effects { cmd } => cmd_effects(cmd),
        Commands::Wallpaper { cmd } => cmd_wallpaper(cmd),
        Commands::Sync { cmd } => cmd_sync(cmd),
        Commands::Run {
            force_fallback,
            dry_run,
            command,
        } => {
            let code = run::run(
                &command,
                &run::RunOptions {
                    force_fallback,
                    dry_run,
                },
            )?;
            Ok(ExitCode::from(code.clamp(0, 255) as u8))
        }
    }
}

fn color(enabled: bool, code: &str, text: &str) -> String {
    if enabled {
        format!("\x1b[{code}m{text}\x1b[0m")
    } else {
        text.to_string()
    }
}

fn cmd_doctor(json: bool, fix: bool) -> Result<ExitCode> {
    if fix {
        let fixed = doctor::fix_template_permissions();
        if fixed.is_empty() {
            println!("No hay plantillas de solo lectura que corregir.");
        } else {
            for path in fixed {
                println!("✓ Permiso de escritura restaurado: {path}");
            }
        }
        for message in config::fix_all() {
            println!("✓ {message}");
        }
        println!();
    }

    let report = doctor::run();

    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
        return Ok(ExitCode::from(report.exit_code() as u8));
    }

    let tty = std::io::stdout().is_terminal();
    let s = &report.system;

    println!("{}", color(tty, "1", "Noct doctor"));
    println!(
        "  sesión: {} · compositor: {} · display: {}",
        s.session_desktop.as_deref().unwrap_or("desconocida"),
        s.compositor,
        s.wayland_display.as_deref().unwrap_or("—"),
    );
    match (&s.noctalia_exe, s.noctalia_running) {
        (Some(exe), true) => println!(
            "  noctalia: {} (pid {}) · {}",
            exe,
            s.noctalia_pids
                .iter()
                .map(|p| p.to_string())
                .collect::<Vec<_>>()
                .join(","),
            s.noctalia_version
                .as_deref()
                .unwrap_or("versión desconocida"),
        ),
        _ => println!("  noctalia: no está corriendo"),
    }
    if let Some(cmd) = &s.greeter_command {
        println!("  greeter: {cmd}");
    }
    println!();

    for item in &report.checks {
        let (code, label) = match item.severity {
            doctor::Severity::Ok => ("32", "ok"),
            doctor::Severity::Info => ("36", "info"),
            doctor::Severity::Warn => ("33", "warn"),
            doctor::Severity::Error => ("31", "error"),
        };
        println!(
            "{} {} {}",
            color(tty, code, &format!("[{label:5}]")),
            color(tty, "1", &item.title),
            color(tty, "2", &format!("({})", item.id)),
        );
        println!("        {}", item.detail);
        if let Some(fix) = &item.fix {
            println!("        {} {}", color(tty, "2", "fix:"), fix);
        }
    }

    println!();
    let summary = &report.summary;
    println!(
        "Resumen: {} ok · {} info · {} warn · {} error",
        summary.ok, summary.info, summary.warn, summary.error
    );

    Ok(ExitCode::from(report.exit_code() as u8))
}

fn copy_wallpaper(src: &Path, dir: &Path, stem: &str) -> Result<String> {
    if !src.exists() {
        bail!("no existe el wallpaper {}", src.display());
    }
    let ext = src.extension().and_then(|e| e.to_str()).unwrap_or("jpg");
    let name = format!("{stem}.{ext}");
    let dest = dir.join(&name);
    std::fs::copy(src, &dest)
        .with_context(|| format!("no se pudo copiar {} a {}", src.display(), dest.display()))?;
    Ok(name)
}

fn mode_label(theme: &theme::Theme) -> String {
    match &theme.resolved_mode {
        Some(resolved) if resolved != &theme.mode => format!("{} ({resolved})", theme.mode),
        _ => theme.mode.clone(),
    }
}

fn cmd_theme(cmd: ThemeCmd) -> Result<ExitCode> {
    match cmd {
        ThemeCmd::Export {
            out,
            name,
            scheme,
            bundle,
            mobile_wallpaper,
        } => {
            let mut theme = theme::Theme::from_noctalia(name.as_deref(), scheme.as_deref())?;

            if let Some(mobile) = mobile_wallpaper {
                theme.wallpapers.mobile = Some(mobile.display().to_string());
            }

            let path = match bundle {
                Some(dir) => {
                    std::fs::create_dir_all(&dir)
                        .with_context(|| format!("no se pudo crear {}", dir.display()))?;

                    if let Some(desktop) = theme.wallpapers.desktop.clone() {
                        let name = copy_wallpaper(Path::new(&desktop), &dir, "wallpaper")?;
                        theme.wallpapers.desktop = Some(name);
                    }
                    if let Some(mobile) = theme.wallpapers.mobile.clone() {
                        let name = copy_wallpaper(Path::new(&mobile), &dir, "wallpaper-mobile")?;
                        theme.wallpapers.mobile = Some(name);
                    }

                    dir.join("theme.json")
                }
                None => out.unwrap_or_else(|| PathBuf::from("noct-theme.json")),
            };

            theme.save(&path)?;

            println!("Tema exportado: {}", path.display());
            println!("  id:        {}", theme.id);
            println!("  nombre:    {}", theme.name);
            println!(
                "  roles:     {} dark / {} light",
                theme.palette.dark.len(),
                theme.palette.light.len()
            );
            println!(
                "  modo:      {} · fuente: {} · esquema: {}",
                mode_label(&theme),
                theme.source,
                theme.scheme
            );
            if let Some(wallpaper) = &theme.wallpapers.desktop {
                println!("  wallpaper: {wallpaper}");
            }
            if let Some(mobile) = &theme.wallpapers.mobile {
                println!("  móvil:     {mobile}");
            }
            Ok(ExitCode::SUCCESS)
        }
        ThemeCmd::Apply {
            file,
            dry_run,
            no_greeter,
            json,
        } => {
            let mut theme = theme::Theme::load(&file)?;
            theme.resolve_paths(file.parent().unwrap_or_else(|| Path::new(".")));
            let opts = theme::ApplyOptions {
                dry_run,
                greeter: !no_greeter,
            };

            let backup = if dry_run {
                None
            } else {
                let snapshot = theme::Backup::snapshot();
                let path = snapshot.save()?;
                Some((snapshot, path))
            };

            match theme.apply(&opts) {
                Ok(steps) => {
                    if json {
                        println!("{}", serde_json::to_string_pretty(&steps)?);
                    } else {
                        let tty = std::io::stdout().is_terminal();
                        if let Some((_, path)) = &backup {
                            println!("Backup: {}", path.display());
                        }
                        for step in &steps {
                            let mark = if !step.executed {
                                color(tty, "2", "·")
                            } else if step.ok {
                                color(tty, "32", "✓")
                            } else {
                                color(tty, "31", "✗")
                            };
                            println!("{mark} {}", step.description);
                            if let Some(command) = &step.command {
                                println!("    {}", color(tty, "2", command));
                            }
                            if let Some(output) = &step.output {
                                if !output.is_empty() {
                                    println!("    {output}");
                                }
                            }
                        }
                        if dry_run {
                            println!("\nDry run: no se cambió nada.");
                        } else {
                            println!("\nTema aplicado: {}", theme.name);
                        }
                    }
                    Ok(ExitCode::SUCCESS)
                }
                Err(err) => {
                    if let Some((snapshot, _)) = backup {
                        eprintln!("Aplicación fallida, restaurando estado anterior…");
                        match snapshot.restore() {
                            Ok(steps) => {
                                for step in steps {
                                    eprintln!("  restaurado: {}", step.description);
                                }
                            }
                            Err(restore_err) => {
                                eprintln!("  no se pudo restaurar: {restore_err:#}");
                            }
                        }
                    }
                    Err(err)
                }
            }
        }
        ThemeCmd::Show { json } => {
            let path = env::app_config("noct").join("current.json");
            if !path.exists() {
                bail!("no hay tema aplicado todavía ({})", path.display());
            }
            let theme = theme::Theme::load(&path)?;
            if json {
                println!("{}", theme.to_json()?);
            } else {
                println!("{} ({})", theme.name, theme.id);
                println!(
                    "  modo: {} · fuente: {} · esquema: {}",
                    mode_label(&theme),
                    theme.source,
                    theme.scheme
                );
                println!(
                    "  roles: {} dark / {} light",
                    theme.palette.dark.len(),
                    theme.palette.light.len()
                );
                if let Some(wallpaper) = &theme.wallpapers.desktop {
                    println!("  wallpaper: {wallpaper}");
                }
            }
            Ok(ExitCode::SUCCESS)
        }
        ThemeCmd::Rollback => {
            let Some(backup) = theme::Backup::load_latest()? else {
                bail!("no hay backups en {}", theme::backup_dir().display());
            };
            let steps = backup.restore()?;
            for step in steps {
                println!("✓ {}", step.description);
            }
            println!("Estado anterior restaurado.");
            Ok(ExitCode::SUCCESS)
        }
    }
}

fn cmd_keys(cmd: KeysCmd) -> Result<ExitCode> {
    match cmd {
        KeysCmd::List { json } => {
            let mut binds = keys::load_umbriel_default()?;
            binds.extend(keys::load_niri_default()?);

            if json {
                println!("{}", serde_json::to_string_pretty(&binds)?);
                return Ok(ExitCode::SUCCESS);
            }

            if binds.is_empty() {
                println!("No se encontraron atajos en Umbriel ni Niri.");
                return Ok(ExitCode::SUCCESS);
            }

            let width = binds.iter().map(|b| b.chord.len()).max().unwrap_or(0);
            for bind in &binds {
                println!(
                    "{:<width$}  {:<40}  {}:{}",
                    bind.chord,
                    bind.action,
                    bind.file,
                    bind.line,
                    width = width,
                );
            }
            println!("\n{} atajos.", binds.len());
            Ok(ExitCode::SUCCESS)
        }
        KeysCmd::Check { json } => {
            let mut binds = keys::load_umbriel_default()?;
            binds.extend(keys::load_niri_default()?);
            let conflicts = keys::find_conflicts(&binds);
            let legacy = keys::scan_legacy();

            if json {
                let payload = serde_json::json!({
                    "bindings": binds.len(),
                    "conflicts": conflicts,
                    "legacy": legacy,
                });
                println!("{}", serde_json::to_string_pretty(&payload)?);
                return Ok(ExitCode::SUCCESS);
            }

            println!("Atajos: {}", binds.len());

            if conflicts.is_empty() {
                println!("Conflictos: ninguno");
            } else {
                println!("Conflictos: {}", conflicts.len());
                for conflict in &conflicts {
                    println!(
                        "  [{}] {} ({})",
                        conflict.compositor, conflict.chord, conflict.normalized
                    );
                    for bind in &conflict.bindings {
                        println!("    → {}  ({}:{})", bind.action, bind.file, bind.line);
                    }
                }
            }

            if legacy.is_empty() {
                println!("Comandos obsoletos: ninguno");
            } else {
                let pending = legacy.iter().filter(|r| r.suggestion.is_none()).count();
                println!(
                    "Comandos obsoletos: {} ({} sin equivalencia automática)",
                    legacy.len(),
                    pending
                );
                for item in &legacy {
                    match &item.suggestion {
                        Some(suggestion) => println!(
                            "  {}:{}  {}  →  {}",
                            item.file, item.line, item.command, suggestion
                        ),
                        None => println!(
                            "  {}:{}  {}  →  (revisar a mano)",
                            item.file, item.line, item.command
                        ),
                    }
                }
                println!("\nAplica la migración con: noct keys migrate --apply");
            }

            Ok(ExitCode::SUCCESS)
        }
        KeysCmd::Migrate { apply, json } => {
            let refs = keys::scan_legacy();

            if json {
                println!("{}", serde_json::to_string_pretty(&refs)?);
                if apply {
                    let outcomes = keys::apply_migration(&refs)?;
                    eprintln!("{}", serde_json::to_string_pretty(&outcomes)?);
                }
                return Ok(ExitCode::SUCCESS);
            }

            if refs.is_empty() {
                println!("No hay comandos obsoletos que migrar.");
                return Ok(ExitCode::SUCCESS);
            }

            let mut pending = 0;
            for item in &refs {
                match &item.suggestion {
                    Some(suggestion) => println!(
                        "{}:{}\n  - {}\n  + {}",
                        item.file, item.line, item.command, suggestion
                    ),
                    None => {
                        pending += 1;
                        println!(
                            "{}:{}\n  ? {}  (sin equivalencia automática)",
                            item.file, item.line, item.command
                        );
                    }
                }
            }

            if apply {
                let outcomes = keys::apply_migration(&refs)?;
                for outcome in &outcomes {
                    println!(
                        "\n✓ {} — {} reemplazos (backup: {})",
                        outcome.file,
                        outcome.replacements,
                        outcome.backup.as_deref().unwrap_or("—")
                    );
                }
                if pending > 0 {
                    println!("\n{pending} comandos quedaron pendientes de revisión manual.");
                }
            } else {
                println!("\nDry run: no se escribió nada. Usa --apply para migrar.");
            }

            Ok(ExitCode::SUCCESS)
        }
        KeysCmd::Set {
            chord,
            action,
            no_repeat,
            dry_run,
        } => {
            let catalog = keys::umbriel_actions();
            if !keys::is_valid_action(&action, &catalog) {
                bail!(
                    "acción desconocida: `{action}`. Usa `noct keys actions` para ver el catálogo."
                );
            }

            let managed = keys::managed_path();
            let previous = std::fs::read_to_string(&managed).ok();
            let repeat = if no_repeat { Some(false) } else { None };

            if dry_run {
                println!("Incluir: {}", managed.display());
                println!("Escribir: {chord} = {{ action = \"{action}\" }}");
                println!("Validar: umbriel config validate");
                println!("Recargar: umbriel msg config-reload (si está corriendo)");
                return Ok(ExitCode::SUCCESS);
            }

            let include = keys::ensure_include()?;
            if include.added {
                println!(
                    "Include añadido a {} (backup: {})",
                    include.file,
                    include.backup.as_deref().unwrap_or("—")
                );
            }

            keys::set_managed(&chord, &action, repeat)?;

            let _ = config::fix_all();
            let validate = env::run("umbriel", &["config", "validate"]);
            let valid = validate.as_ref().map(|o| o.success).unwrap_or(false);
            if !valid {
                match previous {
                    Some(text) => {
                        let _ = std::fs::write(&managed, text);
                    }
                    None => {
                        let _ = std::fs::remove_file(&managed);
                    }
                }
                let detail = validate
                    .map(|o| o.stderr)
                    .unwrap_or_else(|| "`umbriel` no está disponible".to_string());
                bail!("la configuración no valida; cambio revertido: {detail}");
            }

            println!("✓ {chord} → {action}");

            let existing = keys::load_umbriel_default().unwrap_or_default();
            let normalized = keys::normalize_chord(&chord);
            for bind in existing
                .iter()
                .filter(|b| keys::normalize_chord(&b.chord) == normalized && b.action != action)
            {
                println!(
                    "! Aviso: {} ya está usado por `{}` ({}:{})",
                    bind.chord, bind.action, bind.file, bind.line
                );
            }

            if env::umbriel_alive() {
                let reload = env::run("umbriel", &["msg", "config-reload"]);
                match reload {
                    Some(out) if out.success => println!("✓ Umbriel recargado en vivo"),
                    Some(out) => println!("! No se pudo recargar: {}", out.stderr),
                    None => println!("! No se pudo contactar con Umbriel"),
                }
            } else {
                println!("· Umbriel no está corriendo; se aplicará al arrancar");
            }

            Ok(ExitCode::SUCCESS)
        }
        KeysCmd::Unset { chord } => {
            if !keys::unset_managed(&chord)? {
                println!("No había un atajo gestionado para {chord}.");
                return Ok(ExitCode::SUCCESS);
            }
            println!("✓ Eliminado {chord}");
            if env::umbriel_alive() {
                let _ = env::run("umbriel", &["msg", "config-reload"]);
            }
            Ok(ExitCode::SUCCESS)
        }
        KeysCmd::Actions { search, json } => {
            let mut actions = keys::umbriel_actions();
            if let Some(query) = &search {
                let query = query.to_ascii_lowercase();
                actions.retain(|a| a.to_ascii_lowercase().contains(&query));
            }

            if json {
                println!("{}", serde_json::to_string_pretty(&actions)?);
            } else if actions.is_empty() {
                println!("No se encontraron acciones.");
            } else {
                for action in &actions {
                    println!("{action}");
                }
                println!("\n{} acciones.", actions.len());
            }
            Ok(ExitCode::SUCCESS)
        }
        KeysCmd::Translate { apply, json } => {
            let report = keys::translate_niri_to_umbriel()?;

            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                println!(
                    "Traducción Niri → Umbriel: {} atajo(s), {} sin traducir",
                    report.translated.len(),
                    report.skipped.len()
                );
                for bind in &report.translated {
                    let repeat = match bind.repeat {
                        Some(false) => "  [sin repeat]",
                        _ => "",
                    };
                    println!("  {:<28} → {}{repeat}", bind.chord, bind.action);
                }
                if !report.skipped.is_empty() {
                    println!("\nSin traducir:");
                    for bind in &report.skipped {
                        println!("  {:<28} {} ({})", bind.chord, bind.action, bind.reason);
                    }
                }
            }

            if !apply {
                if !json {
                    println!(
                        "\nNada escrito. Revisa la lista y ejecuta `noct keys translate --apply`."
                    );
                }
                return Ok(ExitCode::SUCCESS);
            }
            if report.translated.is_empty() {
                println!("No hay atajos que aplicar.");
                return Ok(ExitCode::SUCCESS);
            }

            let managed = keys::managed_path();
            let previous = std::fs::read_to_string(&managed).ok();
            let include = keys::ensure_include()?;
            if include.added {
                println!(
                    "Include añadido a {} (backup: {})",
                    include.file,
                    include.backup.as_deref().unwrap_or("—")
                );
            }

            let binds: Vec<keys::ManagedBind> = report
                .translated
                .iter()
                .map(|bind| keys::ManagedBind {
                    chord: bind.chord.clone(),
                    action: bind.action.clone(),
                    repeat: bind.repeat,
                })
                .collect();
            let written = keys::merge_managed(&binds)?;

            let _ = config::fix_all();
            let validate = env::run("umbriel", &["config", "validate"]);
            let valid = validate.as_ref().map(|o| o.success).unwrap_or(false);
            if !valid {
                match previous {
                    Some(text) => {
                        let _ = std::fs::write(&managed, text);
                    }
                    None => {
                        let _ = std::fs::remove_file(&managed);
                    }
                }
                let detail = validate
                    .map(|o| o.stderr)
                    .unwrap_or_else(|| "`umbriel` no está disponible".to_string());
                bail!("la configuración no valida; traducción revertida: {detail}");
            }

            println!("✓ {written} atajo(s) escritos en {}", managed.display());
            if env::umbriel_alive() {
                match env::run("umbriel", &["msg", "config-reload"]) {
                    Some(out) if out.success => println!("✓ Umbriel recargado en vivo"),
                    Some(out) => println!("! No se pudo recargar: {}", out.stderr),
                    None => println!("! No se pudo contactar con Umbriel"),
                }
            } else {
                println!("· Umbriel no está corriendo; se aplicará al arrancar");
            }
            Ok(ExitCode::SUCCESS)
        }
    }
}

fn cmd_bridge(cmd: BridgeCmd) -> Result<ExitCode> {
    match cmd {
        BridgeCmd::Export { from, out, json } => {
            let source = resolve_source(from)?;
            let report = profile::Profile::export_from(&source)?;
            let path = out.unwrap_or_else(profile::profile_path);
            report.profile.save(&path)?;

            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                println!("Perfil exportado desde {source}");
                println!("  archivo: {}", path.display());
                println!("  mapeado: {}", report.mapped.join(", "));
                if !report.unmapped.is_empty() {
                    println!("  sin traducir: {}", report.unmapped.join(", "));
                    println!("  (esas claves siguen viviendo solo en {source}; Noct no las borra)");
                }
                println!("\nAplica el perfil al otro compositor con:");
                let other = if source == "niri" { "umbriel" } else { "niri" };
                println!("  noct bridge apply --to {other}");
            }
            Ok(ExitCode::SUCCESS)
        }
        BridgeCmd::Show { json } => {
            let path = profile::profile_path();
            if !path.exists() {
                bail!(
                    "no hay perfil en {}; ejecuta `noct bridge export` primero",
                    path.display()
                );
            }
            let profile = profile::Profile::load_default()?;
            if json {
                println!("{}", serde_json::to_string_pretty(&profile)?);
            } else {
                println!("{}", profile.to_toml()?);
            }
            Ok(ExitCode::SUCCESS)
        }
        BridgeCmd::Apply { to, dry_run, json } => {
            let profile = profile::Profile::load_default().with_context(|| {
                format!(
                    "no hay perfil en {}; ejecuta `noct bridge export` primero",
                    profile::profile_path().display()
                )
            })?;
            let (report, content) = profile::apply(&profile, &to, dry_run)?;
            if dry_run {
                println!("{content}");
            }
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else if !dry_run {
                println!("Destino: {}", report.target);
                println!("  archivo: {}", report.file);
                if report.include_added {
                    println!("  include añadido a la configuración");
                }
                if report.validated {
                    println!("  validación: ok");
                }
                println!("  {}", report.message);
            }
            Ok(ExitCode::SUCCESS)
        }
        BridgeCmd::Sync {
            from,
            dry_run,
            both,
            json,
        } => {
            let source = resolve_source(from)?;
            let report = profile::Profile::export_from(&source)?;
            let path = profile::profile_path();
            report.profile.save(&path)?;

            let mut results = Vec::new();
            let mut failures = Vec::new();
            for target in ["niri", "umbriel"] {
                if target == source && !both {
                    continue;
                }
                match profile::apply(&report.profile, target, dry_run) {
                    Ok((apply_report, content)) => results.push((apply_report, content)),
                    Err(err) => failures.push(format!("{target}: {err:#}")),
                }
            }

            if dry_run {
                for (apply_report, content) in &results {
                    println!("# {}", apply_report.file);
                    println!("{content}");
                }
            }

            if json {
                let payload = serde_json::json!({
                    "source": source,
                    "profile": path.display().to_string(),
                    "mapped": report.mapped,
                    "unmapped": report.unmapped,
                    "applied": results.iter().map(|(r, _)| r).collect::<Vec<_>>(),
                    "failures": failures,
                });
                println!("{}", serde_json::to_string_pretty(&payload)?);
            } else {
                println!("Sincronizado desde {source} → {}", path.display());
                for (apply_report, _) in &results {
                    println!(
                        "  {}: {}{}",
                        apply_report.target,
                        apply_report.message,
                        if apply_report.include_added {
                            " (include añadido)"
                        } else {
                            ""
                        }
                    );
                }
                if !report.unmapped.is_empty() {
                    println!("  sin traducir: {}", report.unmapped.join(", "));
                }
            }

            if !failures.is_empty() {
                for failure in &failures {
                    eprintln!("noct: {failure}");
                }
                return Ok(ExitCode::FAILURE);
            }
            Ok(ExitCode::SUCCESS)
        }
    }
}

fn resolve_source(from: Option<String>) -> Result<String> {
    if let Some(from) = from {
        return Ok(from);
    }
    let detected = env::detect_compositor().as_str().to_string();
    match detected.as_str() {
        "niri" | "umbriel" => Ok(detected),
        _ => bail!(
            "no se pudo detectar el compositor activo; indica el origen con --from niri|umbriel"
        ),
    }
}

fn cmd_effects(cmd: EffectsCmd) -> Result<ExitCode> {
    match cmd {
        EffectsCmd::List { json } => {
            let presets = effects::presets();
            let installed = effects::installed_effects();
            let current = effects::current()?;

            if json {
                let payload = serde_json::json!({
                    "presets": presets,
                    "installed": installed,
                    "current": current,
                });
                println!("{}", serde_json::to_string_pretty(&payload)?);
                return Ok(ExitCode::SUCCESS);
            }

            println!("Presets de Noct");
            for preset in &presets {
                let active = current.preset.as_deref() == Some(preset.id.as_str());
                let marker = if active { "●" } else { "○" };
                println!("  {marker} {:<8} {}", preset.id, preset.name);
                println!("           {}", preset.description);
                if !preset.effects.is_empty() || !preset.animations.is_empty() {
                    let mut parts = preset.effects.clone();
                    parts.extend(preset.animations.iter().cloned());
                    println!("           {}", parts.join(", "));
                }
                if !preset.missing.is_empty() {
                    println!("           faltan: {}", preset.missing.join(", "));
                }
            }
            println!("\nEfectos instalados en Umbriel ({}):", installed.len());
            for effect in &installed {
                println!("  {effect}");
            }
            if let Some(preset) = &current.preset {
                println!("\nActivo: {preset}");
            } else {
                println!("\nActivo: ninguno");
            }
            Ok(ExitCode::SUCCESS)
        }
        EffectsCmd::Apply {
            preset,
            dry_run,
            json,
        } => {
            let (report, content) = effects::apply(&preset, dry_run)?;
            if dry_run {
                println!("{content}");
            }
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else if !dry_run {
                println!("Preset: {} ({})", report.name, report.preset);
                if !report.effects.is_empty() {
                    println!("  efectos: {}", report.effects.join(", "));
                }
                if !report.animations.is_empty() {
                    println!("  animaciones: {}", report.animations.join(", "));
                }
                println!("  archivo: {}", report.file);
                if report.validated {
                    println!("  validación: ok");
                }
                println!("  {}", report.message);
            }
            Ok(ExitCode::SUCCESS)
        }
        EffectsCmd::Current { json } => {
            let current = effects::current()?;
            if json {
                println!("{}", serde_json::to_string_pretty(&current)?);
            } else if current.effects.is_empty() && current.animations.is_empty() {
                println!("Sin efectos de Noct activos.");
            } else {
                if let Some(preset) = &current.preset {
                    println!("Preset: {preset}");
                }
                if let Some(tint) = &current.tint {
                    println!("  tinte: {tint}");
                } else if current.preset.as_deref() == Some("mono") {
                    println!("  tinte: gris puro");
                }
                if current.preset.as_deref() == Some("mono") {
                    println!("  intensidad: {}%", current.intensity);
                }
                if !current.overrides.is_empty() {
                    let parts: Vec<String> = current
                        .overrides
                        .iter()
                        .map(|(slot, value)| format!("{slot}={value}"))
                        .collect();
                    println!("  mezcla: {}", parts.join(", "));
                }
                if !current.effects.is_empty() {
                    println!("  efectos: {}", current.effects.join(", "));
                }
                if !current.animations.is_empty() {
                    println!("  animaciones: {}", current.animations.join(", "));
                }
            }
            Ok(ExitCode::SUCCESS)
        }
        EffectsCmd::Widget { json } => {
            let report = effects::install_widget()?;
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                println!("Widget instalado en {}", report.plugin_dir);
                for file in &report.files {
                    println!("  {file}");
                }
                println!("  estado: {}", report.state_file);
                println!("  ajustes: {}", report.settings_file);
                if report.shortcut_added {
                    println!("  tile añadido al control center (máx. 6 atajos)");
                }
                if !report.noct_in_path {
                    println!("! `noct` no está en PATH; el widget no podrá aplicar cambios.");
                    println!(
                        "  Instálalo con: cargo install --path crates/noct-cli --root ~/.local"
                    );
                }
                println!(
                    "
Abre el panel con:"
                );
                println!("  noctalia msg panel-toggle noct/effects:panel");
                println!("En el control center, añade el tile «Noct Effects» si no aparece solo.");
            }
            Ok(ExitCode::SUCCESS)
        }
        EffectsCmd::Intensity {
            value,
            dry_run,
            json,
        } => {
            let (report, shader) = effects::set_intensity(value, dry_run)?;
            if dry_run {
                println!("{shader}");
            }
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else if !dry_run {
                println!("Intensidad: {}%", report.intensity);
                if report.validated {
                    println!("  validación: ok");
                }
                println!("  {}", report.message);
            }
            Ok(ExitCode::SUCCESS)
        }
        EffectsCmd::Tint {
            color,
            dry_run,
            json,
        } => {
            let (report, shader) = effects::set_tint(&color, dry_run)?;
            if dry_run {
                println!("{shader}");
            }
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else if !dry_run {
                match &report.tint {
                    Some(tint) => println!("Tinte: {tint}"),
                    None => println!("Tinte: gris puro"),
                }
                if report.validated {
                    println!("  validación: ok");
                }
                println!("  {}", report.message);
            }
            Ok(ExitCode::SUCCESS)
        }
        EffectsCmd::Mix { cmd } => match cmd {
            MixCmd::Show { json } => {
                let show = effects::mix_show()?;
                if json {
                    println!("{}", serde_json::to_string_pretty(&show)?);
                    return Ok(ExitCode::SUCCESS);
                }
                println!("Preset: {}", show.preset.as_deref().unwrap_or("ninguno"));
                if show.overrides.is_empty() {
                    println!("Mezcla: sin overrides (cada preset usa su configuración)");
                } else {
                    println!("Mezcla:");
                    for (slot, value) in &show.overrides {
                        println!("  {slot} = {value}");
                    }
                }
                println!("\nEfectivo:");
                for slot in effects::EFFECT_SLOTS
                    .iter()
                    .chain(effects::ANIMATION_SLOTS.iter())
                {
                    let value = show
                        .effective
                        .get(*slot)
                        .and_then(|value| value.as_deref())
                        .unwrap_or("off");
                    println!("  {slot:<13} {value}");
                }
                println!("\nOpciones instaladas:");
                for (slot, choices) in &show.choices {
                    let list = if choices.is_empty() {
                        "ninguna".to_string()
                    } else {
                        choices.join(", ")
                    };
                    println!("  {slot:<13} {list}");
                }
                Ok(ExitCode::SUCCESS)
            }
            MixCmd::Set {
                slot,
                value,
                dry_run,
                json,
            } => {
                let (report, content) = effects::set_override(&slot, &value, dry_run)?;
                if dry_run {
                    println!("{content}");
                }
                if json {
                    println!("{}", serde_json::to_string_pretty(&report)?);
                } else if !dry_run {
                    println!(
                        "Mezcla: {} = {} (preset {})",
                        report.slot.as_deref().unwrap_or("?"),
                        report.value.as_deref().unwrap_or("?"),
                        report.preset
                    );
                    if !report.effects.is_empty() {
                        println!("  efectos: {}", report.effects.join(", "));
                    }
                    if !report.animations.is_empty() {
                        println!("  animaciones: {}", report.animations.join(", "));
                    }
                    if report.validated {
                        println!("  validación: ok");
                    }
                    println!("  {}", report.message);
                }
                Ok(ExitCode::SUCCESS)
            }
            MixCmd::Clear {
                slot,
                dry_run,
                json,
            } => {
                let (report, content) = effects::clear_override(slot.as_deref(), dry_run)?;
                if dry_run {
                    println!("{content}");
                }
                if json {
                    println!("{}", serde_json::to_string_pretty(&report)?);
                } else if !dry_run {
                    match report.slot.as_deref() {
                        Some(slot) => println!("Mezcla: {slot} vuelve al preset {}", report.preset),
                        None => println!("Mezcla limpiada (preset {})", report.preset),
                    }
                    if report.validated {
                        println!("  validación: ok");
                    }
                    println!("  {}", report.message);
                }
                Ok(ExitCode::SUCCESS)
            }
        },
    }
}

fn print_wallpaper_status(status: &wallpaper::WallpaperStatus) {
    let mode = match status.mode {
        wallpaper::WallpaperMode::Fixed => "fijo",
        wallpaper::WallpaperMode::BingDaily => "Bing diario",
    };
    println!("Wallpaper: {mode}");
    if let Some(path) = &status.path {
        println!("  fijado: {path}");
    }
    if let Some(current) = &status.current {
        println!("  actual: {current}");
    }
    if let Some(last) = &status.last_bing {
        println!("  último Bing: {last}");
    }
    println!(
        "  timer Bing: {} · timer greeter: {} · plugin antiguo: {}",
        if status.bing_timer { "sí" } else { "no" },
        if status.greeter_timer { "sí" } else { "no" },
        if status.daily_plugin { "sí" } else { "no" },
    );
    if !status.applied {
        if let Some(message) = &status.message {
            println!("  aviso: no se pudo aplicar ahora: {message}");
        }
    }
    println!("  config: {}", status.config_file);
}

fn cmd_wallpaper(cmd: WallpaperCmd) -> Result<ExitCode> {
    match cmd {
        WallpaperCmd::Status { json } => {
            let status = wallpaper::status();
            if json {
                println!("{}", serde_json::to_string_pretty(&status)?);
            } else {
                print_wallpaper_status(&status);
            }
            Ok(ExitCode::SUCCESS)
        }
        WallpaperCmd::Fixed { path, json } => {
            let value = path.as_ref().map(|path| path.display().to_string());
            let status = wallpaper::set_fixed(value.as_deref())?;
            if json {
                println!("{}", serde_json::to_string_pretty(&status)?);
            } else {
                print_wallpaper_status(&status);
                println!("  Bing diario desactivado; el wallpaper queda fijado.");
            }
            Ok(ExitCode::SUCCESS)
        }
        WallpaperCmd::BingDaily { json } => {
            let status = wallpaper::set_bing_daily()?;
            if json {
                println!("{}", serde_json::to_string_pretty(&status)?);
            } else {
                print_wallpaper_status(&status);
                println!("  Bing diario activado: escritorio y greeter se actualizarán cada día.");
            }
            Ok(ExitCode::SUCCESS)
        }
        WallpaperCmd::Bing { quiet, json } => {
            let status = wallpaper::run_bing()?;
            if json {
                println!("{}", serde_json::to_string_pretty(&status)?);
            } else if !quiet {
                print_wallpaper_status(&status);
            }
            Ok(ExitCode::SUCCESS)
        }
        WallpaperCmd::SyncGreeter { quiet, json } => {
            let changed = wallpaper::sync_greeter()?;
            if json {
                println!("{}", serde_json::json!({ "synced": changed }));
            } else if !quiet {
                if changed {
                    println!("Greeter sincronizado con el wallpaper activo.");
                } else {
                    println!("El greeter ya estaba al día.");
                }
            }
            Ok(ExitCode::SUCCESS)
        }
    }
}

fn cmd_sync(cmd: SyncCmd) -> Result<ExitCode> {
    match cmd {
        SyncCmd::Serve {
            port,
            bind,
            token,
            theme,
            wallpaper,
        } => {
            sync::serve(sync::ServeOptions {
                port,
                bind,
                token,
                theme,
                wallpaper,
            })?;
            Ok(ExitCode::SUCCESS)
        }
    }
}
