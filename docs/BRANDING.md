# Branding — notas para la pasada final

> **Summary (EN):** Noga = distro (brightness / Venus), Rakia = desktop (expanse /
> infinite scroll), Noct = integration layer. Logo idea: a Kepler polyhedron as
> technical line art on black; no crescent, moon or owl. Final pass pending.

Estado: **pendiente**. La ISO actual usa fondo negro y conserva el creciente
antiguo solo como marcador. No publicar branding definitivo hasta cerrar esta nota.

## Nombres

- **Noga Linux** — la distro. *Noga* (נוגה): Venus, brillo, lucero. Es el sistema,
  la ISO y el nombre que verá quien instale.
- **Rakia** — el escritorio. *Rakia* (רקיע): expansión, espacio infinito. Encaja con
  Niri y su scroll infinito; es la configuración Niri + Noctalia + Noct.
- **Noct** — la capa de integración: CLI, plugin de Noctalia, sync y configuración.
- **Emet** — framework de automatización QA, proyecto aparte; no forma parte del
  naming de la distro.

## Logo

Idea guardada: **el poliedro de Kepler**.

Candidatos:

- **Pequeño dodecaedro estrellado** `{5/2,5}`: el “poliedro de Kepler” más reconocible,
  12 caras pentagrammicas. Buena lectura a tamaño pequeño y muy geométrico.
- **Gran dodecaedro estrellado** `{5/2,3}`: más puntiagudo y estrellado.
- **Modelo de Kepler** (sólidos platónicos anidados): demasiado complejo para un icono;
  reservado para una ilustración grande o un wallpaper.

Estilo propuesto:

- línea fina, wireframe o aristas, sin relleno;
- blanco hueso o mint sobre negro;
- geometría técnica, sin lectura mística o religiosa;
- **sin creciente, sin luna y sin búho** en ninguna superficie.

Uso:

- **Noga** puede usar el poliedro completo (brillo, Venus, sistema).
- **Rakia** puede usar solo las aristas o una expansión del mismo poliedro
  (expansión, scroll infinito, escritorio).
- El logo es un acento; el fondo por defecto sigue siendo negro vacío.

## Paleta y tipografía

- Negro `#000000` / `#05060A`; texto `#E8F6EC`; acento mint `#7CFFB2` (revisar
  saturación en la pasada final).
- Tipografía geométrica sans: Inter, IBM Plex Sans o DejaVu Sans como fallback.
- Tagline provisional: “un escritorio hecho de noche” (español); revisar la versión
  inglesa cuando se cierre el naming.

## Superficies a actualizar

- `os-release`, `/etc/issue`, hostname y label de la ISO (`NOGA_...`).
- Calamares: `branding/noga/*` (logo, icono, welcome, slideshow) y
  `GRUB_DISTRIBUTOR`.
- Menús de syslinux y systemd-boot.
- `/etc/skel`: fondo negro y sesión `rakia.desktop`.
- README/README.es, guías, releases, nombres de paquetes (`noga-branding`,
  `noga-defaults`) y, si se separa, el repo de la distro.
- Retirar el creciente y el búho de todas las imágenes.

## Decisiones pendientes

- Variante exacta del poliedro (pequeño vs. gran dodecaedro estrellado).
- ¿Noga y Rakia comparten logo o llevan marcas separadas?
- Paleta y tipografía finales.
- Tagline final en inglés y español.
- ¿El repo `noct` se separa en `noga` (distro) y `rakia` (escritorio)?
