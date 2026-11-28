#!/usr/bin/env bash
# Instalador de Noct: Noctalia + Umbriel + Noct + plugin + configuración.
#
# Uso:
#   ./install.sh [opciones]
#
# Opciones:
#   --dry-run                 Muestra los comandos sin ejecutarlos
#   --yes                     No pregunta nada
#   --lang en|es              Idioma de la salida (por defecto: locale)
#   --user                    Instala todo en ~/.local (sin sudo para Noctalia/Umbriel)
#   --prefix RUTA             Prefijo de Noct (por defecto ~/.local)
#   --system-prefix RUTA      Prefijo de Noctalia/Umbriel (por defecto /usr/local)
#   --no-system               No instala dependencias del sistema
#   --no-noctalia             No instala/actualiza Noctalia
#   --no-umbriel              No instala/actualiza Umbriel
#   --no-udisks               No configura el montaje automático de USB
#   --from-source             Fuerza compilación de Noctalia y Umbriel
#   --wallpaper fixed|bing-daily
#   --wallpaper-path RUTA     Wallpaper fijo a aplicar
#   --no-migrate              No migra la sintaxis antigua de Umbriel
#   -h, --help
#
# El modo wallpaper es explícito:
#   fixed       deja el fondo elegido y desactiva el actualizador de Bing
#   bing-daily  actualiza Bing cada día y sincroniza escritorio + greeter
#
set -euo pipefail

REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PREFIX="${NOCT_PREFIX:-$HOME/.local}"
SYSTEM_PREFIX="${NOCT_SYSTEM_PREFIX:-/usr/local}"
CACHE_DIR="${XDG_CACHE_HOME:-$HOME/.cache}/noct-install"
export PATH="$PREFIX/bin:$PATH"

DRY_RUN=0
ASSUME_YES=0
USER_ONLY=0
NO_SYSTEM=0
WITH_NOCTALIA=1
WITH_UMBRIEL=1
FROM_SOURCE=0
MIGRATE=1
WITH_GREETER=0
WITH_UDISKS=1
HIDE_GREETER_LOGO=1
DEBIAN_REPO_READY=0
SUSE_REPO_READY=0
WALLPAPER_MODE=""
WALLPAPER_PATH=""

# Idioma de la interfaz: --lang, NOCT_LANG o el locale del sistema.
LANG_MODE="${NOCT_LANG:-}"
if [ -z "$LANG_MODE" ]; then
  case "${LC_ALL:-${LC_MESSAGES:-${LANG:-}}}" in
    es*) LANG_MODE=es ;;
    *) LANG_MODE=en ;;
  esac
fi
case "$LANG_MODE" in
  en|es) ;;
  *) printf 'error: --lang must be en or es / --lang debe ser en o es\n' >&2; exit 2 ;;
esac

# say "English" "Español" [args...] — imprime el texto en el idioma activo.
say() {
  local en="$1" es="$2"
  shift 2
  if [ "$LANG_MODE" = es ]; then
    printf "$es" "$@"
  else
    printf "$en" "$@"
  fi
}

usage() {
  if [ "$LANG_MODE" = es ]; then
    cat <<'EOF'
Instalador de Noct: Noctalia + Umbriel + Noct + plugin + configuración.

Uso:
  ./install.sh [opciones]

Opciones:
  --dry-run                 Muestra los comandos sin ejecutarlos
  --yes                     No pregunta nada
  --lang en|es              Idioma de la salida (por defecto: locale)
  --user                    Instala todo en ~/.local (sin sudo para Noctalia/Umbriel)
  --prefix RUTA             Prefijo de Noct (por defecto ~/.local)
  --system-prefix RUTA      Prefijo de Noctalia/Umbriel (por defecto /usr/local)
  --no-system               No instala dependencias del sistema
  --no-noctalia             No instala/actualiza Noctalia
  --no-umbriel              No instala/actualiza Umbriel
  --no-udisks               No configura el montaje automático de USB
  --keep-greeter-logo       No oculta el logo del greeter de Noctalia
  --from-source             Fuerza compilación de Noctalia y Umbriel
  --wallpaper fixed|bing-daily
  --wallpaper-path RUTA     Wallpaper fijo a aplicar
  --with-greeter            Instala noctalia-greeter (no configura greetd)
  --no-migrate              No migra la sintaxis antigua de Umbriel
  -h, --help

El modo wallpaper es explícito:
  fixed       deja el fondo elegido y desactiva el actualizador de Bing
  bing-daily  actualiza Bing cada día y sincroniza escritorio + greeter

El montaje de USB y discos externos usa udisks2 + udiskie. Se configura un
servicio de usuario para que arranque con la sesión. --no-udisks lo omite.

Si noctalia-greeter está instalado, se oculta su logo en el login. Usa
--keep-greeter-logo para conservarlo.
EOF
  else
    cat <<'EOF'
Noct installer: Noctalia + Umbriel + Noct + plugin + configuration.

Usage:
  ./install.sh [options]

Options:
  --dry-run                 Print commands without running them
  --yes                     Do not ask for confirmation
  --lang en|es              Output language (default: system locale)
  --user                    Install everything under ~/.local (no sudo for Noctalia/Umbriel)
  --prefix PATH             Noct install prefix (default ~/.local)
  --system-prefix PATH      Noctalia/Umbriel prefix (default /usr/local)
  --no-system               Skip system dependency installation
  --no-noctalia             Do not install/update Noctalia
  --no-umbriel              Do not install/update Umbriel
  --no-udisks               Do not configure automatic USB mounting
  --keep-greeter-logo       Keep the Noctalia greeter logo
  --from-source             Force Noctalia and Umbriel source builds
  --wallpaper fixed|bing-daily
  --wallpaper-path PATH     Fixed wallpaper to apply
  --with-greeter            Install noctalia-greeter (does not configure greetd)
  --no-migrate              Do not migrate old Umbriel keybind syntax
  -h, --help

Wallpaper modes are explicit:
  fixed       keep the chosen wallpaper and disable the Bing updater
  bing-daily  update Bing every day and sync desktop + greeter

USB sticks and external drives use udisks2 + udiskie. A user service starts
the automounter with the session. --no-udisks skips it.

If noctalia-greeter is installed, its logo is hidden on the login screen by
default. Use --keep-greeter-logo to keep it.
EOF
  fi
}

