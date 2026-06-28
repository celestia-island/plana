# 貢獻指南（Evernight）

> 本檔案是貢獻政策的繁體中文版本。建構命令與詳細安裝步驟請見倉庫根目錄的英文
> [`CONTRIBUTING.md`](../../../CONTRIBUTING.md)；命令本身不翻譯。如有歧義，以英文版為準。

## 貢獻政策（請先閱讀）

Evernight 可驅動物理與工業系統，因此**穩定性與安全性優先於貢獻吞吐量**。在提交
Pull Request 之前，請先閱讀本節。

- **合併門檻高，並非公開路線圖。** 提交 PR 不等於會被合併。我們只接受數量刻意保持較少、
  且符合架構並通過審查的改動。這是有意為之，並非不禮貌。
- **歡迎的貢獻：** bug 報告、聚焦的修復、對**外延**（Layer 3 外掛、裝置 profile、LLM
  provider 適配器、整合、文件）的範圍明確的改進，以及在寫程式前的設計討論。
- **通常不予合併：** 大規模未經提議的重寫、沒有事先設計討論的架構變更、批量「vibe-coded」
  PR、任何降低核心安全或正確性門檻的改動，以及未經邀請與加長審查的對安全關鍵核心
  的改動。
- **核心 vs. 外延。** 核心（編排、微核心、安全）維持最嚴標準，主要由核心團隊維護。外延
  是外部貢獻最有用、也最可能被接受的地方。
- **必須簽署 CLA。** 每一個被接受的貢獻都需要簽署貢獻者授權協議，見 [`CLA.md`](../../../CLA.md)。
  提交須帶 `Signed-off-by`（`git commit -s`）。

> **授權會開放，合併門檻不會。** 在 **2030-01-01**，本專案從 BUSL-1.1 轉為 Apache-2.0 或
> MIT（接收者任選），見 [`LICENSE`](../../../LICENSE)。這放寬的是*你能用程式碼做什麼*，而
> **不是**降低審查門檻、取消 CLA，也不意味著我們會接受更多 PR。變更日期前後，貢獻政策不變。

## 安全

**不要**用公開 issue 回報安全漏洞。請透過
[GitHub Security Advisories](https://github.com/celestia-island/evernight/security/advisories/new)
私下回報。威脅模型與回應 SLA 見 [`SECURITY.md`](../../../SECURITY.md)。

## 行為準則

請保持尊重、建設性與包容。我們遵循 [Contributor Covenant Code of Conduct](../../../CODE_OF_CONDUCT.md)。

## Pull Request 流程

1. Fork 並從 `main` 拉取分支。
2. 大改動先在 issue 中討論。
3. 提交原子化、遵循 Conventional Commits。
4. 確保 `just ci`（或倉庫的 CI 命令）通過。
5. 簽署 CLA 並加上 `Signed-off-by`。
6. 回覆審議意見；force-push 僅用於 rebase。

## 授權與 CLA

採用 **BUSL-1.1**，**Change Date 為 2030-01-01**，屆時轉為接收者任選的 **Apache-2.0 或
MIT**。如今在內部營運、學術、政府、教育與非商業用途下，它已等同於 Apache-2.0 或 MIT（見
[`LICENSE`](../../../LICENSE) 中的 Additional Use Grant）。受限的商業用途（託管、轉售、作為
服務換皮銷售）在 Change Date 之前需另行取得商業授權。

提交貢獻即表示你同意貢獻依本專案授權授權，並簽署 CLA（[`CLA.md`](../../../CLA.md)）。CLA
授予專案**含再授權權在內的**寬鬆授權，使專案能維持 BUSL→Apache/MIT 的路徑，並在未來調整授權。
