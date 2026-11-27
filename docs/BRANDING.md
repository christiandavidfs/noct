# Branding — Noga Linux / Rakia

> **Summary (EN):** Noga = distro (brightness / Venus), Rakia = desktop (expanse /
> infinite scroll), Noct = integration layer. The mark is the small stellated
> dodecahedron {5/2,5}, a Kepler–Poinsot polyhedron, rendered as a pseudo-3D mint
> wireframe. No crescent, moon or owl anywhere.

Estado: **cerrado para la primera preview**. El creciente y el búho han sido
retirados de las superficies de la distro.

## Nombres

- **Noga Linux** — la distro. *Noga* (נוגה): Venus, brillo, lucero. Es el sistema,
  la ISO y el nombre que verá quien instale.
- **Rakia** — el escritorio. *Rakia* (רקיע): expansión, espacio infinito. Encaja con
  Umbriel y su scroll infinito; es la configuración Umbriel + Noctalia + Noct.
- **Noct** — la capa de integración: CLI, plugin de Noctalia, sync y configuración.
- **Emet** — framework de automatización QA, proyecto aparte; no forma parte del
  naming de la distro.

## Logo

Marca elegida: **pequeño dodecaedro estrellado** `{5/2,5}`.

- 12 vértices, 30 aristas, 12 caras pentagrammicas; es el poliedro de Kepler más
  reconocible.
- Render pseudo-3D: aristas de tubo, nodos en los vértices, luz mint y fondo negro.
- Se acepta la lectura geométrica de estrella; **no hay referencias islámicas,
  creciente, luna ni búho**.
- Archivos:
  - `docs/branding/logo-kepler-3d.png` — presentación sobre negro.
  - `docs/branding/logo-kepler-3d-transparent.png` — marca con alfa.
  - `iso/profile/airootfs/usr/share/rakia/branding/` — marca, avatar e icono usados
    por el escritorio.
- Usos:
  - icono del lanzador del dock y avatar de usuario en Noctalia;
  - logo e icono de Calamares y welcome del instalador;
  - base para el logo definitivo de Noga.

## Fondo y tema

- Fondo por defecto: `iso/profile/airootfs/usr/share/rakia/wallpapers/rakia-default.jpg`
  (imagen libre de Wallhaven, 16:9). Se queda como fondo de Rakia.
- Noctalia arranca en **modo oscuro** (`mode = "dark"`, `shell_mode = "dark"`).
- El fondo negro liso anterior queda retirado.

## Paleta y tipografía

- Negro `#000000` / `#05060A`; texto `#E8F6EC`; acento mint `#7CFFB2`.
- Tipografía geométrica sans: Inter, IBM Plex Sans o DejaVu Sans como fallback.
- Tagline: “un escritorio hecho de noche” (español); pendiente la versión inglesa.

## Superficies actualizadas

- `os-release`, `/etc/issue`, hostname y label de la ISO (`NOGA_...`).
- Calamares: `branding/noga/*` (logo, icono, welcome, slideshow) y
  `GRUB_DISTRIBUTOR`.
- Menús de syslinux y systemd-boot.
- `/etc/skel`: fondo por defecto, tema oscuro, dock con la marca y sesión
  `rakia.desktop`.
- README/README.es y guías.

## Decisiones pendientes

- Tagline final en inglés.
- ¿Noga y Rakia comparten marca o Rakia usa una variante (solo aristas)?
- ¿El repo `noct` se separa en `noga` (distro) y `rakia` (escritorio)?
