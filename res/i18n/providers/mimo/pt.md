# Xiaomi MiMo

## Introdução

Xiaomi MiMo is a series of AI models developed by Xiaomi, designed for coding and general-purpose tasks. The Token Plan provides subscription-based access to MiMo models with OpenAI-compatible and Anthropic-compatible API endpoints across multiple regional clusters.

## Organização

Xiaomi Corporation é uma empresa chinesa electronics company that develops AI models under the MiMo brand. MiMo models are optimized for coding assistance and support function calling, streaming, and other OpenAI-compatible features.

## Plano de Token

The Token Plan is a subscription-based access model where API keys use the `tp-xxxxx` format (distinct from pay-as-you-go `sk-xxxxx` keys). Available clusters:

- China: `https://token-plan-cn.xiaomimimo.com/v1`
- Singapore: `https://token-plan-sgp.xiaomimimo.com/v1`
- Europe: `https://token-plan-ams.xiaomimimo.com/v1`

## Autenticação

The Token Plan uses a custom `api-key` header (not the standard `Authorization: Bearer`). The system automatically handles this when the auth type is set to `api-key`.

## Site Oficial

https://platform.xiaomimimo.com