log() { printf '\033[1;34m==>\033[0m %s\n' "$*"; }
warn() { printf '\033[1;33m%s:\033[0m %s\n' "$(say warning aviso)" "$*" >&2; }
die() { printf '\033[1;31m%s:\033[0m %s\n' "$(say error error)" "$*" >&2; exit 1; }

run() {
  if [ "$DRY_RUN" -eq 1 ]; then
    printf '  + %s\n' "$*"
  else
    "$@"
  fi
}

run_in() {
  local dir="$1"
  shift
  if [ "$DRY_RUN" -eq 1 ]; then
    printf '  + (cd %s && %s)\n' "$dir" "$*"
  else
    (cd "$dir" && "$@")
  fi
}

as_root() {
  if [ "$(id -u)" -eq 0 ]; then
    run "$@"
  elif [ "$USER_ONLY" -eq 1 ]; then
    die "$(say 'this operation needs root; drop --user or run it manually' 'esta operación necesita root; quita --user o ejecútala manualmente')"
  else
    run sudo "$@"
  fi
}

need_cmd() {
  command -v "$1" >/dev/null 2>&1
}

while [ $# -gt 0 ]; do
  case "$1" in
    --dry-run) DRY_RUN=1 ;;
    --yes|-y) ASSUME_YES=1 ;;
    --lang) shift; [ $# -gt 0 ] || die "$(say '--lang needs en or es' '--lang necesita en o es')"; LANG_MODE="$1" ;;
    --user) USER_ONLY=1 ;;
    --prefix) shift; [ $# -gt 0 ] || die "$(say '--prefix needs a path' '--prefix necesita una ruta')"; PREFIX="$1" ;;
    --system-prefix) shift; [ $# -gt 0 ] || die "$(say '--system-prefix needs a path' '--system-prefix necesita una ruta')"; SYSTEM_PREFIX="$1" ;;
    --no-system) NO_SYSTEM=1 ;;
    --no-noctalia) WITH_NOCTALIA=0 ;;
    --no-umbriel) WITH_UMBRIEL=0 ;;
    --no-udisks) WITH_UDISKS=0 ;;
    --keep-greeter-logo) HIDE_GREETER_LOGO=0 ;;
    --from-source) FROM_SOURCE=1 ;;
    --wallpaper) shift; [ $# -gt 0 ] || die "$(say '--wallpaper needs fixed or bing-daily' '--wallpaper necesita fixed o bing-daily')"; WALLPAPER_MODE="$1" ;;
    --wallpaper-path) shift; [ $# -gt 0 ] || die "$(say '--wallpaper-path needs a path' '--wallpaper-path necesita una ruta')"; WALLPAPER_PATH="$1" ;;
    --with-greeter) WITH_GREETER=1 ;;
    --no-migrate) MIGRATE=0 ;;
    -h|--help) usage; exit 0 ;;
    *) die "$(say "unknown option: $1 (use --help)" "opción desconocida: $1 (usa --help)")" ;;
  esac
  shift
done

case "$LANG_MODE" in
  en|es) ;;
  *) die "$(say '--lang must be en or es' '--lang debe ser en o es')" ;;
esac

case "$WALLPAPER_MODE" in
  ""|fixed|bing-daily) ;;
  *) die "$(say '--wallpaper must be fixed or bing-daily' '--wallpaper debe ser fixed o bing-daily')" ;;
esac

# ---------------------------------------------------------------------------
# Distribución y gestor de paquetes
# ---------------------------------------------------------------------------

if [ -r /etc/os-release ]; then
  # shellcheck disable=SC1091
  . /etc/os-release
fi
DISTRO_ID="${ID:-unknown}"
DISTRO_LIKE="${ID_LIKE:-}"
VERSION_CODENAME="${VERSION_CODENAME:-}"

case " $DISTRO_ID $DISTRO_LIKE " in
  *" arch "*|*" manjaro "*|*" endeavouros "*|*" cachyos "*) FAMILY=arch ;;
  *" debian "*|*" ubuntu "*|*" linuxmint "*|*" pop "*) FAMILY=debian ;;
  *" fedora "*|*" rhel "*|*" centos "*) FAMILY=fedora ;;
  *" opensuse"*|*" suse "*) FAMILY=suse ;;
  *" alpine "*) FAMILY=alpine ;;
  *" void "*) FAMILY=void ;;
  *" solus "*) FAMILY=solus ;;
  *) FAMILY=generic ;;
esac

if command -v pacman >/dev/null 2>&1; then PM=pacman
elif command -v apt-get >/dev/null 2>&1; then PM=apt
elif command -v dnf >/dev/null 2>&1; then PM=dnf
elif command -v zypper >/dev/null 2>&1; then PM=zypper
elif command -v apk >/dev/null 2>&1; then PM=apk
elif command -v xbps-install >/dev/null 2>&1; then PM=xbps
elif command -v eopkg >/dev/null 2>&1; then PM=eopkg
else PM=none
fi

log "$(say 'System: %s · family %s · packages %s' 'Sistema: %s · familia %s · paquetes %s' "${PRETTY_NAME:-$DISTRO_ID}" "$FAMILY" "$PM")"
if [ "$DRY_RUN" -eq 1 ]; then
  log "$(say 'Dry-run: nothing will be changed' 'Modo dry-run: no se cambiará nada')"
fi

pkg_update() {
  [ "$NO_SYSTEM" -eq 1 ] && return 0
  case "$PM" in
    pacman) as_root pacman -Sy --noconfirm ;;
    apt) as_root apt-get update ;;
    dnf) as_root dnf makecache ;;
    zypper) as_root zypper --non-interactive refresh ;;
    apk) as_root apk update ;;
    xbps) as_root xbps-install -S ;;
    *) return 0 ;;
  esac
}

