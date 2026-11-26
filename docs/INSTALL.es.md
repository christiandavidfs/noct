# Guía de instalación

[English](INSTALL.md) · **Español**

Esta guía cubre dos cosas:

1. instalar **Noct** en un escritorio Wayland existente;
2. compilar, arrancar y probar **Nocturne Linux**, la ISO live de referencia.

Si solo quieres la capa de integración, empieza por [Noct](#noct). Si quieres el escritorio
completo, salta a [Nocturne Linux](#nocturne-linux).

---

## Noct

### Requisitos

- Linux con Wayland (Niri, Umbriel, Hyprland, Sway…).
- Noctalia y Umbriel. El instalador puede instalar o actualizar ambos.
- `git`, `curl` y un usuario normal con acceso a `sudo`.
- Rust 1.80+ para compilar `noct`. El instalador instala Rust con `rustup` si falta.
- Una sesión de Noctalia en ejecución para comandos en caliente (`noctalia msg ...`).

### Instalación rápida

```bash
git clone https://github.com/christiandavidfs/noct.git
cd noct

# 1. Ver el plan completo sin tocar el sistema
./install.sh --dry-run

# 2. Instalar/actualizar Noctalia, Umbriel, Noct y el plugin de Noctalia
./install.sh --wallpaper fixed
```

`install.sh` es idempotente: ejecutarlo de nuevo actualiza lo que ya existe. Instala `noct`
en `~/.local/bin` por defecto y guarda su configuración en `~/.config/noct`.

Al terminar, cierra sesión y vuelve a entrar eligiendo **Umbriel** como sesión. Si prefieres
seguir con tu compositor actual, Noct también funciona con Niri; los comandos de puente y
atajos detectan el compositor.

### Qué hace el instalador

- instala o actualiza Noctalia y Umbriel (paquete, repositorio o compilación según la
  distribución);
- compila e instala la CLI `noct`;
- instala el plugin de Noctalia `noct/effects` y activa su tile en el centro de control;
- escribe los fragmentos gestionados de Umbriel (`~/.config/umbriel/noct/*`);
- configura los atajos de captura cuando están libres;
- aplica el modo de fondo elegido (`fixed` por defecto).

### Comprobaciones posteriores

```bash
noct doctor
noct wallpaper status
noct effects current
noctalia msg panel-toggle noct/effects:panel
```

`noct doctor` informa de piezas que faltan (socket de Noctalia, greeter, includes de
Umbriel, estado del plugin) y puede reparar algunas con `noct doctor --fix`.

### Opciones del instalador

| Opción | Significado |
| --- | --- |
| `--dry-run` | Muestra los comandos sin ejecutarlos |
| `--yes` | No pide confirmación |
| `--user` | Instala Noctalia/Umbriel en `~/.local` (sin sudo para ellos) |
| `--prefix RUTA` | Prefijo de Noct (por defecto `~/.local`) |
| `--system-prefix RUTA` | Prefijo de Noctalia/Umbriel (por defecto `/usr/local`) |
| `--no-system` | No instala dependencias del sistema |
| `--no-noctalia` | No instala/actualiza Noctalia |
| `--no-umbriel` | No instala/actualiza Umbriel |
| `--from-source` | Compila Noctalia y Umbriel desde fuente |
| `--wallpaper fixed\|bing-daily` | Modo de fondo |
| `--wallpaper-path RUTA` | Fondo fijo que aplicar |
| `--with-greeter` | Instala `noctalia-greeter` (no configura greetd) |
| `--no-migrate` | No migra la sintaxis antigua de Umbriel |

### Notas por distribución

| Distribución | Qué usa el instalador |
| --- | --- |
| Arch | `noctalia` de los repos oficiales, `umbriel-git` de AUR (`paru`/`yay`) |
| Debian/Ubuntu | Repositorio APT de Noctalia, backports si hacen falta, `xdg-desktop-portal-umbriel` |
| Fedora | `noctalia`, `umbriel-nightly` de Terra, compilación desde fuente como respaldo |
| openSUSE | Repositorio OBS de Noctalia |
| Otras | Compilación desde fuente con dependencias best-effort |

### Actualizar

```bash
cd noct
git pull
./install.sh --wallpaper fixed
cargo build --release -p noct-cli   # solo si compilas a mano
```

El plugin vive en `~/.local/share/noctalia/plugins/effects`. Vuelve a ejecutar
`noct effects widget` tras actualizar si quieres forzar la reinstalación y reactivación.

### Solución de problemas

**`noctalia msg` no conecta**

Noctalia no está corriendo en la sesión actual, o `XDG_RUNTIME_DIR` / `WAYLAND_DISPLAY` no
están definidos. Arranca Noctalia dentro de tu sesión Wayland y vuelve a intentarlo.

**El panel Noct Effects no aparece en la lista**

Comprueba que el plugin está cargado y activo:

```bash
noctalia msg plugins list | grep noct
noctalia msg plugins enable noct/effects
noctalia msg panel-toggle noct/effects:panel
```

`noct effects widget` ya ejecuta la activación automáticamente. Si Noctalia no estaba
corriendo al instalar, ejecuta los comandos anteriores tras iniciar sesión.

**Sintaxis antigua de atajos de Umbriel**

```bash
noct keys check
noct keys migrate --dry-run
noct keys migrate
```

**Las apps Qt/GL no arrancan en Wayland**

```bash
noct run <comando>
```

`noct run` aplica el entorno de respaldo Qt/GL para el comando.

**NVIDIA**

Usa el driver propietario/abierto con `nvidia_drm.modeset=1` (y `nvidia_drm.fbdev=1` en
drivers recientes). Noct no sustituye tu driver; solo gestiona temas, efectos y
configuración.

---

## Nocturne Linux

### Qué es

Nocturne Linux es una ISO live mínima basada en CachyOS con **Niri + Noctalia + Noct**
listos para usar. Mantiene los repositorios, el llavero y los nombres de paquetes de
CachyOS, así que las actualizaciones siguen viniendo de CachyOS. La marca es Nocturne; los
paquetes no se renombran ni se recompilan.

Estado actual:

- **la ISO live funciona** en BIOS (syslinux) y UEFI (systemd-boot);
- autologin a Niri + Noctalia con el usuario `nocturne`;
- edición **slim** para Intel/AMD y edición **NVIDIA** con `nvidia-open`;
- **el instalador gráfico (Calamares) es el siguiente paso**. La ISO actual es un entorno
  live, todavía no un instalador completo.

> No instales la ISO pre-release actual sobre un disco que te importe. Usa una máquina
> virtual o un disco de repuesto hasta que se publique el hito del instalador.

### Descarga

Las ISOs precompiladas se publicarán en
[GitHub Releases](https://github.com/christiandavidfs/noct/releases). Todavía no hay ISO
pública: hasta el hito del instalador, compílala desde fuente con los pasos de abajo.

### Requisitos de compilación

Compila en CachyOS (recomendado) o en un Arch con los repositorios de CachyOS y
`cachyos-keyring` configurados.

```bash
sudo pacman -S --needed archiso syslinux libisoburn squashfs-tools dosfstools mtools
```

También necesitas:

- ~15 GB de disco libre;
- root (la compilación crea un chroot y dispositivos loop);
- un mirror de CachyOS funcionando (el perfil fija algunos mirrors conocidos para la
  compilación);
- `target/release/noct` si quieres la CLI dentro de la imagen live.

### Compilar la ISO

```bash
git clone https://github.com/christiandavidfs/noct.git
cd noct

# Validar el perfil y la lista de paquetes sin compilar
./iso/build.sh --check

# Compilar la CLI que se copiará a la imagen live
cargo build --release -p noct-cli

# Edición slim (recomendada para Intel/AMD)
sudo ./iso/build.sh --slim

# Edición NVIDIA (nvidia-open + nvidia-utils en la imagen live)
sudo ./iso/build.sh --nvidia
```

Salida:

```text
build/iso-out/nocturne-AAAA.MM.DD-x86_64.iso
build/iso-out/nocturne-nvidia-AAAA.MM.DD-x86_64.iso
build/iso-out/nocturne-slim.iso       # enlace a la última build slim
build/iso-out/nocturne-nvidia.iso     # enlace a la última build NVIDIA
```

Cada ISO genera un archivo `.sha256`. El log de compilación queda en
`build/iso-slim.log` o `build/iso-nvidia.log`.

### Grabar la ISO en un USB

```bash
# Sustituye /dev/sdX por el dispositivo USB, NO por una partición
sudo dd if=build/iso-out/nocturne-slim.iso of=/dev/sdX bs=4M status=progress oflag=sync
sync
```

También puedes usar **balenaEtcher** o **Ventoy**. La ISO es híbrida: arranca desde USB y
desde medio óptico.

### Arranque

- **UEFI**: elige el USB en el menú de arranque del firmware. El menú de systemd-boot
  muestra *Nocturne Linux (UEFI)* y *Nocturne Linux (modo seguro)*.
- **BIOS/CSM**: el menú de syslinux muestra las mismas opciones más *Arrancar sistema
  existente*, *Reiniciar* y *Apagar*.

**Secure Boot** debe estar desactivado: la ISO no está firmada. Si necesitas Secure Boot,
firma el kernel y el bootloader con `sbctl` o usa un shim; queda fuera del alcance actual.

### Sesión live

| | |
| --- | --- |
| Usuario | `nocturne` (autologin, sin contraseña) |
| Sudo | sin contraseña en la sesión live |
| Consola de rescate | `Ctrl+Alt+F2`, login `root` (sin contraseña) |
| Terminal | `Mod+Return` (foot) |
| Lanzador | `Mod+D` |
| Centro de control | `Mod+S` |
| Ajustes de Noctalia | `Mod+Shift+S` |
| Fondo de pantalla | `Mod+Shift+Return` |
| Bloquear pantalla | `Mod+Alt+L` |
| Menú de sesión | `Mod+Shift+Q` |
| Captura (región) | `Print` |
| Captura (ventana) | `Shift+Print` |
| Captura (monitor) | `Ctrl+Print` |
| Salir de Niri | `Ctrl+Alt+Delete` |

Wi-Fi:

```bash
nmtui
# o
nmcli device wifi list
nmcli device wifi connect "SSID" password "contraseña"
```

El sistema live usa los repositorios de CachyOS + Arch. El llavero de pacman se crea en un
tmpfs al arrancar, así que la ISO sigue siendo pequeña y aun así puede instalar paquetes:

```bash
sudo pacman -Syu
```

### Instalar en disco

La integración con Calamares está en curso. Cuando llegue, la ISO arrancará en un
instalador gráfico con:

- selección de disco y cifrado;
- creación de usuario;
- idioma y teclado;
- grupos de paquetes de Nocturne (`noct-base`, `noct-desktop`, `noct-tools`, `noct-nvidia`,
  `noct-gaming`, `noct-dev`, `noct-office`);
- instalación del bootloader (systemd-boot o GRUB).

Hasta entonces, usa la sesión live para evaluar Nocturne y Noct. La ruta de instalación en
disco se documentará aquí en cuanto el instalador sea probable.

### Probar en QEMU

Niri necesita un renderer GL real. Con el `virtio-vga` normal de QEMU (sin 3D) Niri descarta
el EGL de software y verás una pantalla negra aunque el sistema haya arrancado. Usa
`virtio-vga-gl`:

```bash
cp /usr/share/edk2/x64/OVMF_VARS.4m.fd /tmp/OVMF_VARS.fd

qemu-system-x86_64 \
  -machine q35,smm=on,accel=kvm \
  -cpu host -smp 4 -m 3072 \
  -device virtio-vga-gl -display gtk,gl=on \
  -drive if=pflash,format=raw,readonly=on,file=/usr/share/edk2/x64/OVMF_CODE.4m.fd \
  -drive if=pflash,format=raw,file=/tmp/OVMF_VARS.fd \
  -drive file=build/iso-out/nocturne-slim.iso,media=cdrom,readonly=on \
  -boot order=d
```

Para arranque sin ventana, cambia `-display gtk,gl=on` por
`-display egl-headless,rendernode=/dev/dri/renderD128` y usa la consola serie
(`console=ttyS0`) o el monitor QMP para inspeccionar el invitado.

Para probar la ruta BIOS, quita las dos unidades `pflash` y mantén `-boot order=d`; SeaBIOS
arranca el menú de syslinux.

### Ediciones

| Edición | Paquetes live | Uso |
| --- | --- | --- |
| `slim` | firmware Intel/AMD, Mesa, Vulkan | La mayoría de portátiles y equipos |
| `nvidia` | añade `nvidia-open`, `nvidia-utils`, `nvidia-prime`, `nvidia-settings` | Portátiles/equipos NVIDIA |

La edición NVIDIA añade `nvidia_drm.modeset=1 nvidia_drm.fbdev=1` a la línea de comandos
del kernel.

### Limitaciones conocidas de la pre-release actual

- Todavía no hay instalador gráfico (Calamares es el siguiente hito).
- La ISO no está firmada; Secure Boot debe estar desactivado.

---

## Licencia

MIT. Ver [LICENSE](../LICENSE).
