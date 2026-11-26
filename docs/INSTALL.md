# Installation guide

**English** · [Español](INSTALL.es.md)

This guide covers two things:

1. installing **Noct** on an existing Wayland desktop;
2. building, booting and testing **Nocturne Linux**, the reference live ISO.

If you only want the integration layer, start with [Noct](#noct). If you want the full
desktop, jump to [Nocturne Linux](#nocturne-linux).

---

## Noct

### Requirements

- Linux with Wayland (Niri, Umbriel, Hyprland, Sway…).
- Noctalia and Umbriel. The installer can install or update both.
- `git`, `curl` and a normal user with `sudo` access.
- Rust 1.80+ to build `noct`. The installer bootstraps Rust with `rustup` if it is missing.
- A running Noctalia session for runtime commands (`noctalia msg ...`).

### Quick install

```bash
git clone https://github.com/christiandavidfs/noct.git
cd noct

# 1. See the full plan without touching the system
./install.sh --dry-run

# 2. Install/update Noctalia, Umbriel, Noct and the Noctalia plugin
./install.sh --wallpaper fixed
```

`install.sh` is idempotent: running it again updates what is already there. It installs
`noct` in `~/.local/bin` by default and keeps its configuration in `~/.config/noct`.

After it finishes, log out and log back in selecting **Umbriel** as the session. If you
prefer to keep your current compositor, Noct also works with Niri; the bridge and keybind
commands are compositor-aware.

### What the installer does

- installs or updates Noctalia and Umbriel (package, repository or source build depending
  on the distribution);
- builds and installs the `noct` CLI;
- installs the Noctalia plugin `noct/effects` and enables its control-center tile;
- writes the managed Umbriel fragments (`~/.config/umbriel/noct/*`);
- configures the screenshot keybinds when they are free;
- applies the wallpaper mode you chose (`fixed` by default).

### Post-install checks

```bash
noct doctor
noct wallpaper status
noct effects current
noctalia msg panel-toggle noct/effects:panel
```

`noct doctor` reports missing pieces (Noctalia socket, greeter, Umbriel includes, plugin
state) and can repair some of them with `noct doctor --fix`.

### Installer options

| Option | Meaning |
| --- | --- |
| `--dry-run` | Print commands without running them |
| `--yes` | Do not ask for confirmation |
| `--user` | Install Noctalia/Umbriel under `~/.local` (no sudo for them) |
| `--prefix PATH` | Noct install prefix (default `~/.local`) |
| `--system-prefix PATH` | Noctalia/Umbriel prefix (default `/usr/local`) |
| `--no-system` | Skip system dependency installation |
| `--no-noctalia` | Do not install/update Noctalia |
| `--no-umbriel` | Do not install/update Umbriel |
| `--from-source` | Build Noctalia and Umbriel from source |
| `--wallpaper fixed\|bing-daily` | Wallpaper mode |
| `--wallpaper-path PATH` | Fixed wallpaper to apply |
| `--with-greeter` | Install `noctalia-greeter` (does not configure greetd) |
| `--no-migrate` | Do not migrate old Umbriel keybind syntax |

### Distribution notes

| Distribution | What the installer uses |
| --- | --- |
| Arch | `noctalia` from the official repositories, `umbriel-git` from AUR (`paru`/`yay`) |
| Debian/Ubuntu | Noctalia APT repository, backports when needed, `xdg-desktop-portal-umbriel` |
| Fedora | `noctalia`, Terra’s `umbriel-nightly`, source-build fallback |
| openSUSE | Noctalia OBS repository |
| Other | Source builds with best-effort dependencies |

### Updating

```bash
cd noct
git pull
./install.sh --wallpaper fixed
cargo build --release -p noct-cli   # only if you build manually
```

The plugin lives in `~/.local/share/noctalia/plugins/effects`. Re-run
`noct effects widget` after updating if you want to force a reinstall and re-enable it.

### Troubleshooting

**`noctalia msg` says it cannot connect**

Noctalia is not running in the current session, or `XDG_RUNTIME_DIR` / `WAYLAND_DISPLAY`
are not set. Start Noctalia inside your Wayland session and try again.

**The Noct Effects panel is not in the list**

Check that the plugin is loaded and enabled:

```bash
noctalia msg plugins list | grep noct
noctalia msg plugins enable noct/effects
noctalia msg panel-toggle noct/effects:panel
```

`noct effects widget` now runs the enable step automatically. If Noctalia was not running
when you installed it, run the commands above after logging in.

**Old Umbriel keybind syntax**

```bash
noct keys check
noct keys migrate --dry-run
noct keys migrate
```

**Qt/GL applications do not start under Wayland**

```bash
noct run <command>
```

`noct run` applies the Qt/GL fallback environment for the command.

**NVIDIA**

Use the proprietary/open driver with `nvidia_drm.modeset=1` (and `nvidia_drm.fbdev=1` on
recent drivers). Noct does not replace your driver; it only manages themes, effects and
configuration.

---

## Nocturne Linux

### What it is

Nocturne Linux is a minimal, CachyOS-based live ISO with **Niri + Noctalia + Noct** ready
to use. It keeps CachyOS repositories, keyring and package names, so updates continue to
come from CachyOS. The branding is Nocturne; the packages are not rebuilt or renamed.

Current status:

- **live ISO works** on BIOS (syslinux) and UEFI (systemd-boot);
- autologin into Niri + Noctalia as the `nocturne` user;
- **slim** edition for Intel/AMD and **NVIDIA** edition with `nvidia-open`;
- **the graphical installer (Calamares) is the next milestone**. The current ISO is a live
  environment, not yet a complete installer.

> Do not install the current pre-release ISO over a disk you care about. Use a virtual
> machine or a spare disk until the installer milestone is published.

### Download

Prebuilt ISOs are published under
[GitHub Releases](https://github.com/christiandavidfs/noct/releases). The current live
preview is
[nocturne-2026.11-live](https://github.com/christiandavidfs/noct/releases/tag/nocturne-2026.11-live).
It is a live environment only; the installer is the next milestone. You can also build from
source with the steps below.

### Build requirements

Build on CachyOS (recommended) or on an Arch system with the CachyOS repositories and
`cachyos-keyring` configured.

```bash
sudo pacman -S --needed archiso syslinux libisoburn squashfs-tools dosfstools mtools
```

You also need:

- ~15 GB free disk space;
- root (the build creates a chroot and loop devices);
- a working CachyOS mirror (the profile pins a few known-good mirrors for the build);
- `target/release/noct` if you want the CLI inside the live image.

### Build the ISO

```bash
git clone https://github.com/christiandavidfs/noct.git
cd noct

# Validate the profile and package list without building
./iso/build.sh --check

# Build the CLI that will be copied into the live image
cargo build --release -p noct-cli

# Slim edition (recommended for Intel/AMD)
sudo ./iso/build.sh --slim

# NVIDIA edition (nvidia-open + nvidia-utils in the live image)
sudo ./iso/build.sh --nvidia
```

Output:

```text
build/iso-out/nocturne-YYYY.MM.DD-x86_64.iso
build/iso-out/nocturne-nvidia-YYYY.MM.DD-x86_64.iso
build/iso-out/nocturne-slim.iso       # symlink to the latest slim build
build/iso-out/nocturne-nvidia.iso     # symlink to the latest NVIDIA build
```

Each ISO gets a `.sha256` file. The build log is written to
`build/iso-slim.log` or `build/iso-nvidia.log`.

### Write the ISO to a USB stick

```bash
# Replace /dev/sdX with the USB device, NOT a partition
sudo dd if=build/iso-out/nocturne-slim.iso of=/dev/sdX bs=4M status=progress oflag=sync
sync
```

You can also use **balenaEtcher** or **Ventoy**. The ISO is hybrid: it boots from USB and
from optical media.

### Boot

- **UEFI**: select the USB device in the firmware boot menu. The systemd-boot menu shows
  *Nocturne Linux (UEFI)* and *Nocturne Linux (modo seguro)*.
- **BIOS/CSM**: the syslinux menu shows the same options plus *Arrancar sistema existente*,
  *Reiniciar* and *Apagar*.

**Secure Boot** must be disabled: the ISO is not signed. If you need Secure Boot, sign the
kernel and bootloader with `sbctl` or use a shim; that is outside the current scope.

### Live session

| | |
| --- | --- |
| User | `nocturne` (autologin, no password) |
| Sudo | passwordless in the live session |
| Rescue console | `Ctrl+Alt+F2`, login `root` (no password) |
| Terminal | `Mod+Return` (foot) |
| Launcher | `Mod+D` |
| Control center | `Mod+S` |
| Noctalia settings | `Mod+Shift+S` |
| Wallpaper | `Mod+Shift+Return` |
| Lock screen | `Mod+Alt+L` |
| Session menu | `Mod+Shift+Q` |
| Screenshot (region) | `Print` |
| Screenshot (window) | `Shift+Print` |
| Screenshot (monitor) | `Ctrl+Print` |
| Quit Niri | `Ctrl+Alt+Delete` |

Wi-Fi:

```bash
nmtui
# or
nmcli device wifi list
nmcli device wifi connect "SSID" password "password"
```

USB sticks and external SSDs are detected and mounted automatically under
`/run/media/$USER/<label>` (`udisks2` + `udiskie`). To unmount or power off:

```bash
udisksctl unmount -b /dev/sdX1
udisksctl power-off -b /dev/sdX
```

The live system uses CachyOS + Arch repositories. The pacman keyring is created in a tmpfs
at boot, so the ISO stays small and can still install packages:

```bash
sudo pacman -Syu
```

### Install to disk

The Calamares integration is in progress. When it lands, the ISO will boot into a graphical
installer with:

- disk selection and encryption;
- user creation;
- locale and keyboard;
- Nocturne package groups (`noct-base`, `noct-desktop`, `noct-tools`, `noct-nvidia`,
  `noct-gaming`, `noct-dev`, `noct-office`);
- bootloader installation (systemd-boot or GRUB).

Until then, use the live session to evaluate Nocturne and Noct. The installed-system path
will be documented here as soon as the installer is testable.

### Test in QEMU

Niri needs a real GL renderer. With QEMU’s plain `virtio-vga` (no 3D) Niri skips software
EGL and you get a black screen even though the system booted. Use `virtio-vga-gl`:

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

For a headless boot, replace `-display gtk,gl=on` with
`-display egl-headless,rendernode=/dev/dri/renderD128` and use the serial console
(`console=ttyS0`) or the QMP monitor to inspect the guest.

To test the BIOS path, remove the two `pflash` drives and keep `-boot order=d`; SeaBIOS
boots the syslinux menu.

### Editions

| Edition | Live packages | Use |
| --- | --- | --- |
| `slim` | Intel/AMD firmware, Mesa, Vulkan | Most laptops and desktops |
| `nvidia` | adds `nvidia-open`, `nvidia-utils`, `nvidia-prime`, `nvidia-settings` | NVIDIA laptops/desktops |

The NVIDIA edition adds `nvidia_drm.modeset=1 nvidia_drm.fbdev=1` to the kernel command
line.

### Known limitations of the current pre-release

- No graphical installer yet (Calamares is the next milestone).
- The ISO is unsigned; Secure Boot must be disabled.

---

## License

MIT. See [LICENSE](../LICENSE).