pkg_install() {
  [ "$NO_SYSTEM" -eq 1 ] && { warn "$(say 'system dependencies skipped (--no-system)' 'dependencias del sistema omitidas (--no-system)')"; return 0; }
  case "$PM" in
    pacman) as_root pacman -S --needed --noconfirm "$@" ;;
    apt) as_root env DEBIAN_FRONTEND=noninteractive apt-get install -y "$@" ;;
    dnf) as_root dnf install -y "$@" ;;
    zypper) as_root zypper --non-interactive install "$@" ;;
    apk) as_root apk add "$@" ;;
    xbps) as_root xbps-install -Sy "$@" ;;
    eopkg) as_root eopkg install -y "$@" ;;
    *) warn "$(say 'no known package manager; install manually: %s' 'no hay gestor de paquetes conocido; instala a mano: %s' "$*")"; return 1 ;;
  esac
}

pkg_install_optional() {
  for pkg in "$@"; do
    pkg_install "$pkg" || warn "$(say 'could not install %s' 'no se pudo instalar %s' "$pkg")"
  done
}

ensure_rust() {
  if need_cmd cargo; then
    return 0
  fi
  if [ "$DRY_RUN" -eq 1 ]; then
    log "$(say 'Would install Rust with rustup' 'Instalaría Rust con rustup')"
    return 0
  fi
  pkg_install_optional curl ca-certificates
  if ! need_cmd rustup; then
    log "$(say 'Installing Rust with rustup' 'Instalando Rust con rustup')"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
  fi
  # shellcheck disable=SC1091
  [ -f "$HOME/.cargo/env" ] && . "$HOME/.cargo/env"
  need_cmd cargo || die "$(say 'cargo is not available; restart the terminal and run again' 'cargo no quedó disponible; reinicia la terminal y vuelve a ejecutar')"
}

# ---------------------------------------------------------------------------
# Noctalia y Umbriel
# ---------------------------------------------------------------------------

setup_debian_repo() {
  [ "$NO_SYSTEM" -eq 1 ] && return 0
  [ "$DEBIAN_REPO_READY" -eq 1 ] && return 0
  local src
  case "$VERSION_CODENAME" in
    trixie) src=trixie ;;
    sid|unstable) src=unstable ;;
    resolute) src=resolute ;;
    stonking) src=stonking ;;
    *)
      warn "$(say "Debian/Ubuntu '$VERSION_CODENAME' is not in Noctalia's list; skipping the APT repo" "Debian/Ubuntu '$VERSION_CODENAME' no está en la lista de Noctalia; se omite el repo APT")"
      return 1
      ;;
  esac
  log "$(say 'Adding the Noctalia APT repository (%s)' 'Añadiendo repositorio APT de Noctalia (%s)' "$src")"
  pkg_install_optional wget ca-certificates
  run wget -qO /tmp/nickh-archive-keyring.deb https://pkg.noctalia.dev/deb/nickh-archive-keyring.deb
  as_root dpkg -i /tmp/nickh-archive-keyring.deb
  as_root wget -qO "/etc/apt/sources.list.d/noctalia-${src}.sources" \
    "https://pkg.noctalia.dev/deb/noctalia-${src}.sources"
  as_root apt-get update
  DEBIAN_REPO_READY=1
}

add_debian_backports() {
  [ "$NO_SYSTEM" -eq 1 ] && return 0
  log "$(say 'Enabling Debian backports for Umbriel' 'Habilitando Debian backports para Umbriel')"
  as_root sh -c 'cat > /etc/apt/sources.list.d/debian-backports.sources <<EOF
Types: deb
URIs: http://deb.debian.org/debian
Suites: trixie-backports
Components: main
Signed-By: /usr/share/keyrings/debian-archive-keyring.gpg
EOF'
  as_root apt-get update
}

setup_terra_repo() {
  [ "$NO_SYSTEM" -eq 1 ] && return 0
  local ver
  ver="$(rpm -E %fedora 2>/dev/null || true)"
  [ -n "$ver" ] || { warn "$(say 'could not detect the Fedora version' 'no se pudo detectar la versión de Fedora')"; return 1; }
  warn "$(say 'Enabling the Terra repository (third party) for umbriel-nightly' 'Habilitando el repositorio Terra (tercero) para umbriel-nightly')"
  as_root dnf install -y --nogpgcheck --repofrompath "terra,https://repos.fyralabs.com/terra${ver}" terra-release
}

install_greeter() {
  log "$(say 'Noctalia greeter (optional)' 'Greeter de Noctalia (opcional)')"
  case "$FAMILY" in
    debian) pkg_install noctalia-greeter || warn "$(say 'could not install noctalia-greeter' 'no se pudo instalar noctalia-greeter')" ;;
    arch) pkg_install noctalia-greeter || warn "$(say 'not in the repositories; look in the AUR if you want it' 'no está en los repos; búscalo en AUR si lo quieres')" ;;
    fedora) pkg_install noctalia-greeter || warn "$(say 'not in the Fedora repositories' 'no está en los repos de Fedora')" ;;
    *) warn "$(say 'install the Noctalia greeter manually' 'instala el greeter de Noctalia manualmente')" ;;
  esac
  if [ "$LANG_MODE" = es ]; then
    cat <<'EOF'
  Para usarlo, configura greetd (/etc/greetd/config.toml):
    [default_session]
    command = "noctalia-greeter-session"
    user = "greeter"
  Luego: sudo systemctl enable --now greetd
EOF
  else
    cat <<'EOF'
  To use it, configure greetd (/etc/greetd/config.toml):
    [default_session]
    command = "noctalia-greeter-session"
    user = "greeter"
  Then: sudo systemctl enable --now greetd
EOF
  fi
}

setup_suse_repo() {
  [ "$NO_SYSTEM" -eq 1 ] && return 0
  [ "$SUSE_REPO_READY" -eq 1 ] && return 0
  local flavour="openSUSE_Tumbleweed"
  case "${VERSION_ID:-}" in
    16.*) flavour="openSUSE_Leap_16.0" ;;
  esac
  case "${PRETTY_NAME:-}" in
    *Slowroll*) flavour="openSUSE_Slowroll" ;;
  esac
  as_root zypper addrepo --refresh --name "Noctalia" \
    "https://download.opensuse.org/repositories/home:neifua:Noctalia/${flavour}/home:neifua:Noctalia.repo" || true
  as_root zypper --non-interactive refresh
  SUSE_REPO_READY=1
}

