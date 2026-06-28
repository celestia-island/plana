# 贡献指南（Evernight）

> 本文件是贡献政策的简体中文版本。构建命令与详细安装步骤请见仓库根目录的英文
> [`CONTRIBUTING.md`](../../../CONTRIBUTING.md)；命令本身不翻译。如有歧义，以英文版为准。

## 贡献政策（请先阅读）

Evernight 可驱动物理与工业系统，因此**稳定性与安全性优先于贡献吞吐量**。在提交
Pull Request 之前，请先阅读本节。

- **合并门槛高，并非公开路线图。** 提交 PR 不等于会被合并。我们只接受数量刻意保持较少、
  且符合架构并通过审查的改动。这是有意为之，并非不礼貌。
- **欢迎的贡献：** bug 报告、聚焦的修复、对**外延**（Layer 3 插件、设备 profile、LLM
  provider 适配器、集成、文档）的范围明确的改进，以及在写代码前的设计讨论。
- **通常不予合并：** 大规模未经提议的重写、没有事先设计讨论的架构变更、批量“vibe-coded”
  PR、任何降低核心安全或正确性门槛的改动，以及未经邀请与加长审查的对安全关键核心
  的改动。
- **核心 vs. 外延。** 核心（编排、微内核、安全）维持最严标准，主要由核心团队维护。外延
  是外部贡献最有用、也最可能被接受的地方。
- **必须签署 CLA。** 每一个被接受的贡献都需要签署贡献者许可协议，见 [`CLA.md`](../../../CLA.md)。
  提交须带 `Signed-off-by`（`git commit -s`）。

> **许可证会开放，合并门槛不会。** 在 **2030-01-01**，本项目从 BUSL-1.1 转为 Apache-2.0 或
> MIT（接收者任选），见 [`LICENSE`](../../../LICENSE)。这放宽的是*你能用代码做什么*，而
> **不是**降低审查门槛、取消 CLA，也不意味着我们会接受更多 PR。变更日期前后，贡献政策不变。

## 安全

**不要**用公开 issue 报告安全漏洞。请通过
[GitHub Security Advisories](https://github.com/celestia-island/evernight/security/advisories/new)
私下报告。威胁模型与响应 SLA 见 [`SECURITY.md`](../../../SECURITY.md)。

## 行为准则

请保持尊重、建设性与包容。我们遵循 [Contributor Covenant Code of Conduct](../../../CODE_OF_CONDUCT.md)。

## Pull Request 流程

1. Fork 并从 `main` 拉取分支。
2. 大改动先在 issue 中讨论。
3. 提交原子化、遵循 Conventional Commits。
4. 确保 `just ci`（或仓库的 CI 命令）通过。
5. 签署 CLA 并添加 `Signed-off-by`。
6. 回复审议意见；force-push 仅用于 rebase。

## 许可证与 CLA

采用 **BUSL-1.1**，**Change Date 为 2030-01-01**，届时转为接收者任选的 **Apache-2.0 或
MIT**。如今在内部运营、学术、政府、教育与非商业用途下，它已等同于 Apache-2.0 或 MIT（见
[`LICENSE`](../../../LICENSE) 中的 Additional Use Grant）。受限的商业用途（托管、转售、作为
服务换皮销售）在 Change Date 之前需另行获取商业授权。

提交贡献即表示你同意贡献按本项目许可证授权，并签署 CLA（[`CLA.md`](../../../CLA.md)）。CLA
授予项目**含再许可权在内的**宽松许可，使项目能保持 BUSL→Apache/MIT 的路径，并在未来调整许可。
