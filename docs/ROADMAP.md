# Noga Linux — hoja de ruta

Objetivo: una distro CachyOS con el escritorio Rakia que se pueda instalar sin miedo,
actualizar sin romperse y usar sin tocar la terminal. Esta lista está ordenada por
impacto, no por dificultad.

## Fase 0 — Base usable (en curso)

- [x] ISO live BIOS/UEFI con Umbriel + Noctalia + Noct.
- [x] Instalador Calamares offline con GRUB (BIOS/UEFI).
- [x] Capa de compatibilidad (`noct compat`, `noct msg`) ante updates de Umbriel/Noctalia.
- [x] Renderizado software cuando no hay GL por hardware (VMware, QEMU, `nomodeset`).
- [x] Instalador localiza el squashfs en Ventoy, YUMI y USB quemado.
- [x] Brave, Thunar, mousepad, `gvfs`.
- [x] Repositorios de CachyOS/Arch disponibles desde el primer arranque.
- [x] Widget `noct/effects` instalado y habilitado por defecto.
- [ ] Verificar en hardware real: Wi-Fi, suspensión, teclas de brillo, dos monitores.

## Fase 1 — Confianza y seguridad

- [x] **Greeter real**: Noctalia Greeter con contraseña en el sistema instalado;
      `noga-greetd --autologin` queda como opción explícita (live sí).
- [ ] **LUKS2 probado de punta a punta**: instalación, arranque, teclado español en
      cryptsetup, TRIM y hibernación.
- [ ] **Secure Boot**: `sbctl`, claves propias, firma de GRUB/kernel en el instalador y
      documentación del enrolado.
- [ ] **ISOs firmadas**: minisign/GPG sobre ISO y `SHA256SUMS`, publicadas en la release.
- [ ] **Cortafuegos por defecto** (`ufw` o `firewalld`) con reglas mínimas documentadas.
- [ ] Revisar permisos del usuario live y sudoers antes de cada release.

## Fase 2 — Instalador de verdad

- [ ] Instalar **junto a Windows/otro Linux** (alongside + resize) además de borrar disco.
- [ ] **Swap explícito**: zram (actual) o swapfile, explicado en la UI.
- [ ] **Sistemas de archivos con consecuencias**: ext4 por defecto; btrfs con snapshots.
- [ ] **Hostname y zona horaria** seleccionables (Madrid solo como propuesta).
- [ ] **Grupos de paquetes**: navegador/oficina/gaming/dev mediante metapaquetes `noga-*`.
- [ ] **Preflight**: espacio, batería, red, firmware, Secure Boot; avisar antes de formatear.
- [ ] Resumen del instalador sin rutas engañosas (`/dev/sda`).

## Fase 3 — Actualizaciones reversibles

- [x] btrfs + **snapper** + **grub-btrfs**: snapshots y entradas de rollback en GRUB
      cuando la raíz es btrfs (paquetes, config y hook del instalador).
- [ ] Integrar `cachy-update` en Noctalia: badge, notificación y “reiniciar para aplicar”.
- [ ] Estado del sistema en el panel: kernel, driver, versión, última actualización.
- [ ] Documentar y probar el rollback completo (snapshot → GRUB → sistema anterior).

## Fase 4 — Hardware

- [x] **Noga Welcome** (zenity): actualizar, drivers con `chwd`, LibreOffice, gaming,
      desarrollo y documentación. Multilib habilitado para Steam.
- [x] `fwupd` + `fwupd-refresh.timer`.
- [x] `power-profiles-daemon` habilitado.
- [x] Impresión: `cups` con socket habilitado y `system-config-printer`.
- [ ] Bluetooth con interfaz (Noctalia) y emparejamiento probado.
- [ ] Optimus/`prime-run` documentado y probado en portátil híbrido.
- [ ] Matriz de pruebas: Intel, AMD, NVIDIA, Wi-Fi, Ethernet, suspend/lid.

## Fase 5 — Pulido

- [ ] Plymouth + tema GRUB + greeter con la marca Kepler.
- [ ] Bloqueo de pantalla al suspender y al cerrar sesión.
- [x] Documentación offline en `/usr/share/doc/noga` (README, INSTALL, ROADMAP).
- [x] **Alt+Tab global**: HUD 3D con todas las ventanas de todos los monitores y
      workspaces, orden por uso reciente y salto directo (Quickshell).
- [ ] Fuentes, iconos, cursores, sonidos y OSD coherentes.
- [ ] Fondos propios y pantalla de bienvenida.

## Fase 6 — Ingeniería y distribución

- [ ] CI que construya la ISO en cada tag y haga **smoke test en QEMU**: arranque BIOS y
      UEFI, llegada al escritorio, instalación desatendida y arranque del sistema instalado.
- [x] `shellcheck`, `clippy`, `cargo test` y `bash -n` en CI.
- [ ] `build.sh --check` dentro de un contenedor Arch/CachyOS en CI.
- [ ] Versionado por fecha, changelog y notas de release generadas.
- [ ] Espejo/torrent además de GitHub Releases.
- [x] Inventario de paquetes (`.pkglist.txt`) junto a la ISO; firma opcional con
      `NOCT_SIGN_KEY` (minisign).
- [ ] Build reproducible verificado (`SOURCE_DATE_EPOCH` ya está).

## Opcionales (no bloquean)

- **AdGuard DNS**: filtrado de anuncios a nivel de sistema. Aplazado por el riesgo de
  romper portales cautivos y redes con DNS propio; si se retoma, hacerlo como opción
  explícita en el welcome, no por defecto.

## Decisiones abiertas

- **Kernel**: seguir con `linux` de Arch o pasar a `linux-cachyos` (+ variante NVIDIA).
  Más rendimiento y coherencia con CachyOS; más superficie de fallo con drivers.
- **Flatpak/Flathub**: cómodo para apps que no están en repos, pero rompe la promesa
  offline y añade otra fuente de actualizaciones.
- **Firewall**: activarlo por defecto puede romper servicios locales (KDE Connect, juegos);
  decidir reglas antes de activarlo.
- **Autologin**: conveniencia en live, riesgo en disco. Propuesta: preguntar en Calamares.

## Criterio de “state of the art”

1. Se instala en BIOS y UEFI, con o sin Secure Boot, sin tocar la terminal.
2. Un update no puede dejar el sistema sin arrancar: snapshot o rollback.
3. El escritorio sobrevive a updates de Umbriel/Noctalia sin pasos manuales.
4. Todo lo que hace falta para empezar (navegador, archivos, ajustes, drivers) está
   instalado o a un clic.
5. Cada release está firmada, probada en VM y publicada con changelog.
