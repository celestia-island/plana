+++
title = "TIA Portal 前置準備 —— 接入 evernight"
description = """如何在 TIA Portal 中對 S7-1200/1500 PLC 做一次性設定，使 evernight 能夠連線、自組網、讀寫裝置，且此後無需任何人工介入。"""
lang = "zht"
category = "guides"
subcategory = "router"
+++

# TIA Portal 前置準備 —— 接入 evernight

> **目標**：在 TIA Portal 中對西門子 S7-1200/1500 PLC 做**一次性**設定，使
> evernight 能夠連線、自組網、讀寫裝置，且**此後無需任何人工介入**。這只是一
> 次 CPU 屬性設定 —— 你永遠不必改動梯形圖/SCL 程式邏輯。

evernight 透過**兩條通道**連西門子 PLC，依你的 PLC 暴露情況二擇一：

| 通道 | 連接埠 | 存取方式 | 需要 TIA 準備 | 適用場景 |
|------|--------|----------|--------------|----------|
| **S7comm** | 102 | M / I / Q / DB 的裸位元組讀寫 | PUT/GET + 非最佳化 DB | 舊系統、精簡、無 OPC UA 授權 |
| **OPC UA** | 4840 | 符號化、自描述 | 啟用內建 server | **推薦** —— 自動探索、最佳化 DB 也可存取 |

若能開 OPC UA，優先用它：evernight 會**自動瀏覽**整棵符號位址空間，零人工錄
符號。

---

## 路徑 A —— S7comm（裸暫存器存取）

### A.1 開啟 PUT/GET 通訊

S7-1200/1500 預設禁止外部 S7 讀寫。

1. 在 **TIA Portal** 開啟專案。
2. 在裝置/網路檢視裡**點選 CPU**。
3. 屬性 → **保護與安全（Protection & Security）→ 連線機制（Connection mechanisms）**。
4. 勾選 **「允許來自遠端物件的 PUT/GET 通訊（Permit access with PUT/GET communication from remote partner）」**。
5. 將硬體設定下載到 CPU。

### A.2 把目標 DB 改為非最佳化

最佳化的區塊存取（S7-1200/1500 預設）沒有固定位元組偏移，絕對位址讀會失敗。對每個
evernight 要讀寫的 DB：

1. 右鍵 DB → **屬性（Properties）**。
2. 取消勾選 **「最佳化的區塊存取（Optimized block access）」**。
3. 重新編譯並下載。

> M 區標誌位與 I/Q 過程映像始終是位元組定址的，無需變更。這一步只針對 DB。

### A.3 用 evernight 連線

```
s7://192.168.1.10:102?rack=0&slot=1
```

- S7-1200/1500：`rack=0, slot=1`
- S7-300：`slot=2`

程式裡自組網：

```rust
use evernight::protocol::auto_provision;

let profile = auto_provision("192.168.1.10").await?;
// profile.data_blocks / profile.db_structures 現在描述了每個可讀 DB
```

---

## 路徑 B —— OPC UA（推薦）

### B.1 韌體與授權前提

- CPU 韌體 **V2.0+**（OPC UA 方法需 V2.5+）。
- 與 CPU 檔位匹配的 **SIMATIC OPC UA 執行系統授權**（在 CPU 屬性 →
  **執行系統授權（Runtime licenses）→ OPC UA** 處分配，合規所需）。

### B.2 啟用 OPC UA server

1. 在網路/裝置檢視裡**點選 CPU**。
2. 屬性 → **OPC UA → 一般（General）**：填一個 server 名稱。
3. 屬性 → **OPC UA → 伺服器（Server）**：勾選 **「啟用 OPC UA 伺服器（Activate OPC UA server）」**。
4. 把 server 分配給客戶端要存取的 **PROFINET 介面**。

### B.3 暴露符號變數

- **OPC UA → 伺服器 → 伺服器介面（Server interface）**：選 **「標準 SIMATIC 伺
  服器介面（Standard SIMATIC server interface）」**，這樣每個符號變數/DB（含
  最佳化 DB）都會自動發布。

### B.4 認證與安全

- **OPC UA → 伺服器 → 使用者認證（User authentication）**：匿名（受信任區網）
  或使用者名稱/密碼。
- **OPC UA → 伺服器 → 安全（Security）**：選策略。首次連線用 `None` 最簡單；
  正式環境用 `Sign & Encrypt`。
- 在韌體 **V3.1+ 且 TIA V19+** 上，把 **「OPC UA 伺服器存取（OPC UA server
  access）」** 功能角色/執行權限授予連線使用者。

