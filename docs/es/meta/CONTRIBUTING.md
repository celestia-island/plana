+++
title = "Guía de Contribución"
description = """¡Gracias por tu interés en contribuir! Esta guía cubre todo lo que necesitas"""
lang = "es"
category = "meta"
+++

# Contribuir a Arona

¡Gracias por tu interés en contribuir! Esta guía cubre todo lo que
necesitas para empezar.

## Política de contribución (lee esto primero)

Arona define los tipos de protocolo JSON-RPC 2.0 compartidos consumidos en toda la
plataforma Entelecheia, por lo que **la corrección, la compatibilidad hacia atrás y la estabilidad
prevalecen sobre el volumen de contribuciones**. Por favor, lee esto antes de abrir una pull
request.

- **Listón de fusión alto, no es una hoja de ruta pública.** Abrir un PR no implica que será

fusionado. Aceptamos un número deliberadamente pequeño de cambios, y solo cuando
encajan en la arquitectura y pasan la revisión. Es por diseño, no por descortesía.

- **Lo que damos la bienvenida:** informes de errores, correcciones enfocadas, campos de protocolo

aditivos (no disruptivos), documentación mejorada y discusiones de diseño previas antes del
código.

- **Lo que generalmente no fusionaremos:** grandes reescrituras no solicitadas, cambios

disruptivos en la superficie de tipos del protocolo, cambios arquitectónicos sin una
discusión de diseño previa, PRs masivos "vibe-coded", y cualquier cosa que baje el
listón de compatibilidad del contrato de tipos.

- **Núcleo vs. periferia.** Las definiciones de tipos del protocolo y su superficie de

serialización se mantienen con el listón más estricto y son mantenidas por el equipo central.

- **CLA requerido.** Cada contribución aceptada requiere un Acuerdo de Licencia del

Contribuyente firmado. Ver [`CLA.md`](CLA.md). Los commits deben incluir una
línea `Signed-off-by` (`git commit -s`).

> **La licencia puede abrirse; el listón de fusión no.** El **2030-01-01** este
> proyecto se convierte de BUSL-1.1 a Apache-2.0 o MIT (a elección del destinatario) — ver
> [`LICENSE`](LICENSE). Eso amplía *lo que puedes hacer con el código*; **no**
> reduce el listón de revisión, elimina el CLA, ni significa que aceptemos más PRs. La
> política de contribución no cambia antes ni después de la fecha de cambio.

## Seguridad

**No** abras issues públicos para vulnerabilidades de seguridad. Repórtalas de forma privada
a través de [Avisos de Seguridad de GitHub](https://github.com/celestia-island/arona/security/advisories/new).
Ver [`SECURITY.md`](SECURITY.md).

## Código de Conducta

Sé respetuoso, constructivo e inclusivo. Seguimos el
[Código de Conducta del Contributor Covenant](CODE_OF_CONDUCT.md).

## Desarrollo

Arona es una pequeña crate de Rust. Inicio rápido:

```bash
git clone https://github.com/celestia-island/arona.git
cd arona
cargo build
cargo test
cargo clippy -- -D warnings
```

- Rust 1.85+.
- Los tipos derivan `ts-rs` (`#[derive(TS)]`) para generar bindings de TypeScript — mantén

los atributos `serde` y las anotaciones `ts-rs` consistentes.

- No introduzcas cambios disruptivos en los tipos de protocolo existentes; prefiere campos

aditivos con `#[serde(default)]`.

## Proceso de pull request

1. Haz un fork y crea una rama desde `main`.
1. Discute los cambios grandes o que afecten al protocolo en un issue primero.
1. Haz commits atómicos siguiendo [Conventional Commits](https://www.conventionalcommits.org/).
1. Asegúrate de que `cargo fmt`, `cargo clippy -D warnings` y `cargo test` pasen.
1. Firma el CLA y añade `Signed-off-by` a cada commit.
1. Atiende los comentarios de revisión; limita los force-pushes solo a rebase.

## Licencia y CLA

Arona está licenciado bajo la **Business Source License 1.1 (BUSL-1.1)** con una
**Fecha de Cambio del 2030-01-01**, en la cual se convierte a elección del destinatario en
**Apache-2.0 o MIT**. Para todo uso interno, académico, gubernamental, educativo y
no comercial, ya es equivalente a Apache-2.0 o MIT hoy (ver la
Concesión de Uso Adicional en [`LICENSE`](LICENSE)). Los usos comerciales restringidos
(alojamiento, reventa o rebranding como servicio) requieren una licencia comercial
separada hasta la Fecha de Cambio.

Al contribuir, aceptas que tus contribuciones se licencian bajo la
licencia del proyecto y que firmas el CLA ([`CLA.md`](CLA.md)). El CLA concede
al proyecto una licencia permisiva **incluyendo el derecho de relicenciar**, para que el
proyecto pueda mantener su ruta BUSL→Apache/MIT y adaptar su licenciamiento en el futuro.
