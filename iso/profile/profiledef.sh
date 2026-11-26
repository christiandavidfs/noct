#!/usr/bin/env bash
# shellcheck disable=SC2034
# Nocturne Linux live ISO profile.

iso_name="nocturne"
iso_label="NOCTURNE_$(date --date="@${SOURCE_DATE_EPOCH:-$(date +%s)}" +%Y%m)"
iso_publisher="Nocturne Linux <https://github.com/christiandavidfs/noct>"
iso_application="Nocturne Linux Live"
iso_version="$(date --date="@${SOURCE_DATE_EPOCH:-$(date +%s)}" +%Y.%m.%d)"
install_dir="nocturne"

buildmodes=('iso')
bootmodes=('bios.syslinux' 'uefi.systemd-boot')
arch="x86_64"
pacman_conf="pacman.conf"
kernel_params_x86_64=""

airootfs_image_type="squashfs"
airootfs_image_tool_options=('-comp' 'zstd' '-Xcompression-level' '19' '-b' '1M')
bootstrap_tarball_compression=('zstd' '-c' '-T0' '--auto-threads=logical' '--long' '-19')

file_permissions=(
  ["/etc/gshadow"]="0:0:400"
  ["/etc/shadow"]="0:0:400"
  ["/etc/sudoers.d/10-nocturne-live"]="0:0:440"
  ["/usr/local/bin/noct"]="0:0:755"
  ["/usr/local/bin/nocturne-install"]="0:0:755"
  ["/root"]="0:0:750"
  ["/root/customize_airootfs.sh"]="0:0:755"
)