### B.5 信任客戶端憑證

OPC UA 客戶端連線時出示 X.509 憑證；PLC 會隔離未知憑證。evernight 首次連線後：

1. TIA Portal → **CPU → 憑證（Certificates）**（線上），**或**
2. PLC **Web 伺服器** → 「與 OPC UA 客戶端的通訊」，**或**
3. CPU 顯示器的憑證管理員。

然後**接受/信任** evernight 的客戶端憑證。

### B.6（可選）匯出 OPC UA NodeSet XML

NodeSet 檔案是所有變數的離線地圖，便於無即時連線時預規劃：

1. CPU 屬性 → **OPC UA → 伺服器 → 匯出（Export）**。
2. 點 **「匯出 OPC UA XML 檔案（Export OPC UA XML file）」**，儲存
   `*.Opc.Ua.NodeSet2.xml`。

### B.7 下載

下載**硬體設定**。這些是 CPU 屬性，不是程式邏輯 —— 你的梯形圖/SCL 程式碼原封不動。

### B.8 用 evernight 連線

端點 URL：

```
opc.tcp://192.168.1.10:4840
```

evernight 作為 OPC UA 客戶端連上，**瀏覽**整棵符號樹，依名稱讀寫 —— 零人工錄
符號，最佳化 DB 照樣可存取。

---

## 驗證連通性（零風險探測）

驅動輸出前，用唯讀探測確認通道存活：

```bash
# 102 連接埠是否在講 S7comm？
evernight probe 192.168.1.10 --ports 102

# 4840 連接埠 OPC UA server 是否起來？
evernight probe 192.168.1.10 --ports 4840
```

二者都是被動交握 —— 不讀不寫任何東西。

---

## 安全邊界

- **絕不要**把安全聯鎖（急停、限位、過載）放到 S7/OPC UA 鏈路上。讓它們留在
  PLC 掃描裡。網路一斷絕不能讓安全功能失效。
- evernight 控制適合**慢速**負載（閥門、狀態機、模式切換）。S7/OPC UA 往返延
  遲約 10–50 ms —— 監督級夠用，運動/伺服太慢。
- 優先寫**命令 M 位**，由現有 PLC 邏輯去執行（你搶佔觸發源），而不是直接寫 Q
  輸出。

---

## 故障排查

| 現象 | 可能原因 | 處理 |
|------|----------|------|
| S7 連線被拒 / 無 COTP 確認 | PUT/GET 未開；rack/slot 錯；防火牆 | A.1；確認 `rack=0 slot=1`（1200/1500） |
| DB 讀回傳 "optimized" / InvalidAddress | 最佳化區塊存取開著 | A.2 —— 取消最佳化存取，重新編譯 |
| OPC UA 端點不可達 | server 未啟用；未下載；缺授權 | B.2 / B.7 / B.1 |
| OPC UA 連上後又被拒 | 客戶端憑證未信任 | B.5 |
| 瀏覽回傳空 | 標準 SIMATIC 介面未啟用 | B.3 |

---

## 參考

- [啟用 OPC UA server（S7-1500）— STEP 7 V20 文件](https://docs.tia.siemens.cloud/r/en-us/v20/configuring-automation-systems/using-opc-ua-communication-s7-1200-s7-1500-s7-1500t/using-the-s7-1500-as-an-opc-ua-server-s7-1500-s7-1500t/configuring-the-opc-ua-server-s7-1500-s7-1500t/enabling-the-opc-ua-server-s7-1500-s7-1500t)
- [存取 OPC UA server（端點 URL / 4840 連接埠）](https://docs.tia.siemens.cloud/r/en-us/v20/configuring-automation-systems/using-opc-ua-communication-s7-1200-s7-1500-s7-1500t/using-the-s7-1500-as-an-opc-ua-server-s7-1500-s7-1500t/configuring-the-opc-ua-server-s7-1500-s7-1500t/access-to-the-opc-ua-server-s7-1500-s7-1500t)
- [匯出 OPC UA XML 檔案（NodeSet）](https://docs.tia.siemens.cloud/r/en-us/v20/configuring-automation-systems/using-opc-ua-communication-s7-1200-s7-1500-s7-1500t/using-the-s7-1500-as-an-opc-ua-server-s7-1500-s7-1500t/configuring-the-opc-ua-server-s7-1500-s7-1500t/accessing-opc-ua-server-data-s7-1500-s7-1500t/export-opc-ua-xml-file-s7-1500-s7-1500t)
