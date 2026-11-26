# Noct

**English** · [Español](README.es.md)

Noct is an integration layer for **[Noctalia](https://github.com/noctalia-dev/noctalia)** and **[Umbriel](https://github.com/noctalia-dev/umbriel)**. It keeps themes, wallpapers, keybinds, effects and extra configuration in sync, and adds a matching Android launcher.

Noct is not a compositor fork. It manages existing Noctalia/Umbriel installations through small, idempotent commands and managed config fragments.

**[Noga Linux](docs/INSTALL.md#noga-linux)** is the reference distribution built on top of it: CachyOS + Umbriel + Noctalia, with Noct preconfigured. It ships the **Rakia desktop**, an Umbriel + Noctalia + Noct configuration built around infinite scroll and infinite space; `rakia.desktop` selects it in display managers.

## What it includes

- **`noct` CLI**: diagnostics, themes, keybinds, compositor bridge, effects, wallpaper modes and a LAN sync server.
- **Noctalia plugin `noct/effects`**: preset picker, per-slot mixing, Mono tint/intensity, tooltips and English/Spanish translations.
- **Umbriel effects**: a custom `noct-mono` screen shader (grayscale, scanlines, vignette, tint, intensity).
- **Wallpaper modes**: fixed wallpapers are protected; Bing daily is optional and updates both the desktop and the greeter.
- **Cross-distro installer** for Arch, Debian/Ubuntu, Fedora and openSUSE, with a source-build fallback.
- **Screenshot keybinds** and Noctalia’s screenshot configuration.
- **Removable drives**: USB sticks and external SSDs are detected and mounted automatically with `udisks2` + `udiskie`.
- **Android launcher** with a widget-only home, recents approximation, text app drawer and theme/wallpaper sync.

## Requirements

- Linux with Wayland.
- Noctalia and Umbriel (the installer can install or update both).
- Rust 1.80+ to build `noct` (the installer bootstraps Rust with `rustup` when needed).
- A running Noctalia session for runtime commands such as `noctalia msg ...`.

## Install

Full guide: **[docs/INSTALL.md](docs/INSTALL.md)** · **[Español](docs/INSTALL.es.md)**.

```bash
git clone https://github.com/christiandavidfs/noct.git
cd noct

# See what would happen without changing anything
./install.sh --dry-run

# Install/update Noctalia, Umbriel, Noct and the Noctalia plugin.
# A fixed wallpaper is the safe default.
./install.sh --wallpaper fixed
```

Then log out and log back in, selecting **Umbriel** as the session.

### Installer options

| Option | Meaning |
| --- | --- |
| `--dry-run` | Print commands without running them |
| `--yes` | Do not ask for confirmation |
| `--lang en\|es` | Output language (default: system locale) |
| `--user` | Install Noctalia/Umbriel under `~/.local` (no sudo for them) |
| `--prefix PATH` | Noct install prefix (default `~/.local`) |
| `--system-prefix PATH` | Noctalia/Umbriel prefix (default `/usr/local`) |
| `--no-system` | Skip system dependency installation |
| `--no-noctalia` | Do not install/update Noctalia |
| `--no-umbriel` | Do not install/update Umbriel |
| `--no-udisks` | Skip automatic USB/removable drive mounting |
| `--keep-greeter-logo` | Keep the Noctalia greeter logo (hidden by default) |
| `--from-source` | Build Noctalia and Umbriel from source |
| `--wallpaper fixed\|bing-daily` | Wallpaper mode |
| `--wallpaper-path PATH` | Fixed wallpaper to apply |
| `--with-greeter` | Install `noctalia-greeter` (does not configure greetd) |
| `--no-migrate` | Do not migrate old Umbriel keybind syntax |

The installer is bilingual: it follows the system locale and also accepts
`--lang en` or `--lang es`. It installs `udisks2` + `udiskie` with a user service so
USB sticks and external SSDs mount automatically; use `--no-udisks` to skip that. If
`noctalia-greeter` is installed, its logo is hidden on the login screen by default;
`--keep-greeter-logo` keeps it.

### Distribution notes

- **Arch**: installs `noctalia` from the official repositories and `umbriel-git` from the AUR (`paru`/`yay`).
- **Debian/Ubuntu**: adds the Noctalia APT repository, enables Debian backports when needed, and installs `noctalia`, `umbriel` and `xdg-desktop-portal-umbriel`.
- **Fedora**: installs `noctalia`; tries Terra’s `umbriel-nightly` and falls back to a source build.
- **openSUSE**: uses the Noctalia OBS repository.
- **Other distributions**: source builds Noctalia/Umbriel with best-effort dependencies.

## Wallpaper modes

Wallpaper behavior is explicit, so a fixed wallpaper never silently turns back into Bing.

```bash
# Keep the current wallpaper and disable Bing automation
noct wallpaper fixed

# Pin a specific file
noct wallpaper fixed ~/Pictures/Wallpapers/wallhaven-21j6og.png

# Enable daily Bing: desktop + greeter are updated every day
noct wallpaper bing-daily

# Apply today's Bing image once and pin it
noct wallpaper bing

# Inspect the active mode
noct wallpaper status
```

- `fixed` disables the Bing timer and the old `nzlov/daily-wallpaper` plugin.
- `bing-daily` installs systemd user timers:
  - `noct-bing-wallpaper.timer` downloads and applies Bing once a day.
  - `noct-wallpaper-greeter.timer` keeps the greeter in sync whenever the wallpaper changes.
- Noct’s Bing updater replaces the old daily-wallpaper plugin to avoid two processes fighting over the wallpaper.

## Effects and the Noctalia widget

```bash
noct effects list
noct effects apply mono
noct effects apply matrix
noct effects intensity 60
noct effects tint "#7cffb2"
noct effects tint none

# Per-slot overrides, mixed over any preset
noct effects mix show
noct effects mix set cursor trail
noct effects mix set screen noct-mono
noct effects mix set border off
noct effects mix clear
```

The widget is installed with:

```bash
noct effects widget
noctalia msg panel-toggle noct/effects:panel
```

It includes:

- preset picker;
- **Mix effects** panel with independent `border`, `cursor`, `screen`, `window` and animation slots;
- tooltips describing every effect;
- Mono tint and intensity controls;
- English and Spanish translations (`translations/en.json`, `translations/es.json`).

## Screen capture keybinds

Noctalia’s screenshot configuration lives in `~/.local/state/noctalia/settings.toml` under `[shell.screenshot]`.

Useful commands:

```bash
noctalia msg screenshot-region       # interactive region
noctalia msg screenshot-window       # focused window
noctalia msg screenshot-fullscreen   # focused monitor
noctalia msg screenshot-fullscreen all
noctalia msg screenshot-annotate     # freeze and annotate
```

The installer adds these keybinds when they are free:

| Key | Action |
| --- | --- |
| `Print` | Region screenshot |
| `Shift+Print` | Window screenshot |
| `Ctrl+Print` | Monitor screenshot |
| `Alt+Print` | Annotate screenshot |

Default configuration: PNG, quality 90, copy to clipboard, save to `~/Pictures/Screenshots`, annotate and confirm region enabled.

## CLI overview

```text
noct doctor [--fix] [--json]
noct theme export|apply|show|rollback
noct keys list|check|migrate|set|unset|actions|translate
noct bridge export|show|apply
noct effects list|apply|current|intensity|tint|mix|widget
noct wallpaper status|fixed|bing-daily|bing|sync-greeter
noct sync serve
noct run <command>
```

## Configuration paths

| Path | Purpose |
| --- | --- |
| `~/.config/umbriel/config.toml` | Umbriel base configuration and includes |
| `~/.config/umbriel/noct/keybinds.toml` | Keybinds managed by Noct |
| `~/.config/umbriel/noct/bridge.toml` | Niri ↔ Umbriel bridge fragment |
| `~/.config/umbriel/noct/effects.toml` | Active effects preset and mix |
| `~/.config/umbriel/noct/overrides.toml` | Per-slot mix overrides |
| `~/.config/noct/wallpaper.toml` | Wallpaper mode and pinned path |
| `~/.config/noct/current.json` | Last applied Noct theme |
| `~/.local/share/noctalia/plugins/effects/` | Installed Noctalia plugin |
| `~/.local/state/noctalia/settings.toml` | Noctalia settings (screenshots, notifications, wallpaper, theme) |

## Umbriel compatibility

Current Umbriel versions use space-separated keybind actions:

```toml
"Mod+Return" = "spawn kitty"
"Mod+1" = "workspace-switch 1"
"Mod+Shift+1" = "window-move-to-workspace 1"
```

Noct and the installer migrate the old colon syntax:

```toml
"Mod+Return" = "spawn:kitty"
"Mod+1" = "workspace-switch:1"
```

They also migrate `input.focus.follows_mouse = true` to `"sloppy"` and `umbriel msg output-enable:eDP-1` to `output-enable eDP-1`.

## Android launcher

The `android/` directory contains a Jetpack Compose launcher:

- widget-only home with reorderable blocks;
- recents carousel with approximate active apps;
- text-only app drawer;
- settings and theme/wallpaper sync.

Run `noct sync serve` on the desktop and enter the URL + token in the launcher’s **Sync** screen.

## Noga Linux

**Noga Linux** is the reference distribution for Noct: a minimal CachyOS-based live ISO with Umbriel, Noctalia and Noct preconfigured (Niri fallback). It keeps CachyOS repositories, keyring and package names, so updates continue to come from CachyOS.

The **Rakia desktop** is the Umbriel + Noctalia + Noct configuration it ships: scrollable tiling, infinite space, and the Noct theme layer. The session is published as `rakia.desktop`. It defaults to the dark theme, the Noga wallpaper and the Kepler mark in the launcher and dock.

Current status:

- live ISO boots on BIOS (syslinux) and UEFI (systemd-boot), with autologin into Umbriel + Noctalia;
- slim and NVIDIA editions are generated from the same profile;
- the graphical installer (Calamares) installs the system offline and writes GRUB for BIOS and UEFI;
- USB and external drives are mounted automatically in live and installed systems;
- Noct’s effects plugin is included in the live image and can be installed with `noct effects widget`.

Build, burn and test instructions: **[docs/INSTALL.md#noga-linux](docs/INSTALL.md#noga-linux)** · **[Español](docs/INSTALL.es.md#noga-linux)**.
The Noga previews are published under [GitHub Releases](https://github.com/christiandavidfs/noct/releases).

## Documentation

- [Installation guide (English)](docs/INSTALL.md)
- [Guía de instalación (Español)](docs/INSTALL.es.md)
- [README en español](README.es.md)

## Development

```bash
cargo fmt
cargo clippy --all-targets
cargo test
cargo build --release -p noct-cli
```

Noctalia plugin lint:

```bash
noctalia plugins lint ~/.local/share/noctalia/plugins/effects
```

## License

MIT. See [LICENSE](LICENSE).
