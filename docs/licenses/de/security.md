> **Hinweis**: Dies ist eine Community-Referenzübersetzung. Bei Unstimmigkeiten ist die englische Version `SECURITY.md` im Wurzelverzeichnis des Repositorys maßgeblich.

# Sicherheitsrichtlinie

## Melden einer Schwachstelle

**Eröffnen Sie keine öffentlichen Issues für Sicherheitsschwachstellen.**

Melden Sie diese privat über
[GitHub Security Advisories](https://github.com/celestia-island/arona/security/advisories/new).
Wenn Ihnen GitHub Security Advisories nicht zur Verfügung stehen, senden Sie eine E-Mail an den Maintainer unter
security@celestia.world mit einer klaren Beschreibung und Reproduktionsschritten.

## Geltungsbereich

Im Geltungsbereich:

- Umgehung der Authentifizierung, JWT/OAuth-Schwächen, Sitzungsverarbeitungsfehler
- Offenlegung von API-Schlüsseln / Anmeldedaten oder unsachgemäße Speicherung
- Lücken bei der Autorisierung und Durchsetzung von RBAC
- Injektionsschwachstellen (SQL, Kommando, SSRF, XSS)
- Unsichere Deserialisierung, Pfadüberquerung, SSRF
- Probleme, die eine Rechteausweitung oder mandantenübergreifenden Zugriff ermöglichen

Nicht im Geltungsbereich:

- Schwachstellen in Upstream-Abhängigkeiten, die über dieses Projekt nicht ausnutzbar sind
- Self-Hosted-Bereitstellungen mit unsicherer Konfiguration entgegen der dokumentierten Vorgaben
- Denial-of-Service gegen die Endpunkte des öffentlichen LLM-Anbieters

## Reaktion

| Phase | Ziel |
| --- | --- |
| Agent-Empfangsbestätigung | 10 Minuten |
| Personalbestätigung | 1 Kalendertag |
| Ersteinschätzung | 3 Kalendertage |
| Fehlerbehebung oder Minderung | 30 Kalendertage (schweregradabhängig) |

Bitte geben Sie an: (1) die betroffene Komponente und Version, (2) den Angriffsvektor und die Auswirkungen, (3) die Reproduktionsschritte sowie (4) vorgeschlagene Gegenmaßnahmen.

## Unterstützte Versionen

Nur die neueste Release-Linie auf den `main` / `dev`-Branches erhält Sicherheitsfixes.
