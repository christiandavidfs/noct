#!/usr/bin/env bash
# shellcheck disable=SC2034
# Noga Linux live ISO profile.

iso_name="noga"
iso_label="NOGA_$(date --date="@${SOURCE_DATE_EPOCH:-$(date +%s)}" +%Y%m)"
iso_publisher="Noga Linux <https://github.com/christiandavidfs/noct>"
iso_application="Noga Linux Live"
iso_version="$(date --date="@${SOURCE_DATE_EPOCH:-$(date +%s)}" +%Y.%m.%d)"
install_dir="noga"

buildmodes=('iso')
bootmodes=('bios.syslinux' 'uefi.systemd-boot')
arch="x86_64"
pacman_conf="pacman.conf"
kernel_params_x86_64=""

airootfs_image_type="squashfs"
airootfs_image_tool_options=('-comp' 'zstd' '-Xcompression-level' '22' '-b' '1M')
bootstrap_tarball_compression=('zstd' '-c' '-T0' '--auto-threads=logical' '--long' '-19')

file_permissions=(
  ["/etc/gshadow"]="0:0:400"
  ["/etc/shadow"]="0:0:400"
  ["/etc/sudoers.d/10-noga-live"]="0:0:440"
  ["/usr/local/bin/noct"]="0:0:755"
  ["/usr/local/bin/noga-install"]="0:0:755"
  ["/usr/local/bin/rakia-compat"]="0:0:755"
  ["/usr/local/bin/noga-unpack-source"]="0:0:755"
  ["/usr/local/bin/noga-vmlinuz"]="0:0:755"
  ["/usr/local/bin/noga-bootloader"]="0:0:755"
  ["/usr/local/bin/noga-greetd"]="0:0:755"
  ["/usr/local/bin/noga-snapshots"]="0:0:755"
  ["/usr/local/bin/noga-welcome"]="0:0:755"
  ["/usr/local/bin/noga-repos"]="0:0:755"
  ["/root"]="0:0:750"
  ["/root/customize_airootfs.sh"]="0:0:755"
)
