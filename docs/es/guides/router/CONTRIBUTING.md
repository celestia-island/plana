# Contribuir a Evernight

> Esta es la versión en español de la política de contribución. Los comandos de
> build y la instalación detallada están en el [`CONTRIBUTING.md`](../../../CONTRIBUTING.md)
> en inglés, en la raíz del repositorio; los comandos no se traducen. En caso de
> conflicto, prevalece la versión en inglés.

## Política de contribución (léase primero)

Evernight puede controlar sistemas físicos e industriales, por lo que **la
estabilidad y la seguridad prevalecen sobre el volumen de contribuciones**. Lea
esto antes de abrir un pull request.

- **Barra de fusión alta; no es una hoja de ruta pública.** Abrir un PR no implica
  que se fusionará. Aceptamos deliberadamente pocos cambios, solo cuando encajan
  en la arquitectura y superan la revisión. Es por diseño, no por descortesía.
- **Lo que se agradece:** reportes de bugs, correcciones enfocadas, mejoras bien
  delimitadas en la **periferia** (plugins Layer 3, perfiles de dispositivos,
  adaptadores de proveedores LLM, integraciones, documentación), y discusiones de
  diseño previas al código.
- **Lo que generalmente no se fusionará:** reescrituras grandes no solicitadas,
  cambios de arquitectura sin discusión previa, PR «vibe-coded» en masa, cualquier
  cosa que baje la barra de seguridad/corrección del núcleo, y cambios en el
  núcleo crítico para la seguridad sin una invitación explícita y revisión extendida.
- **Núcleo vs periferia.** El núcleo (orquestación, micronúcleo, seguridad) sigue
  la barra más estricta y lo mantiene principalmente el equipo núcleo. La periferia
  es donde las contribuciones externas son más útiles y más propensas a aceptarse.
- **CLA obligatorio.** Toda contribución aceptada requiere un CLA firmado, ver
  [`CLA.md`](../../../CLA.md). Los commits deben llevar una línea `Signed-off-by`
  (`git commit -s`).

> **La licencia se abre, la barra de fusión no.** El **2030-01-01** este proyecto
> pasa de BUSL-1.1 a Apache-2.0 o MIT (a elección del destinatario), ver
> [`LICENSE`](../../../LICENSE). Eso amplía *qué puedes hacer con el código*; **no**
> baja la barra de revisión, no elimina el CLA ni significa que aceptemos más PR.
> La política de contribución no cambia antes ni después de la fecha de cambio.

## Seguridad

**No** abra issues públicos para vulnerabilidades de seguridad. Repórtelas de
forma privada vía [GitHub Security Advisories](https://github.com/celestia-island/evernight/security/advisories/new).
El modelo de amenazas y el SLA están en [`SECURITY.md`](../../../SECURITY.md).

## Código de conducta

Sea respetuoso, constructivo e inclusivo. Seguimos el [Contributor Covenant Code of Conduct](../../../CODE_OF_CONDUCT.md).

## Proceso de pull request

1. Haga fork y ramifique desde `main`.
2. Discuta los cambios grandes primero en un issue.
3. Commits atómicos, Conventional Commits.
4. Asegúrese de que `just ci` (o el comando CI del repo) pase.
5. Firme el CLA y añada `Signed-off-by`.
6. Responda a la revisión; force-push solo para rebase.

## Licencia y CLA

Bajo **BUSL-1.1**, **Fecha de cambio 2030-01-01**, convertida a elección del
destinatario en **Apache-2.0 o MIT**. Para todo uso interno, académico, gubernamental,
educativo y no comercial, ya equivale hoy a Apache-2.0 o MIT (ver la Additional Use
Grant en [`LICENSE`](../../../LICENSE)). Los usos comerciales restringidos (hosting,
reventa, rebranding como servicio) requieren una licencia comercial separada hasta la
fecha de cambio.

Al contribuir, acepta que sus contribuciones se licencien bajo la licencia del proyecto
y que firme el CLA ([`CLA.md`](../../../CLA.md)). El CLA otorga al proyecto una licencia
permisiva **incluyendo el derecho a relicenciar**, para que pueda mantener su ruta
BUSL→Apache/MIT y adaptar su licencia en el futuro.
