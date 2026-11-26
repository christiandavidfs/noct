#!/usr/bin/env bash
# Construye la ISO de Noga Linux.
#
# Uso:
#   sudo ./iso/build.sh                 # edición slim (sin NVIDIA en el live)
#   sudo ./iso/build.sh --nvidia        # edición NVIDIA (driver en el live)
#   ./iso/build.sh --check              # valida perfil y paquetes, sin root
#
# Requiere construir en CachyOS (o con los repos de CachyOS configurados),
# archiso y syslinux instalados, y ~10 GB libres.
set -euo pipefail

REPO_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
ISO_DIR="$REPO_DIR/iso"
PROFILE="$ISO_DIR/profile"
BUILD_DIR="$REPO_DIR/build"

VARIANT="slim"
MODE="build"
CLEAN=0

usage() {
  cat <<'EOF'
Uso: build.sh [opciones]

  --slim        Edición sin NVIDIA en el live (por defecto)
  --nvidia      Edición con nvidia-open/nvidia-utils en el live
  --check       Valida el perfil y los paquetes sin construir
  --clean       Borra el directorio de trabajo antes de construir
  -h, --help    Muestra esta ayuda

Salida:
  build/iso-out/noga-<fecha>-x86_64.iso
  build/iso-out/noga-nvidia-<fecha>-x86_64.iso
EOF
}

while (($#)); do
  case "$1" in
    --slim) VARIANT=slim ;;
    --nvidia) VARIANT=nvidia ;;
    --check) MODE=check ;;
    --clean) CLEAN=1 ;;
    -h|--help) usage; exit 0 ;;
    *)
      printf 'Opción desconocida: %s\n\n' "$1" >&2
      usage >&2
      exit 2
      ;;
  esac
  shift
done

STAGE="$BUILD_DIR/iso-stage-$VARIANT"
WORK="$BUILD_DIR/iso-work-$VARIANT"
OUT="$BUILD_DIR/iso-out"

check_mirrors() {
  local mirror
  for mirror in /etc/pacman.d/mirrorlist /etc/pacman.d/cachyos-mirrorlist; do
    if [[ ! -s "$mirror" ]]; then
      printf 'Falta %s. Construye en CachyOS o instala cachyos-mirrorlist.\n' "$mirror" >&2
      return 1
    fi
  done
  if ! pacman -Q cachyos-keyring >/dev/null 2>&1; then
    printf 'Falta cachyos-keyring en el host: pacman no podrá verificar [cachyos].\n' >&2
    printf 'Instálalo con: sudo pacman -S cachyos-keyring\n' >&2
    return 1
  fi
}

check_packages() {
  local pkg missing=()
  while IFS= read -r pkg; do
    pkg="${pkg%%#*}"
    pkg="${pkg//[[:space:]]/}"
    [[ -z "$pkg" ]] && continue
    if ! pacman -Si "$pkg" >/dev/null 2>&1; then
      missing+=("$pkg")
    fi
  done < <(cat "$PROFILE/packages.x86_64" "$ISO_DIR/packages/nvidia-live.txt")

  if ((${#missing[@]})); then
    printf 'Paquetes no encontrados en los repos configurados:\n' >&2
    printf '  %s\n' "${missing[@]}" >&2
    return 1
  fi
}

if [[ "$MODE" == check ]]; then
  check_mirrors
  check_packages
  printf 'Perfil %s válido: mirrors y paquetes encontrados.\n' "$VARIANT"
  exit 0
fi

if ((EUID != 0)); then
  printf 'Ejecuta con sudo: sudo %s %s\n' "$0" "$*" >&2
  exit 1
fi

missing_deps=()
command -v mkarchiso >/dev/null 2>&1 || missing_deps+=(archiso)
command -v mksquashfs >/dev/null 2>&1 || missing_deps+=(squashfs-tools)
command -v xorriso >/dev/null 2>&1 || missing_deps+=(libisoburn)
command -v mkfs.vfat >/dev/null 2>&1 || missing_deps+=(dosfstools)
command -v mformat >/dev/null 2>&1 || missing_deps+=(mtools)
[[ -e /usr/lib/syslinux/bios/ldlinux.c32 ]] || missing_deps+=(syslinux)

if ((${#missing_deps[@]})); then
  printf 'Faltan dependencias de build:\n  %s\n' "${missing_deps[*]}" >&2
  printf 'Instálalas con: sudo pacman -S --needed %s\n' "${missing_deps[*]}" >&2
  exit 1
fi

check_mirrors
check_packages

printf 'Preparando perfil %s...\n' "$VARIANT"
rm -rf -- "$STAGE"
mkdir -p -- "$STAGE"
cp -a -- "$PROFILE/." "$STAGE/"

if [[ "$VARIANT" == nvidia ]]; then
  sed -i 's|^iso_name=.*|iso_name="noga-nvidia"|' "$STAGE/profiledef.sh"
  sed -i "s|^iso_label=.*|iso_label=\"NOGA_NV_$(date +%Y%m)\"|" "$STAGE/profiledef.sh"
  sed -i 's|^kernel_params_x86_64=.*|kernel_params_x86_64="nvidia_drm.modeset=1 nvidia_drm.fbdev=1"|' "$STAGE/profiledef.sh"
  sed '/^[[:blank:]]*#/d;/^[[:blank:]]*$/d' "$ISO_DIR/packages/nvidia-live.txt" >>"$STAGE/packages.x86_64"
fi

if [[ -x "$REPO_DIR/target/release/noct" ]]; then
  install -Dm755 -- "$REPO_DIR/target/release/noct" "$STAGE/airootfs/usr/local/bin/noct"
  printf 'Incluido noct en /usr/local/bin del live.\n'
else
  printf 'Aviso: no hay target/release/noct; el live no incluirá el CLI.\n'
fi

if ((CLEAN)); then
  rm -rf -- "$WORK"
fi
mkdir -p -- "$WORK" "$OUT"

printf 'Construyendo ISO %s (esto descarga paquetes y tarda)...\n' "$VARIANT"
mkarchiso -v -w "$WORK" -o "$OUT" "$STAGE" 2>&1 | tee "$BUILD_DIR/iso-$VARIANT.log"

iso_file="$(find "$OUT" -maxdepth 1 -type f -name 'noga*.iso' -printf '%T@ %p\n' \
  | sort -nr | head -n1 | cut -d' ' -f2-)"

if [[ -z "$iso_file" || ! -f "$iso_file" ]]; then
  printf 'No se encontró la ISO generada en %s\n' "$OUT" >&2
  exit 1
fi

sha256sum -- "$iso_file" >"$iso_file.sha256"
ln -sfn -- "$(basename -- "$iso_file")" "$OUT/noga-$VARIANT.iso"

printf '\nISO lista: %s\n' "$iso_file"
printf 'SHA256:    %s\n' "$iso_file.sha256"
printf 'Enlace:    %s\n' "$OUT/noga-$VARIANT.iso"