install_build_deps_noctalia() {
  case "$FAMILY" in
    arch)
      pkg_install_optional meson gcc just wayland wayland-protocols libglvnd freetype2 \
        fontconfig cairo pango harfbuzz libxkbcommon glib2 libsecret libsodium sdbus-cpp \
        libpipewire wireplumber polkit pam curl libwebp libjxl libsndfile librsvg \
        libqalculate libxml2 md4c tomlplusplus libical nlohmann-json stb jemalloc
      ;;
    debian)
      pkg_install_optional meson g++ just libwayland-dev wayland-protocols libglvnd-dev \
        libfreetype-dev libfontconfig-dev libcairo2-dev libpango1.0-dev libharfbuzz-dev \
        libxkbcommon-dev libglib2.0-dev libsecret-1-dev libsodium-dev libsdbus-c++-dev \
        libpipewire-0.3-dev wireplumber policykit-1 libpolkit-agent-1-dev libpam0g-dev \
        curl libwebp-dev libjxl-dev libsndfile1-dev librsvg2-dev libqalculate-dev \
        libxml2-dev libmd4c-dev libtomlplusplus-dev libical-dev nlohmann-json3-dev \
        libstb-dev libjemalloc-dev
      ;;
    fedora)
      pkg_install_optional meson ninja-build just gcc-c++ wayland-devel wayland-protocols-devel \
        libglvnd-devel freetype-devel fontconfig-devel cairo-devel pango-devel harfbuzz-devel \
        libxkbcommon-devel glib2-devel libsecret-devel libsodium-devel sdbus-cpp-devel \
        pipewire-devel wireplumber polkit-devel pam-devel curl libwebp-devel libjxl-devel \
        libsndfile-devel librsvg2-devel libqalculate-devel libxml2-devel md4c-devel \
        tomlplusplus-devel libical-devel nlohmann-json-devel stb-devel jemalloc-devel
      ;;
  esac
}

install_build_deps_umbriel() {
  case "$FAMILY" in
    arch)
      pkg_install_optional meson ninja just pkgconf wayland wayland-protocols wlroots0.20 \
        libxkbcommon libinput libei libdrm libdisplay-info cairo pango tomlplusplus \
        nlohmann-json libxcb xcb-util-wm lcms2 systemd-libs jemalloc xorg-xwayland
      ;;
    debian)
      pkg_install_optional meson ninja-build just g++ pkg-config libwayland-dev \
        wayland-protocols libwlroots-0.20-dev libxkbcommon-dev libinput-dev libei-dev \
        libdrm-dev libdisplay-info-dev libcairo2-dev libpango1.0-dev libtomlplusplus-dev \
        nlohmann-json3-dev libxcb1-dev libxcb-icccm4-dev libxcb-ewmh-dev libxcb-shape0-dev \
        libxcb-render0-dev libxcb-xfixes0-dev liblcms2-dev libsystemd-dev libjemalloc-dev xwayland
      ;;
    fedora)
      pkg_install_optional meson ninja-build just gcc-c++ pkgconf wayland-devel \
        wayland-protocols-devel wlroots0.20-devel libxkbcommon-devel libinput-devel \
        libei-devel libdrm-devel libdisplay-info-devel cairo-devel pango-devel \
        tomlplusplus-devel nlohmann-json-devel libxcb-devel xcb-util-wm-devel lcms2-devel \
        systemd-devel jemalloc-devel xorg-x11-server-Xwayland
      ;;
  esac
}

build_noctalia() {
  install_build_deps_noctalia
  ensure_cmd git
  ensure_cmd meson
  ensure_cmd ninja
  ensure_cmd just
  local dir="$CACHE_DIR/noctalia"
  local prefix="$SYSTEM_PREFIX"
  [ "$USER_ONLY" -eq 1 ] && prefix="$PREFIX"
  if [ -d "$dir/.git" ]; then
    run_in "$dir" git pull --ff-only
  else
    run git clone --depth 1 https://github.com/noctalia-dev/noctalia "$dir"
  fi
  run_in "$dir" just configure release "$prefix"
  run_in "$dir" just build release
  if [ "$USER_ONLY" -eq 1 ]; then
    run_in "$dir" just install release
  else
    as_root sh -c "cd '$dir' && just install release"
  fi
}

build_umbriel() {
  install_build_deps_umbriel
  ensure_cmd git
  ensure_cmd meson
  ensure_cmd ninja
  ensure_cmd just
  local dir="$CACHE_DIR/umbriel"
  local prefix="$SYSTEM_PREFIX"
  [ "$USER_ONLY" -eq 1 ] && prefix="$PREFIX"
  if [ -d "$dir/.git" ]; then
    run_in "$dir" git pull --ff-only
  else
    run git clone --depth 1 https://github.com/noctalia-dev/umbriel "$dir"
  fi
  run_in "$dir" just configure release "$prefix"
  run_in "$dir" just release
  if [ "$USER_ONLY" -eq 1 ]; then
    run_in "$dir" meson install -C build-release --no-rebuild
    for unit in umbriel.service umbriel-session.target umbriel-shutdown.target; do
      run systemctl --user link "$PREFIX/lib/systemd/user/$unit" || true
    done
    run systemctl --user daemon-reload || true
  else
    as_root meson install -C "$dir/build-release" --no-rebuild
  fi
}

install_noctalia() {
  log "Noctalia"
  if [ "$FROM_SOURCE" -eq 1 ]; then build_noctalia; return; fi
  case "$FAMILY" in
    arch)
      pkg_install noctalia || build_noctalia
      ;;
    fedora)
      pkg_install noctalia || {
        warn "$(say 'noctalia is not in the Fedora repositories; building from source' 'noctalia no está en los repos de Fedora; compilando')"
        build_noctalia
      }
      ;;
    debian)
      if setup_debian_repo; then
        pkg_install noctalia || build_noctalia
      else
        build_noctalia
      fi
      ;;
    suse)
      setup_suse_repo
      pkg_install noctalia || build_noctalia
      ;;
    *)
      build_noctalia
      ;;
  esac
}

