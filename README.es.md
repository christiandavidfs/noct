# Noct

[English](README.md) · **Español**

Noct es una capa de integración para **[Noctalia](https://github.com/noctalia-dev/noctalia)** y **[Umbriel](https://github.com/noctalia-dev/umbriel)**. Mantiene sincronizados temas, fondos, atajos, efectos y configuración extra, y añade un lanzador Android a juego.

Noct no es un fork del compositor. Gestiona instalaciones existentes de Noctalia/Umbriel mediante comandos pequeños e idempotentes y fragmentos de configuración propios.

**[Noga Linux](docs/INSTALL.es.md#noga-linux)** es la distribución de referencia construida sobre Noct: CachyOS + Umbriel + Noctalia, con Noct ya configurado. Incluye el **escritorio Rakia**, una configuración de Umbriel + Noctalia + Noct pensada para el scroll infinito y el espacio infinito; `rakia.desktop` lo selecciona en los gestores de sesión.

## Qué incluye

- **CLI `noct`**: diagnóstico, temas, atajos, puente con el compositor, efectos, modos de fondo y servidor de sincronización LAN.
- **Plugin de Noctalia `noct/effects`**: selector de presets, mezcla por slot, tinte/intensidad de Mono, tooltips y traducciones en inglés y español.
- **Efectos de Umbriel**: shader de pantalla `noct-mono` (escala de grises, scanlines, viñeta, tinte, intensidad).
- **Modos de fondo**: los fondos fijos están protegidos; el Bing diario es opcional y actualiza escritorio y greeter.
- **Instalador multi-distro** para Arch, Debian/Ubuntu, Fedora y openSUSE, con compilación desde fuente como respaldo.
- **Atajos de captura** y configuración de capturas de Noctalia.
- **Dispositivos extraíbles**: los USB y discos SSD externos se detectan y montan automáticamente con `udisks2` + `udiskie`.
- **Lanzador Android** con inicio solo de widgets, carrusel de recientes, cajón de apps de texto y sincronización de tema/fondo.

## Requisitos

- Linux con Wayland.
- Noctalia y Umbriel (el instalador puede instalar o actualizar ambos).
- Rust 1.80+ para compilar `noct` (el instalador instala Rust con `rustup` si hace falta).
- Una sesión de Noctalia en ejecución para comandos como `noctalia msg ...`.

## Instalación

Guía completa: **[docs/INSTALL.es.md](docs/INSTALL.es.md)** · **[English](docs/INSTALL.md)**.

```bash
git clone https://github.com/christiandavidfs/noct.git
cd noct

# Ver qué haría sin cambiar nada
./install.sh --dry-run

# Instalar/actualizar Noctalia, Umbriel, Noct y el plugin de Noctalia.
# Un fondo fijo es el modo seguro por defecto.
./install.sh --wallpaper fixed
```

Después cierra sesión y vuelve a entrar eligiendo **Umbriel** como sesión.

### Opciones del instalador

| Opción | Significado |
| --- | --- |
| `--dry-run` | Muestra los comandos sin ejecutarlos |
| `--yes` | No pide confirmación |
| `--lang en\|es` | Idioma de la salida (por defecto: locale del sistema) |
| `--user` | Instala Noctalia/Umbriel en `~/.local` (sin sudo para ellos) |
| `--prefix RUTA` | Prefijo de Noct (por defecto `~/.local`) |
| `--system-prefix RUTA` | Prefijo de Noctalia/Umbriel (por defecto `/usr/local`) |
| `--no-system` | No instala dependencias del sistema |
| `--no-noctalia` | No instala/actualiza Noctalia |
| `--no-umbriel` | No instala/actualiza Umbriel |
| `--no-udisks` | No configura el montaje automático de USB |
| `--keep-greeter-logo` | Conserva el logo del greeter de Noctalia (oculto por defecto) |
| `--from-source` | Compila Noctalia y Umbriel desde fuente |
| `--wallpaper fixed\|bing-daily` | Modo de fondo |
| `--wallpaper-path RUTA` | Fondo fijo que aplicar |
| `--with-greeter` | Instala `noctalia-greeter` (no configura greetd) |
| `--no-migrate` | No migra la sintaxis antigua de Umbriel |

El instalador es bilingüe: sigue el locale del sistema y también acepta
`--lang en` o `--lang es`. Instala `udisks2` + `udiskie` con un servicio de usuario
para que los USB y SSD externos se monten automáticamente; `--no-udisks` lo omite. Si
`noctalia-greeter` está instalado, su logo se oculta en el login por defecto;
`--keep-greeter-logo` lo conserva.

### Notas por distribución

- **Arch**: instala `noctalia` desde los repos oficiales y `umbriel-git` desde AUR (`paru`/`yay`).
- **Debian/Ubuntu**: añade el repositorio APT de Noctalia, activa backports de Debian si hace falta e instala `noctalia`, `umbriel` y `xdg-desktop-portal-umbriel`.
- **Fedora**: instala `noctalia`; prueba `umbriel-nightly` de Terra y cae a compilación desde fuente.
- **openSUSE**: usa el repositorio OBS de Noctalia.
- **Otras distribuciones**: compila Noctalia/Umbriel desde fuente con dependencias best-effort.

## Modos de fondo

El comportamiento es explícito, así que un fondo fijo nunca vuelve a Bing sin avisar.

```bash
# Mantener el fondo actual y desactivar la automatización de Bing
noct wallpaper fixed

# Fijar un archivo concreto
noct wallpaper fixed ~/Pictures/Wallpapers/wallhaven-21j6og.png

# Activar Bing diario: escritorio + greeter se actualizan cada día
noct wallpaper bing-daily

# Aplicar la imagen de Bing de hoy una vez y fijarla
noct wallpaper bing

# Ver el modo activo
noct wallpaper status
```

- `fixed` desactiva el timer de Bing y el antiguo plugin `nzlov/daily-wallpaper`.
- `bing-daily` instala timers de usuario de systemd:
  - `noct-bing-wallpaper.timer` descarga y aplica Bing una vez al día.
  - `noct-wallpaper-greeter.timer` mantiene el greeter sincronizado cuando cambia el fondo.
- El actualizador de Bing de Noct sustituye al plugin antiguo para evitar dos procesos peleando por el fondo.

## Efectos y widget de Noctalia

```bash
noct effects list
noct effects apply mono
noct effects apply matrix
noct effects intensity 60
noct effects tint "#7cffb2"
noct effects tint none

# Overrides por slot, mezclados sobre cualquier preset
noct effects mix show
noct effects mix set cursor trail
noct effects mix set screen noct-mono
noct effects mix set border off
noct effects mix clear
```

El widget se instala con:

```bash
noct effects widget
noctalia msg panel-toggle noct/effects:panel
```

Incluye:

- selector de presets;
- panel **Mezclar efectos** con slots independientes de `border`, `cursor`, `screen`, `window` y animaciones;
- tooltips que describen cada efecto;
- controles de tinte e intensidad de Mono;
- traducciones en inglés y español (`translations/en.json`, `translations/es.json`).

## Atajos de captura

La configuración de capturas de Noctalia vive en `~/.local/state/noctalia/settings.toml`, bajo `[shell.screenshot]`.

Comandos útiles:

```bash
noctalia msg screenshot-region       # región interactiva
noctalia msg screenshot-window       # ventana enfocada
noctalia msg screenshot-fullscreen   # monitor enfocado
noctalia msg screenshot-fullscreen all
noctalia msg screenshot-annotate     # congelar y anotar
```

El instalador añade estos atajos cuando están libres:

| Tecla | Acción |
| --- | --- |
| `Print` | Captura de región |
| `Shift+Print` | Captura de ventana |
| `Ctrl+Print` | Captura de monitor |
| `Alt+Print` | Captura con anotación |

Configuración por defecto: PNG, calidad 90, copiar al portapapeles, guardar en `~/Pictures/Screenshots`, anotación y confirmación de región activadas.

## Resumen de la CLI

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

## Rutas de configuración

| Ruta | Propósito |
| --- | --- |
| `~/.config/umbriel/config.toml` | Configuración base de Umbriel e includes |
| `~/.config/umbriel/noct/keybinds.toml` | Atajos gestionados por Noct |
| `~/.config/umbriel/noct/bridge.toml` | Fragmento puente Niri ↔ Umbriel |
| `~/.config/umbriel/noct/effects.toml` | Preset y mezcla de efectos activos |
| `~/.config/umbriel/noct/overrides.toml` | Overrides de mezcla por slot |
| `~/.config/noct/wallpaper.toml` | Modo de fondo y ruta fijada |
| `~/.config/noct/current.json` | Último tema aplicado por Noct |
| `~/.local/share/noctalia/plugins/effects/` | Plugin de Noctalia instalado |
| `~/.local/state/noctalia/settings.toml` | Ajustes de Noctalia (capturas, notificaciones, fondo, tema) |

## Compatibilidad con Umbriel

Las versiones actuales de Umbriel usan acciones de atajo separadas por espacios:

```toml
"Mod+Return" = "spawn kitty"
"Mod+1" = "workspace-switch 1"
"Mod+Shift+1" = "window-move-to-workspace 1"
```

Noct y el instalador migran la sintaxis antigua con dos puntos:

```toml
"Mod+Return" = "spawn:kitty"
"Mod+1" = "workspace-switch:1"
```

También migran `input.focus.follows_mouse = true` a `"sloppy"` y `umbriel msg output-enable:eDP-1` a `output-enable eDP-1`.

## Lanzador Android

El directorio `android/` contiene un lanzador Jetpack Compose:

- inicio solo de widgets con bloques reordenables;
- carrusel de recientes con apps activas aproximadas;
- cajón de apps solo de texto;
- ajustes y sincronización de tema/fondo.

Ejecuta `noct sync serve` en el escritorio e introduce la URL y el token en la pantalla **Sync** del lanzador.

## Noga Linux

**Noga Linux** es la distribución de referencia de Noct: una ISO live mínima basada en CachyOS con Umbriel, Noctalia y Noct ya configurados (Niri como fallback). Mantiene los repositorios, el llavero y los nombres de paquetes de CachyOS, así que las actualizaciones siguen viniendo de CachyOS.

El **escritorio Rakia** es la configuración de Umbriel + Noctalia + Noct que incluye: mosaico desplazable, espacio infinito y la capa de temas de Noct. La sesión se publica como `rakia.desktop`. Usa por defecto el tema oscuro, el fondo de Noga y la marca Kepler en el lanzador y el dock.

Estado actual:

- la ISO live arranca en BIOS (syslinux) y UEFI (systemd-boot), con autologin a Umbriel + Noctalia;
- las ediciones slim y NVIDIA se generan desde el mismo perfil;
- la edición NVIDIA arranca en hardware NVIDIA real (dos monitores, NVML/nvidia-utils);
- el instalador gráfico (Calamares) instala el sistema offline y escribe GRUB para BIOS y UEFI;
- una capa de compatibilidad valida y repara la configuración tras actualizar Umbriel/Noctalia;
- herramientas de CachyOS incluidas: `cachy-update`, `cachyos-packageinstaller` y `cachyos-kernel-manager`;
- los USB y discos externos se montan automáticamente en el live y en el sistema instalado;
- el plugin de efectos de Noct está incluido en la imagen live y se instala con `noct effects widget`.

Instrucciones de compilación, grabación y prueba: **[docs/INSTALL.es.md#noga-linux](docs/INSTALL.es.md#noga-linux)** · **[English](docs/INSTALL.md#noga-linux)**.
Las previews de Noga se publican en [GitHub Releases](https://github.com/christiandavidfs/noct/releases).

## Documentación

- [Guía de instalación (Español)](docs/INSTALL.es.md)
- [Installation guide (English)](docs/INSTALL.md)
- [README in English](README.md)

## Desarrollo

```bash
cargo fmt
cargo clippy --all-targets
cargo test
cargo build --release -p noct-cli
```

Lint del plugin de Noctalia:

```bash
noctalia plugins lint ~/.local/share/noctalia/plugins/effects
```

## Licencia

MIT. Ver [LICENSE](LICENSE).
