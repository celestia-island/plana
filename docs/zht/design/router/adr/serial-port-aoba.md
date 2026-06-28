# 透過 aoba 進行序列埠通訊

- **狀態**：已接受
- **日期**：2026-06-09
- **作者**：Evernight Core Team

## 背景

Evernight 需要序列埠支援，用於嵌入式裝置管理和工業協定（Modbus RTU）探測。同級 crate `aoba` 已經提供了跨平台的序列埠列舉、VID/PID/序號擷取，以及 Modbus RTU/TCP 主站功能。

## 決策

將所有序列埠操作委派給 aoba。Evernight 的 `serial` 模組定義型別（`SerialConfig`、`SerialPortInfo`），並透過序列傳輸實作 `TerminalBackend`，呼叫 aoba 進行實際的埠 I/O。協定自動偵測（Modbus RTU 鮑率／校驗位掃描）也委派給 aoba。

## 後果

### 正面

- 重用經過驗證的程式碼；aoba 處理跨平台的邊緣情況。

### 負面

- 新增對 aoba 的相依；序列埠功能需要 aoba 可用。