install_umbriel() {
  log "Umbriel"
  if [ "$FROM_SOURCE" -eq 1 ]; then build_umbriel; return; fi
  case "$FAMILY" in
    arch)
      if need_cmd paru; then
        run paru -S --needed --noconfirm umbriel-git
      elif need_cmd yay; then
        run yay -S --needed --noconfirm umbriel-git
      else
        warn "$(say 'no paru/yay found; building Umbriel from source' 'no hay paru/yay; compilando Umbriel desde fuente')"
        build_umbriel
      fi
      ;;
    debian)
      if setup_debian_repo; then
        if [ "$VERSION_CODENAME" = trixie ]; then
          if ! as_root env DEBIAN_FRONTEND=noninteractive apt-get install -y -t trixie-backports umbriel; then
            add_debian_backports
            as_root env DEBIAN_FRONTEND=noninteractive apt-get install -y -t trixie-backports umbriel || build_umbriel
          fi
        else
          pkg_install umbriel || build_umbriel
        fi
        pkg_install_optional xdg-desktop-portal-umbriel
        if ! need_cmd umbriel; then build_umbriel; fi
      else
        build_umbriel
      fi
      ;;
    fedora)
      if ! pkg_install umbriel-nightly; then
        if setup_terra_repo; then
          pkg_install umbriel-nightly || {
            warn "$(say 'umbriel-nightly is still unavailable; building from source' 'umbriel-nightly sigue sin estar disponible; compilando desde fuente')"
            build_umbriel
          }
        else
          build_umbriel
        fi
      fi
      ;;
    suse)
      setup_suse_repo
      pkg_install umbriel-git || build_umbriel
      ;;
    *)
      build_umbriel
      ;;
  esac
}

# ---------------------------------------------------------------------------
# Configuración
# ---------------------------------------------------------------------------

ensure_cmd() {
  need_cmd "$1" || die "$(say "missing '$1'; install the dependencies or run without --no-system" "falta '$1'; instala las dependencias o ejecuta sin --no-system")"
}

NOCT_BIN="$PREFIX/bin/noct"

ensure_umbriel_config() {
  local cfg="$HOME/.config/umbriel/config.toml"
  [ -f "$cfg" ] && return 0
  local example=""
  for candidate in \
    "$REPO_DIR/iso/profile/airootfs/etc/skel/.config/umbriel/config.toml" \
    /usr/share/umbriel/config.toml /usr/local/share/umbriel/config.toml \
    "$PREFIX/share/umbriel/config.toml"; do
    if [ -f "$candidate" ]; then example="$candidate"; break; fi
  done
  [ -n "$example" ] || { warn "$(say 'Umbriel did not leave an example config.toml; it will be created when the session starts' 'Umbriel no dejó config.toml de ejemplo; se creará al iniciar la sesión')"; return 0; }
  run mkdir -p "$HOME/.config/umbriel"
  run cp "$example" "$cfg"
  # El ejemplo trae autostart comentado; Noctalia debe arrancar con la sesión.
  if [ "$DRY_RUN" -eq 0 ]; then
    if need_cmd python3; then
      python3 - "$cfg" <<'PY'
import re, sys
path = sys.argv[1]
text = open(path).read()
if re.search(r'(?m)^\s*autostart\s*=', text):
    text = re.sub(r'(?m)^\s*#?\s*autostart\s*=\s*\[[^\]]*\]', 'autostart = ["noctalia"]', text, count=1)
else:
    text = text.replace('[general]\n', '[general]\nautostart = ["noctalia"]\n', 1)
open(path, 'w').write(text)
PY
    else
      warn "$(say 'python3 is missing; add autostart = ["noctalia"] to [general] in %s' 'python3 no está; añade autostart = ["noctalia"] a [general] en %s' "$cfg")"
    fi
    # Rakia abre Brave con Mod+B; si no está, se usa el primer navegador disponible.
    if ! need_cmd brave; then
      local browser
      for browser in firefox chromium google-chrome-stable; do
        if need_cmd "$browser"; then
          sed -i "s|spawn brave|spawn $browser|g" "$cfg"
          break
        fi
      done
    fi
  fi
}

RAKIA_SHARE="$PREFIX/share/rakia"

install_rakia_assets() {
  local src="$REPO_DIR/iso/profile/airootfs/usr/share/rakia"
  [ -d "$src" ] || return 0
  run mkdir -p "$RAKIA_SHARE"
  run cp -a "$src/." "$RAKIA_SHARE/"
}

ensure_noctalia_config() {
  local cfg="$HOME/.config/noctalia/config.toml"
  local src="$REPO_DIR/iso/profile/airootfs/etc/skel/.config/noctalia/config.toml"
  [ -f "$cfg" ] && return 0
  [ -f "$src" ] || return 0
  [ "$DRY_RUN" -eq 1 ] && { log "$(say 'Would install the Rakia Noctalia config' 'Instalaría la configuración de Noctalia de Rakia')"; return 0; }
  mkdir -p "$HOME/.config/noctalia"
  sed "s|/usr/share/rakia|$RAKIA_SHARE|g" "$src" >"$cfg"
}

install_theme_watcher() {
  local dir="$HOME/.config/systemd/user"
  [ "$DRY_RUN" -eq 1 ] && { log "$(say 'Would write the noct-fix-umbriel-theme watcher' 'Escribiría el watcher noct-fix-umbriel-theme')"; return 0; }
  mkdir -p "$dir"
  cat > "$dir/noct-fix-umbriel-theme.service" <<EOF
[Unit]
Description=Corrige claves del tema de Umbriel tras Noctalia

[Service]
Type=oneshot
ExecStart=$NOCT_BIN doctor --fix
EOF
  cat > "$dir/noct-fix-umbriel-theme.path" <<'EOF'
[Unit]
Description=Vigila la plantilla de Umbriel generada por Noctalia

[Path]
PathChanged=%h/.config/umbriel/noctalia.toml
Unit=noct-fix-umbriel-theme.service

[Install]
WantedBy=default.target
EOF
  systemctl --user daemon-reload >/dev/null 2>&1 || true
  systemctl --user enable --now noct-fix-umbriel-theme.path >/dev/null 2>&1 || true
}

