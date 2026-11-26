#!/usr/bin/env bash
# Se ejecuta dentro del airootfs, después de pacstrap y antes de crear la ISO.
# Es el último punto para mutar el sistema de archivos del live.
set -euo pipefail

# Grupos base (los crea systemd-sysusers al instalar los paquetes).
# Se ejecuta de nuevo por si el orden de pacstrap dejó alguno fuera.
systemd-sysusers

# Usuario live. El passwd viene en airootfs para que mkarchiso copie /etc/skel;
# aquí solo se completa la pertenencia a los grupos de hardware.
if id -u nocturne >/dev/null 2>&1; then
  usermod -aG wheel,video,render,input,audio,storage,power,rfkill nocturne
else
  useradd -m -u 1000 -g 1000 -G wheel,video,render,input,audio,storage,power,rfkill -s /bin/bash nocturne
fi
chown -hR 1000:1000 /home/nocturne

# Locales del live.
sed -i \
  -e 's/^#[[:space:]]*en_US.UTF-8/en_US.UTF-8/' \
  -e 's/^#[[:space:]]*es_ES.UTF-8/es_ES.UTF-8/' \
  /etc/locale.gen
locale-gen

# La consola de rescate no pide contraseña: el acceso físico ya es root.
passwd -d root >/dev/null 2>&1 || true

# Identidad de la ISO.
printf 'Nocturne Linux \\r (\\l)\n' >/etc/issue

# machine-id vacío: systemd genera uno nuevo en cada arranque live.
: >/etc/machine-id

# Limpieza de .pacnew de los archivos de branding.
find /etc -maxdepth 2 -name '*.pacnew' -delete

# Permisos del sudoers live (también los fija profiledef.sh).
chmod 0440 /etc/sudoers.d/10-nocturne-live

# ananicy no tiene sentido en el live; en el sistema instalado sí.
ln -sf /dev/null /etc/systemd/system/ananicy-cpp.service

# Punto de montaje del llavero temporal. Se deja creado (vacío) para que
# pacman-init pueda montar el tmpfs al arrancar y para que mkarchiso pueda
# consultar la base de datos local al generar la lista de paquetes.
install -d -m 0755 /etc/pacman.d/gnupg
pacman-key --init >/dev/null 2>&1 || true
