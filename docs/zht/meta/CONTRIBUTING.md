+++
title = "貢獻指南"
description = """感謝您有興趣貢獻！本指南涵蓋了您入門所需的一切"""
lang = "zht"
category = "meta"
+++

# 貢獻 Arona

感謝您有興趣貢獻！本指南涵蓋了您入門所需的一切。

## 貢獻政策（請先閱讀）

Arona 定義了 Entelecheia 平台中共享的 JSON-RPC 2.0 協定型別，
因此**正確性、向下相容性和穩定性優先於貢獻吞吐量**。
在發起 pull request 之前請閱讀本節。

- **高合併門檻，非公開路線圖。** 發起 PR 並不意味著它會被合併。

我們有意識地只接受少量變更，且僅在它們符合架構並通過審查時
才接受。這是設計上的選擇，並非無禮。

- **我們歡迎的內容：** 錯誤報告、針對性修復、新增（非破壞性）的

協定欄位、改進的文件，以及在程式碼之前的設計討論。

- **我們通常不會合併的內容：** 大型未經請求的重寫、對協定型別

表面的破壞性變更、未經事先設計討論的架構變更、批次「vibe-coding」
PR，以及任何降低型別合約相容性門檻的內容。

- **核心 vs. 外圍。** 協定型別定義及其序列化表面受到最嚴格的

門檻要求，並由核心團隊維護。

- **需要 CLA。** 每個被接受的貢獻都需要簽署貢獻者授權協議。

請參閱 [`CLA.md`](CLA.md)。提交必須帶有 `Signed-off-by` 行
（`git commit -s`）。

> **授權可能會開放；合併門檻不會。** 在 **2030-01-01**，本專案
> 將從 BUSL-1.1 轉換為 Apache-2.0 或 MIT（接收方可選擇）— 請參閱
> [`LICENSE`](LICENSE)。這擴大了*您可以對程式碼做什麼*的範圍；
> 它**不**會降低審查門檻、不會移除 CLA，也不意味著我們接受更多 PR。
> 貢獻政策在變更日期前後保持不變。

## 安全性

**不要**為安全漏洞開立公開 issue。請透過
[GitHub 安全諮詢](https://github.com/celestia-island/arona/security/advisories/new)
私下回報。請參閱 [`SECURITY.md`](SECURITY.md)。

## 行為準則

保持尊重、建設性和包容。我們遵循
[貢獻者公約行為準則](CODE_OF_CONDUCT.md)。

## 開發

Arona 是一個小型的 Rust crate。快速開始：

```bash
git clone https://github.com/celestia-island/arona.git
cd arona
cargo build
cargo test
cargo clippy -- -D warnings
```

- Rust 1.85+。
- 型別衍生 `ts-rs`（`#[derive(TS)]`）以生成 TypeScript 繫結 — 保持

`serde` 屬性和 `ts-rs` 註解一致。

- 不要對現有協定型別引入破壞性變更；偏好使用帶有 `#[serde(default)]`

的新增欄位。

## Pull Request 流程

1. 從 `main` 分叉並建立分支。
1. 先在 issue 中討論大型或影響協定的變更。
1. 按照 [Conventional Commits](https://www.conventionalcommits.org/)

進行原子提交。

1. 確保 `cargo fmt`、`cargo clippy -D warnings` 和 `cargo test` 通過。
1. 簽署 CLA 並在每個提交中新增 `Signed-off-by`。
1. 回應審查回饋；強制推送僅限於 rebase。

## 授權與 CLA

Arona 依據 **Business Source License 1.1 (BUSL-1.1)** 授權，
**變更日期為 2030-01-01**，屆時轉換為接收方可選擇的
**Apache-2.0 或 MIT**。對於所有內部、學術、政府、教育和
非商業用途，今天已等同於 Apache-2.0 或 MIT（請參閱
[`LICENSE`](LICENSE) 中的額外使用授權）。受限的商業用途
（作為服務託管、轉售或重新品牌化）在變更日期前需要
單獨的商業授權。

透過貢獻，您同意您的貢獻依據本專案的授權條款授權，
並且您簽署了 CLA（[`CLA.md`](CLA.md)）。CLA 授予本專案
一個寬鬆的授權，**包括重新授權的權利**，因此本專案可以
保持其 BUSL→Apache/MIT 路徑並在未來調整其授權。