install_compat_layer() {
  local dir="$HOME/.config/systemd/user"
  [ "$DRY_RUN" -eq 1 ] && { log "$(say 'Would write the Rakia compatibility service' 'Escribiría el servicio de compatibilidad de Rakia')"; return 0; }
  mkdir -p "$dir"
  mkdir -p "$dir/umbriel.service.d"
  cat > "$dir/umbriel.service.d/rakia-software.conf" <<'EOF'
[Service]
Environment=WLR_RENDERER_ALLOW_SOFTWARE=1
EOF
  cat > "$dir/rakia-compat.service" <<EOF
[Unit]
Description=Rakia: compatibilidad con Umbriel y Noctalia

[Service]
Type=oneshot
ExecStart=-$NOCT_BIN compat --apply --quiet
Nice=10
IOSchedulingClass=idle
EOF
  cat > "$dir/rakia-compat.timer" <<'EOF'
[Unit]
Description=Rakia: comprobación periódica de compatibilidad

[Timer]
OnStartupSec=45s
OnUnitActiveSec=20min
AccuracySec=1min
Persistent=true

[Install]
WantedBy=timers.target
EOF
  local umbriel_bin noctalia_bin
  umbriel_bin="$(command -v umbriel 2>/dev/null || true)"
  noctalia_bin="$(command -v noctalia 2>/dev/null || true)"
  if [ -n "$umbriel_bin" ] || [ -n "$noctalia_bin" ]; then
    {
      printf '%s\n' \
        '[Unit]' \
        'Description=Rakia: vigila actualizaciones de Umbriel y Noctalia' \
        '' \
        '[Path]'
      if [ -n "$umbriel_bin" ]; then printf 'PathChanged=%s\n' "$umbriel_bin"; fi
      if [ -n "$noctalia_bin" ]; then printf 'PathChanged=%s\n' "$noctalia_bin"; fi
      printf '%s\n' \
        'Unit=rakia-compat.service' \
        '' \
        '[Install]' \
        'WantedBy=default.target'
    } > "$dir/rakia-compat.path"
    systemctl --user enable --now rakia-compat.path >/dev/null 2>&1 || true
  fi
  systemctl --user daemon-reload >/dev/null 2>&1 || true
  systemctl --user enable rakia-compat.timer >/dev/null 2>&1 || true
  systemctl --user start rakia-compat.timer >/dev/null 2>&1 || true
  "$NOCT_BIN" compat --apply --quiet >/dev/null 2>&1 || true
}

write_wallpaper_config() {
  local mode="$1" path="${2:-}"
  [ "$DRY_RUN" -eq 1 ] && { log "$(say 'Would write wallpaper.toml in %s mode' 'Escribiría wallpaper.toml en modo %s' "$mode")"; return 0; }
  mkdir -p "$HOME/.config/noct"
  if [ "$mode" = fixed ] && [ -n "$path" ]; then
    printf 'mode = "fixed"\npath = "%s"\nupdated_at = 0\n' "$path" > "$HOME/.config/noct/wallpaper.toml"
  else
    printf 'mode = "bing-daily"\nupdated_at = 0\n' > "$HOME/.config/noct/wallpaper.toml"
  fi
}

configure_wallpaper() {
  local noctalia_running=0
  if noctalia msg wallpaper-get >/dev/null 2>&1; then noctalia_running=1; fi

  case "$WALLPAPER_MODE" in
    bing-daily)
      run "$NOCT_BIN" wallpaper bing-daily || warn "$(say 'could not apply Bing now; the timer will do it at login' 'no se pudo aplicar Bing ahora; el timer lo hará al iniciar sesión')"
      ;;
    fixed)
      if [ -n "$WALLPAPER_PATH" ]; then
        run "$NOCT_BIN" wallpaper fixed "$WALLPAPER_PATH"
      elif [ "$noctalia_running" -eq 1 ]; then
        run "$NOCT_BIN" wallpaper fixed
      else
        write_wallpaper_config fixed
        warn "$(say 'Noctalia is not running; fixed mode will be applied at login' 'Noctalia no está corriendo; el modo fijo se aplicará al iniciar sesión')"
      fi
      ;;
    "")
      if [ -f "$HOME/.config/noct/wallpaper.toml" ]; then
        log "$(say 'Wallpaper: keeping the saved mode' 'Wallpaper: se conserva el modo guardado')"
      elif [ "$noctalia_running" -eq 1 ]; then
        run "$NOCT_BIN" wallpaper fixed || true
      else
        write_wallpaper_config fixed
        log "$(say 'Wallpaper: fixed mode by default (applied at login)' 'Wallpaper: modo fijo por defecto (se aplicará al iniciar sesión)')"
      fi
      ;;
  esac
}

