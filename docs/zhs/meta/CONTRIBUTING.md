+++
title = "贡献指南"
description = """感谢您对贡献的兴趣！本指南涵盖了您需要了解的全部内容"""
lang = "zhs"
category = "meta"
+++

# 为 Arona 做贡献

感谢您对贡献的兴趣！本指南涵盖了您需要了解的全部内容以帮助您
快速入门。

## 贡献政策（请先阅读）

Arona 定义了在 Entelecheia 平台中共享的 JSON-RPC 2.0 协议类型，
因此**正确性、向后兼容性和稳定性优先于贡献吞吐量**。在提交
pull request 之前，请先阅读本部分。

- **高合并门槛，非公开路线图。** 提交 PR 并不意味着它会被合并。

我们仅接受有意为之的少量变更，且仅当其符合架构要求并通过审核时
才会合并。这是设计使然，并非无礼之举。

- **我们欢迎的内容：** 缺陷报告、针对性修复、非破坏性的新增协议

字段、改进的文档，以及在编写代码之前进行的设计讨论。

- **我们通常不会合并的内容：** 未经请求的大规模重写、对协议类型

接口的破坏性变更、未经事先设计讨论的架构变更、大量"氛围编码"
的 PR，以及任何降低类型契约兼容性标准的内容。

- **核心与外围。** 协议类型定义及其序列化接口受到最严格的标准

约束，由核心团队维护。

- **需要 CLA。** 所有被接受的贡献都需要签署贡献者许可协议。

参见 [`CLA.md`](CLA.md)。提交必须包含 `Signed-off-by` 行
（`git commit -s`）。

> **许可可能会开放；合并门槛不会。** 在 **2030-01-01**，本项目
> 将从 BUSL-1.1 转换为 Apache-2.0 或 MIT（接收方自行选择）—— 参见
> [`LICENSE`](LICENSE)。这会扩大*您可以对代码做什么*；但**不会**
> 降低审核标准、不会取消 CLA，也不意味着我们会接受更多的 PR。
> 贡献政策在变更日期前后保持不变。

## 安全

**不要**为安全漏洞公开发布 issue。请通过
[GitHub Security Advisories](https://github.com/celestia-island/arona/security/advisories/new)
私下报告。参见 [`SECURITY.md`](SECURITY.md)。

## 行为准则

请保持尊重、建设性和包容。我们遵循
[Contributor Covenant 行为准则](CODE_OF_CONDUCT.md)。

## 开发

Arona 是一个小型 Rust crate。快速开始：

```bash
git clone https://github.com/celestia-island/arona.git
cd arona
cargo build
cargo test
cargo clippy -- -D warnings
```

- Rust 1.85+。
- 类型派生 `ts-rs`（`#[derive(TS)]`）以生成 TypeScript 绑定——请保持

`serde` 属性和 `ts-rs` 注解的一致性。

- 不要对现有协议类型引入破坏性变更；优先使用带

`#[serde(default)]` 的增量字段。

## Pull Request 流程

1. 从 `main` 分支 fork 并创建新分支。
1. 先通过 issue 讨论影响广泛或涉及协议的变更。
1. 制作遵循 [Conventional Commits](https://www.conventionalcommits.org/) 的

原子化提交。

1. 确保 `cargo fmt`、`cargo clippy -D warnings` 和 `cargo test` 全部通过。
1. 签署 CLA 并在每个提交中添加 `Signed-off-by`。
1. 回应审核反馈；仅在变基时使用 force-push。

## 许可与 CLA

Arona 依据 **Business Source License 1.1 (BUSL-1.1)** 许可，**变更日期为
2030-01-01**，届时将转换为接收方可选择的 **Apache-2.0 或 MIT**。
对于所有内部、学术、政府、教育和非商业用途，它今天已等同于 Apache-2.0
或 MIT（参见 [`LICENSE`](LICENSE) 中的附加使用授权）。受限的商业用途
（托管、转售或作为服务重新品牌化）在变更日期之前需要单独的
商业许可。

通过贡献，您同意您的贡献按照项目许可进行许可，并且您签署了 CLA
（[`CLA.md`](CLA.md)）。CLA 授予项目一项宽松的许可，**包括再许可的权利**，
以便项目可以保持其 BUSL→Apache/MIT 的许可路径，并在未来调整其许可。
