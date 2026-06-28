> **Nota**: Esta é uma tradução de referência da comunidade. Em caso de divergência, prevalece a versão em inglês `SECURITY.md` na raiz do repositório.

# Política de segurança

## Como relatar uma vulnerabilidade

**Não abra issues públicas para vulnerabilidades de segurança.**

Relate-as de forma privada via
[GitHub Security Advisories](https://github.com/celestia-island/arona/security/advisories/new).
Se o GitHub Security Advisories não estiver disponível para você, envie um e-mail ao mantenedor em
security@celestia.world com uma descrição clara e os passos de reprodução.

## Escopo

Dentro do escopo:

- Bypass de autenticação, falhas de JWT/OAuth, defeitos de tratamento de sessão
- Divulgação de chave de API / credenciais ou armazenamento inadequado
- Lacunas de autorização e aplicação de RBAC
- Vulnerabilidades de injeção (SQL, comando, SSRF, XSS)
- Desserialização insegura, path traversal, SSRF
- Problemas que permitem escalonamento de privilégios ou acesso entre locatários

Fora do escopo:

- Vulnerabilidades em dependências upstream não exploráveis através deste projeto
- Implantações auto-hospedadas com configuração insegura contrária à orientação documentada
- Ataques de negação de serviço contra os endpoints do provedor de LLM público

## Resposta

| Etapa | Objetivo |
| --- | --- |
| Confirmação de recebimento pelo Agent | 10 minutos |
| Confirmação de recebimento humana | 1 dia corrido |
| Avaliação inicial | 3 dias corridos |
| Correção ou mitigação | 30 dias corridos (depende da gravidade) |

Inclua por favor: (1) o componente e a versão afetados, (2) o vetor de ataque e o impacto, (3) os passos de reprodução e (4) as mitigações sugeridas.

## Versões suportadas

Apenas a linha de lançamento mais recente nos branches `main` / `dev` recebe correções de segurança.