migrate_umbriel_syntax() {
  [ "$MIGRATE" -eq 1 ] || return 0
  need_cmd umbriel || return 0
  # Solo migramos si el Umbriel instalado ya usa la sintaxis con espacios.
  if ! umbriel msg --help 2>/dev/null | grep -q 'workspace-switch <workspace>'; then
    log "$(say 'Old Umbriel detected; keeping the current keybind syntax' 'Umbriel antiguo detectado; se conserva la sintaxis de atajos')"
    return 0
  fi
  [ "$DRY_RUN" -eq 1 ] && { log "$(say 'Would migrate the Umbriel keybind syntax' 'Migraría la sintaxis de atajos de Umbriel')"; return 0; }
  if ! need_cmd python3; then
    warn "$(say 'python3 is missing; skipping keybind migration' 'python3 no está; se omite la migración de atajos')"
    return 0
  fi
  LANG_MODE="$LANG_MODE" python3 - "$HOME/.config/umbriel" <<'PY'
import os, re, sys, pathlib, shutil, datetime
base = pathlib.Path(sys.argv[1])
files = [base/'config.toml', base/'noct/keybinds.toml', base/'noct/bridge.toml']

def action(value):
    if value.startswith('spawn:'):
        return 'spawn ' + value[6:]
    m = re.match(r'^([a-z][a-z0-9-]*):(.+)$', value)
    if m:
        return m.group(1) + ' ' + m.group(2).replace(':', ' ')
    return value

def line(raw):
    if re.match(r'^\s*follows_mouse\s*=\s*true\s*$', raw):
        return re.sub(r'^(\s*follows_mouse\s*=\s*)true\s*$', r'\1"sloppy"', raw)
    m = re.match(r'^(\s*"[^"]+"\s*=\s*)"([^"]*)"(\s*(?:#.*)?)$', raw)
    if m:
        return m.group(1) + '"' + action(m.group(2)) + '"' + m.group(3)
    m = re.match(r'^(\s*"[^"]+"\s*=\s*\{.*?\baction\s*=\s*")([^"]*)(".*)$', raw)
    if m:
        return m.group(1) + action(m.group(2)) + m.group(3)
    return raw

def commands(text):
    # También los eventos y scripts: `umbriel msg output-enable:eDP-1`.
    return re.sub(
        r'(umbriel\s+msg\s+)([a-z][a-z0-9-]*):([^\s"\']+)',
        lambda m: m.group(1) + m.group(2) + ' ' + m.group(3).replace(':', ' '),
        text,
    )

stamp = datetime.datetime.now().strftime('%Y%m%d-%H%M%S')
for path in files:
    if not path.exists():
        continue
    original = path.read_text()
    updated = ''.join(line(l.rstrip('\n')) + ('\n' if l.endswith('\n') else '') for l in original.splitlines(keepends=True))
    updated = commands(updated)
    if updated != original:
        shutil.copy2(path, path.with_name(path.name + f'.pre-b56-{stamp}.bak'))
        path.write_text(updated)
        print((('migrado: ' if os.environ.get('LANG_MODE') == 'es' else 'migrated: ')) + str(path))
PY
}

install_noct() {
  log "Noct"
  ensure_rust
  run_in "$REPO_DIR" cargo build --release -p noct-cli
  run mkdir -p "$PREFIX/bin"
  run install -Dm755 "$REPO_DIR/target/release/noct" "$NOCT_BIN"
  case ":$PATH:" in
    *":$PREFIX/bin:"*) ;;
    *)
      warn "$(say '%s/bin is not in PATH; add it to your login profile' '%s/bin no está en PATH; añádelo en tu perfil de login' "$PREFIX")"
      if [ "$DRY_RUN" -eq 0 ] && [ -w "$HOME/.profile" ] && ! grep -q 'noct installer PATH' "$HOME/.profile" 2>/dev/null; then
        printf '\n# noct installer PATH\nexport PATH="%s/bin:$PATH"\n' "$PREFIX" >> "$HOME/.profile"
        log "$(say 'PATH added to ~/.profile' 'PATH añadido a ~/.profile')"
      fi
      ;;
  esac
}

has_bind() {
  local chord="$1"
  grep -q "\"$chord\"[[:space:]]*=" "$HOME/.config/umbriel/config.toml" \
    "$HOME/.config/umbriel/noct/keybinds.toml" 2>/dev/null
}

configure_screenshot_keys() {
  local chord action pair
  for pair in \
    "Print|spawn noct msg screenshot-region" \
    "Shift+Print|spawn noct msg screenshot-window" \
    "Ctrl+Print|spawn noct msg screenshot-fullscreen" \
    "Alt+Print|spawn noct msg screenshot-annotate"; do
    chord="${pair%%|*}"
    action="${pair#*|}"
    if has_bind "$chord"; then
      log "$(say 'Shortcut %s already exists; keeping it' 'Atajo %s ya existe; se conserva' "$chord")"
    else
      run "$NOCT_BIN" keys set "$chord" "$action" || warn "$(say 'could not add %s' 'no se pudo añadir %s' "$chord")"
    fi
  done
}

install_removable_drives() {
  [ "$WITH_UDISKS" -eq 1 ] || return 0
  log "$(say 'Removable drives (udisks2 + udiskie)' 'Discos extraíbles (udisks2 + udiskie)')"

  if [ "$USER_ONLY" -eq 0 ] && [ "$NO_SYSTEM" -eq 0 ]; then
    pkg_install_optional udisks2 udiskie
  fi
  if ! need_cmd udiskie; then
    warn "$(say 'udiskie is not available; install it manually (pipx install udiskie) and run: udiskie --automount --notify' 'udiskie no está disponible; instálalo a mano (pipx install udiskie) y ejecuta: udiskie --automount --notify')"
    return 0
  fi

  local cfg="$HOME/.config/udiskie/config.yml"
  if [ ! -f "$cfg" ]; then
    if [ "$DRY_RUN" -eq 1 ]; then
      log "$(say 'Would write %s' 'Escribiría %s' "$cfg")"
    else
      mkdir -p "$(dirname "$cfg")"
      cat > "$cfg" <<'EOF'
program_options:
  automount: true
  notify: true
  tray: false
EOF
    fi
  fi

  if [ "$USER_ONLY" -eq 0 ] && [ "$NO_SYSTEM" -eq 0 ]; then
    as_root systemctl enable --now udisks2 >/dev/null 2>&1 || \
      warn "$(say 'could not enable udisks2' 'no se pudo habilitar udisks2')"
  fi

  if [ "$USER_ONLY" -eq 0 ] && [ "$NO_SYSTEM" -eq 0 ]; then
    if [ "$DRY_RUN" -eq 1 ]; then
      log "$(say 'Would install the udisks polkit rule' 'Instalaría la regla polkit de udisks')"
    else
      as_root install -d -m 0755 /etc/polkit-1/rules.d
      printf '%s\n' \
        '// Noct — mount/eject removable drives for wheel (local, active session).' \
        'polkit.addRule(function(action, subject) {' \
        '    var allowed = [' \
        '        "org.freedesktop.udisks2.filesystem-mount",' \
        '        "org.freedesktop.udisks2.eject-media",' \
        '        "org.freedesktop.udisks2.power-off-drive"' \
        '    ];' \
        '    if (allowed.indexOf(action.id) >= 0 &&' \
        '        subject.isInGroup("wheel") &&' \
        '        subject.local &&' \
        '        subject.active) {' \
        '        return polkit.Result.YES;' \
        '    }' \
        '});' | as_root tee /etc/polkit-1/rules.d/49-noct-udisks.rules >/dev/null
    fi
  else
    log "$(say 'Skipping the system polkit rule (--user or --no-system)' 'Se omite la regla polkit del sistema (--user o --no-system)')"
  fi

  local unit="$HOME/.config/systemd/user/noct-udiskie.service"
  if [ "$DRY_RUN" -eq 1 ]; then
    log "$(say 'Would enable the noct-udiskie user service' 'Habilitaría el servicio de usuario noct-udiskie')"
    return 0
  fi
  mkdir -p "$(dirname "$unit")"
  cat > "$unit" <<EOF
[Unit]
Description=Noct removable drive automount

[Service]
ExecStart=$(command -v udiskie) --automount --notify
Restart=on-failure

[Install]
WantedBy=default.target
EOF
  systemctl --user daemon-reload >/dev/null 2>&1 || true
  systemctl --user enable --now noct-udiskie.service >/dev/null 2>&1 || \
    warn "$(say 'could not start the noct-udiskie user service' 'no se pudo iniciar el servicio de usuario noct-udiskie')"
}

