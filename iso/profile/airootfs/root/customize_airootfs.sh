#!/usr/bin/env bash
# Se ejecuta dentro del airootfs, después de pacstrap y antes de crear la ISO.
# Es el último punto para mutar el sistema de archivos del live.
set -euo pipefail

# Grupos base (los crea systemd-sysusers al instalar los paquetes).
# Se ejecuta de nuevo por si el orden de pacstrap dejó alguno fuera.
systemd-sysusers

# Usuario live. El passwd viene en airootfs para que mkarchiso copie /etc/skel;
# aquí solo se completa la pertenencia a los grupos de hardware.
if id -u noga >/dev/null 2>&1; then
  usermod -aG wheel,video,render,input,audio,storage,power,rfkill noga
else
  useradd -m -u 1000 -g 1000 -G wheel,video,render,input,audio,storage,power,rfkill -s /bin/bash noga
fi
chown -hR 1000:1000 /home/noga

# Locales del live.
sed -i \
  -e 's/^#[[:space:]]*en_US.UTF-8/en_US.UTF-8/' \
  -e 's/^#[[:space:]]*es_ES.UTF-8/es_ES.UTF-8/' \
  /etc/locale.gen
locale-gen

# La consola de rescate no pide contraseña: el acceso físico ya es root.
passwd -d root >/dev/null 2>&1 || true

# Identidad de la ISO.
printf 'Noga Linux \\r (\\l)\n' >/etc/issue

# machine-id vacío: systemd genera uno nuevo en cada arranque live.
: >/etc/machine-id

# Limpieza de .pacnew de los archivos de branding.
find /etc -maxdepth 2 -name '*.pacnew' -delete

# Permisos del sudoers live (también los fija profiledef.sh).
chmod 0440 /etc/sudoers.d/10-noga-live

# ananicy no tiene sentido en el live; en el sistema instalado sí.
ln -sf /dev/null /etc/systemd/system/ananicy-cpp.service

# Fondo por defecto de Noga (sin el búho de Noctalia).
if [ -f /usr/share/rakia/wallpapers/noctalia-wallpaper.png ]; then
  install -Dm644 /usr/share/rakia/wallpapers/noctalia-wallpaper.png \
    /usr/share/noctalia/assets/noctalia-wallpaper.png
fi

# Calamares: nuestra secuencia y el qml que instala el paquete.
if [ -f /etc/calamares/settings-noct.conf ]; then
  install -Dm644 /etc/calamares/settings-noct.conf /etc/calamares/settings.conf
fi
ln -sfn /usr/share/calamares/qml /etc/calamares/qml

# Nombre de Noga en el menú de GRUB del sistema instalado.
if [ -f /etc/default/grub ]; then
  sed -i 's/^GRUB_DISTRIBUTOR=.*/GRUB_DISTRIBUTOR="Noga"/' /etc/default/grub
fi

# Punto de montaje del llavero temporal. Se deja creado (vacío) para que
# pacman-init pueda montar el tmpfs al arrancar.
install -d -m 0755 /etc/pacman.d/gnupg
find /etc/pacman.d/gnupg -mindepth 1 -delete

# Arch trae los servidores de /etc/pacman.d/mirrorlist comentados; se activan
# unos cuantos para que pacman pueda sincronizar core/extra/multilib.
if [ -f /etc/pacman.d/mirrorlist ] && ! grep -q '^Server' /etc/pacman.d/mirrorlist; then
  awk 'BEGIN{n=0} /^#Server/ && n<6 {sub(/^#/,""); n++} {print}' /etc/pacman.d/mirrorlist > /etc/pacman.d/mirrorlist.tmp
  mv /etc/pacman.d/mirrorlist.tmp /etc/pacman.d/mirrorlist
fi

# Widget de efectos de Noct, instalado y habilitado en el perfil por defecto y
# en la sesión live. Noctalia no está corriendo, pero deja el estado escrito.
install_noct_widget() {
  local home="$1"
  [ -d "$home" ] || return 0
  [ -x /usr/local/bin/noct ] || return 0
  HOME="$home" \
    XDG_CONFIG_HOME="$home/.config" \
    XDG_DATA_HOME="$home/.local/share" \
    timeout 20 /usr/local/bin/noct effects widget >/dev/null 2>&1 || true
}
install_noct_widget /etc/skel
install_noct_widget /home/noga
chown -hR 1000:1000 /home/noga

# La ISO se mantiene por debajo del límite de 2 GiB de GitHub Releases:
# se quitan páginas de manual, documentación y traducciones que no sean es/en.
# Los paquetes siguen instalados; `pacman -S` puede restaurarlas.
find /usr/share/doc -mindepth 1 -maxdepth 1 ! -name noga -exec rm -rf {} + 2>/dev/null || true
rm -rf /usr/share/man/* /usr/share/info/*
find /usr/share/locale -mindepth 1 -maxdepth 1 -type d \
  ! -name 'es*' ! -name 'en*' -exec rm -rf {} + 2>/dev/null || true

# Contrato del escritorio: avisa si la configuración por defecto deja de ser
# válida con la versión empaquetada de Umbriel o si Noctalia renombró un verbo.
if command -v umbriel >/dev/null 2>&1; then
  umbriel config validate -c /etc/skel/.config/umbriel/config.toml ||
    printf 'Aviso: la configuración por defecto de Umbriel no valida con la versión empaquetada.\n' >&2
fi
if command -v noctalia >/dev/null 2>&1; then
  noctalia msg --help 2>/dev/null | awk '/^  [a-z0-9-]+/{print $1}' | sort -u >/tmp/noctalia-verbs.txt || true
  grep -ho 'spawn noct msg [a-z0-9-]*' /etc/skel/.config/umbriel/config.toml 2>/dev/null |
    awk '{print $4}' | sort -u |
    while read -r verb; do
      [ -n "$verb" ] || continue
      grep -qx -- "$verb" /tmp/noctalia-verbs.txt ||
        printf 'Aviso: Noctalia no anuncia el verbo %s (noct msg lo gestiona en runtime).\n' "$verb" >&2
    done
  rm -f /tmp/noctalia-verbs.txt
fi
