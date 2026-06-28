+++
title = "About Shittim Chest"
description = """Version 0.1.0"""
lang = "en"
category = "architecture"
subcategory = "webui"
+++

# Shittim Chest (什亭之匣)

**Version 0.1.0**

Shittim Chest is the user-facing shell for the [entelecheia](https://github.com/celestia-island/entelecheia) multi-agent collaboration platform, built with Rust and Vue 3.

## Architecture

Shittim Chest consists of several components that work together to provide a complete user experience:

- **arona** — The chat UI you are currently using, featuring streaming responses, image generation, agent status monitoring, thinking window, remote device viewer, and multi-language support.
- **`shittim_chest`** — The unified Rust + Axum backend handling authentication (JWT + OAuth), independent LLM routing, chat API, image generation, webhook ingress, scepter proxy, and remote device signaling.

## Relationship with Entelecheia

[entelecheia](https://github.com/celestia-island/entelecheia) is the core multi-agent orchestration engine. It provides the agent runtime (scepter, 13 specialized agents, Cosmos/IEPL runtime). Shittim Chest handles everything the user directly interacts with — identity, presentation, and communication.

The two projects are separated by design: entelecheia manages agent orchestration, while shittim-chest manages user identity and presentation. They communicate via JWT-authenticated HTTP/WebSocket. Login credentials live in `shittim_chest_db`; permissions and identity data live in entelecheia_db. This separation allows the frontend shell to evolve independently of the agent core.

## Relationship with Hikari

[hikari](https://github.com/celestia-island/hikari) is the gateway and routing layer for the Celestia Island ecosystem. It serves as the entry point for all external traffic, handling request routing, load balancing, and API gateway functionality between shittim-chest, entelecheia, and other services.

## Relationship with Tairitsu

[tairitsu](https://github.com/celestia-island/tairitsu) is the cross-platform native application framework for the Celestia Island ecosystem. It provides Tauri-based desktop and mobile clients that wrap arona as a native application, along with the browser automation and testing infrastructure that powers the development workflow.

## License

Shittim Chest is licensed under the **Business Source License 1.1 (BSL-1.1)**.

For **non-commercial use** — including internal operations, academic research, teaching, personal study, evaluation, government and public service, and educational use — the granted rights are equivalent to the **Synthetic Source License 1.0 (SySL-1.0)** (the "Free Use License"). You may freely use, study, modify, and run the software for these purposes.

**Commercial use** — such as offering the software as a hosted service to third parties, redistributing it as a standalone product, or using it as a core component of a commercial offering — requires a separate commercial license from the Licensor.

See the [full license text](https://github.com/celestia-island/shittim-chest/blob/main/LICENSE) for details.

---

Built with ❤ by [Celestia Island](https://github.com/celestia-island).