configure_greeter_logo() {
  [ "$HIDE_GREETER_LOGO" -eq 1 ] || return 0
  need_cmd noctalia-greeter || return 0
  if [ "$USER_ONLY" -eq 1 ] || [ "$NO_SYSTEM" -eq 1 ]; then
    log "$(say 'Skipping the greeter logo (--user or --no-system)' 'Se omite el logo del greeter (--user o --no-system)')"
    return 0
  fi
  local home cfg tmp
  home="$(getent passwd greeter 2>/dev/null | cut -d: -f6)"
  [ -n "$home" ] && [ -d "$home" ] || return 0
  cfg="$home/greeter.toml"
  if [ "$DRY_RUN" -eq 1 ]; then
    log "$(say 'Would hide the Noctalia greeter logo' 'Ocultaría el logo del greeter de Noctalia')"
    return 0
  fi

  tmp="$(mktemp)"
  if [ -f "$cfg" ]; then
    as_root cp "$cfg" "$tmp"
    as_root chown "$(id -u):$(id -g)" "$tmp"
  else
    printf '[appearance]\nhide_logo = true\n' > "$tmp"
  fi

  if need_cmd python3; then
    python3 - "$tmp" <<'PY'
import re
import sys

path = sys.argv[1]
text = open(path).read()
if re.search(r'(?m)^\s*hide_logo\s*=', text):
    text = re.sub(r'(?m)^\s*hide_logo\s*=.*$', 'hide_logo = true', text, count=1)
elif re.search(r'(?m)^\[appearance\]\s*$', text):
    text = re.sub(r'(?m)^(\[appearance\]\s*)$', r'\1\nhide_logo = true', text, count=1)
else:
    if text and not text.endswith('\n'):
        text += '\n'
    text += '\n[appearance]\nhide_logo = true\n'
open(path, 'w').write(text)
PY
  else
    printf '\n[appearance]\nhide_logo = true\n' >> "$tmp"
  fi

  if as_root install -o greeter -g greeter -m 0644 "$tmp" "$cfg"; then
    log "$(say 'Noctalia greeter logo hidden' 'Logo del greeter de Noctalia oculto')"
  else
    warn "$(say 'could not update %s' 'no se pudo actualizar %s' "$cfg")"
  fi
  rm -f "$tmp"
}

configure_noct() {
  log "$(say 'Configuring Noct' 'Configuración de Noct')"
  install_rakia_assets
  ensure_umbriel_config
  ensure_noctalia_config
  run "$NOCT_BIN" doctor --fix || true
  if [ ! -f "$HOME/.config/umbriel/noct/effects.toml" ]; then
    run "$NOCT_BIN" effects apply mono || true
  fi
  run "$NOCT_BIN" effects widget || true
  configure_screenshot_keys
  install_theme_watcher
  migrate_umbriel_syntax
  configure_wallpaper
  install_compat_layer
}

# ---------------------------------------------------------------------------
# Ejecución
# ---------------------------------------------------------------------------

if { [ "$WITH_NOCTALIA" -eq 1 ] || [ "$WITH_UMBRIEL" -eq 1 ]; } && [ "$NO_SYSTEM" -eq 0 ]; then
  pkg_update || true
  pkg_install_optional git curl ca-certificates
  if ! need_cmd python3; then
    pkg_install_optional python3 python
  fi
fi

if [ "$WITH_NOCTALIA" -eq 1 ]; then
  install_noctalia
fi
if [ "$WITH_GREETER" -eq 1 ]; then
  install_greeter
fi
if [ "$WITH_UMBRIEL" -eq 1 ]; then
  install_umbriel
fi

install_noct
install_removable_drives
configure_greeter_logo
configure_noct

log "$(say 'Done.' 'Listo.')"
if [ "$DRY_RUN" -eq 0 ]; then
  echo
  if [ "$LANG_MODE" = es ]; then
    echo "Comprobaciones:"
    echo "  noct --version"
    echo "  umbriel --version"
    echo "  umbriel config validate"
    echo "  noct wallpaper status"
    echo
    echo "Cierra sesión y vuelve a entrar para usar el Umbriel nuevo."
    echo "Wallpaper:"
    echo "  noct wallpaper fixed [ruta]   # fijo, sin volver a Bing"
    echo "  noct wallpaper bing-daily     # Bing diario + greeter"
    echo
    echo "USB: conecta un dispositivo; udiskie lo montará en /run/media/$USER."
  else
    echo "Checks:"
    echo "  noct --version"
    echo "  umbriel --version"
    echo "  umbriel config validate"
    echo "  noct wallpaper status"
    echo
    echo "Log out and back in to use the new Umbriel."
    echo "Wallpaper:"
    echo "  noct wallpaper fixed [path]   # fixed, never reverts to Bing"
    echo "  noct wallpaper bing-daily     # daily Bing + greeter"
    echo
    echo "USB: plug a device; udiskie will mount it under /run/media/$USER."
  fi
fi
