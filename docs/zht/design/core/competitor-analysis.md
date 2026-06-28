+++
title = "多代理框架競爭分析"
description = """日期：2026 年 5 月 12 日（對 43 個 crate × 1500+ 個原始檔進行完整原始碼稽核後更新）"""
lang = "zht"
category = "design"
subcategory = "core"
+++

# 多代理框架競爭分析

**日期**：2026 年 5 月 12 日（對 43 個 crate × 1500+ 個原始檔進行完整原始碼稽核後更新）
**上下文**：針對 Entelecheia（玄枢）設計維度的結構化比較。

> 當前狀態說明：本文件中對 Entelecheia 的引用混合了當前程式碼現實與預期架構。請將「vs. Entelecheia」章節視為與 Entelecheia 設計目標的比較，而非聲稱每項功能今日已完整交付。如需當前實作的真實情況，請優先參閱此附錄和 2026-05-13 診斷報告。

---

## 1. CrewAI

**倉庫**：[crewAIInc/crewAI](https://github.com/crewAIInc/crewAI)
**語言**：Python
**授權**：MIT
**規模**：~23k+ 星。獨立於 LangChain。

### 架構

- **代理**：透過 YAML（角色、目標、背景故事）或 Python `Agent` 類別定義。每個代理包裝一個 LLM 並具備工具存取。
- **編排**：兩種模式：
  - **Crews**：具有順序或階層式流程的代理團隊。順序式按順序執行任務；階層式分配一個「管理者」代理進行委派。
  - **Flows**：事件驅動的 DAG，使用 `@start`、`@listen`、`@router` 裝飾器。Pydantic 型別狀態。支援 `and_`/`or_` 條件組合器。
- **通訊**：透過 Crew/Flow 執行時期進行訊息傳遞。代理產生結構化輸出（`output_pydantic`、`output_json`）。
- **流程類型**：`Process.sequential` 和 `Process.hierarchical`。

### 工具暴露

- 透過 `crewai[tools]` 套件的內建工具（SerperDev 等）。自訂工具作為 Python 函數。
- 文件記載的 MCP（模型上下文協定）支援。
- 工具在定義時分配給每個代理。
- 每次 LLM 呼叫沒有顯式的工具數量限制 — 所有分配的工具有每次回合中都暴露。

### 安全模型

- **無沙箱化**。代理在與編排相同的 Python 程序中運行。
- 企業版「AMP Suite」提供具備可觀測性和存取控制的控制平面（專有）。
- 透過任務上的 `human_input=True` 實作人機迴圈。
- 未提及程式碼執行隔離。

### 記憶/上下文

- 短期：透過對話歷史的代理記憶。
- 長期：透過代理上的 `memory=True` 啟用的可選記憶儲存。
- 無顯式的上下文壓縮或 Token 管理 — 依賴 LLM 上下文視窗。
- 文件中提及檢查點，但 OSS 中的細節稀疏。

### 獨特功能

- **Flows + Crews 協同**：結合自主代理團隊與精確的事件驅動工作流程。
- **YAML 優先配置**：代理和任務以宣告方式定義，適合非開發人員。
- **大型社群**：透過 `learn.crewai.com` 擁有 100k+ 認證開發者。
- **效能聲稱**：在某些 QA 任務中比 LangGraph 快 5.76 倍（自我報告）。

### 相對於 Entelecheia 設計目標的潛在差距

- 無程式碼執行沙箱化或隔離。
- 無正式的工具執行安全模型。
- 記憶相對簡單 — 無階層式上下文管理或歸檔。
- 單程序 Python 執行時期限制跨機器擴展性。
- 無原生的瀏覽器/Shell 整合用於代理程式碼任務。
- 編排僅限 Python（無多語言執行時期）。

---

## 2. LangGraph

**倉庫**：[langchain-ai/langgraph](https://github.com/langchain-ai/langgraph)
**語言**：Python（也透過 `langgraphjs` 支援 JS/TS）
**授權**：MIT

### 架構

- **基於圖形的狀態機**：節點（代理/函數）和邊（轉換）形成一個受 Pregel/Beam 啟發的有向圖。
- **代理**：節點可以是 LLM 呼叫、工具執行或任何 Python 函數。沒有強型別化為「代理」—— 更像是圖形中的函數。
- **編排**：具有型別狀態模式的 `StateGraph`。節點讀取/寫入狀態。條件邊用於分支。子圖用於組合。
- **通訊**：狀態物件是單一事實來源。訊息附加到狀態列表中。

### 工具暴露

- 工具是 LangChain 工具或綁定到圖形節點的任意可呼叫物件。
- 節點中所有可用的工具都在該步驟中暴露給 LLM。
- 無內建的工具節流；開發者管理每個節點傳遞哪些工具。

### 安全模型

- **無沙箱化**。程式碼執行是開發者的責任。
- 透過 `interrupt()` 實作人機迴圈 — 暫停圖形執行，允許狀態檢查/修改。
- 持久執行：狀態持久化，可在失敗後恢復（檢查點）。
- 無工具執行或檔案系統存取的隔離原語。

### 記憶/上下文

- **短期**：透過狀態的工作記憶（訊息列表）。
- **長期**：透過 `Store` 抽象的跨會話持久記憶（具有嵌入的鍵值儲存）。
- 上下文壓縮非內建 — 開發者管理狀態大小。
- 透過 `MemorySaver` 或 `SqliteSaver` 實作檢查點。

### 獨特功能

- **持久執行**：在失敗/逾時後自動從檢查點恢復 — 非常適合長期運行的代理。
- **透過中斷的人機迴圈**：用於審批工作流程的強大模式。
- **LangSmith 整合**：深度可觀測性、追蹤和評估。
- **LangSmith 部署**：具備視覺原型設計的生產部署平台。
- **Deep Agents**：用於規劃、使用子代理並利用檔案系統的代理新子專案。
- **LangChain 生態系統**：與 LangChain 工具、模型和組件的無縫整合。

### 相對於 Entelecheia 設計目標的潛在差距

- 與 LangChain 生態系統緊密耦合（儘管「可以不使用 LangChain」）。
- 無安全模型 — 無沙箱化、無權限系統。
- 低階框架 — 代理互動需要大量樣板程式碼。
- 無原生的多代理通訊協定（A2A）。
- 狀態圖方法對於複雜的代理互動可能變得難以管理。
- 無內建的程式碼執行隔離支援。

---

## 3. MetaGPT

**倉庫**：[`FoundationAgents`/MetaGPT](https://github.com/`FoundationAgents`/MetaGPT)
**語言**：Python
**授權**：MIT
**研究**：發表於 ICLR 2024

### 架構

- **基於 SOP 的多代理**：模擬一個具有預定義角色的軟體公司（PM、架構師、工程師等）。
- **代理（角色）**：每個 `Role` 具有設定檔、目標、約束和一組 `Action`。角色使用 ReAct 循環（思考 → 行動），具有三種模式：`REACT`、`BY_ORDER`、`PLAN_AND_ACT`。
- **編排**：`Team` 類別雇用角色、投入預算、執行輪次。`Environment` 透過發布-訂閱管理角色間的訊息傳遞。
- **通訊**：透過 Environment 的基於訊息的發布/訂閱。角色訂閱特定的訊息標籤。
- **核心哲學**：`Code = SOP(Team)` — 具現化的標準作業程序。

### 工具暴露

- 動作是預定義的 Python 類別（`WriteCode`、`DesignAPI`、`DebugError` 等）— 約 40+ 種動作類型。
- 每個角色在建構時獲得指定的特定動作。
- 工具包括：網頁搜尋引擎（Serper、SerpAPI、DuckDuckGo、Google、Bing）、網頁瀏覽器（Playwright、Selenium）、圖像生成（DALL-E）、文件儲存（Chroma、FAISS、Milvus、LanceDB、Qdrant）。
- LLM 僅看到其當前分配動作的動作模式，而非完整的工具集。

### 安全模型

- **無沙箱化**。程式碼在相同環境中生成和執行。
- 提供 Dockerfile 但用於部署，而非每任務隔離。
- 預算追蹤：`investment` 參數限制 LLM API 總成本，超出時引發例外。
- 執行期間無人機迴圈。
- 無工具執行沙箱化或權限模型。

### 記憶/上下文

- **短期**：`RoleContext` 中的 `Memory` 類別 — 每個角色有序的訊息列表。
- **長期**：用於持久知識的 `LongTermMemory` 和 `BrainMemory` 類別。
- **工作記憶**：規劃器操作的獨立工作記憶。
- **訊息緩衝區**：具備按訂閱標籤過濾的異步訊息佇列。
- 上下文壓縮未顯式處理 — 角色觀察過濾後的訊息子集。

### 獨特功能

- **完整 SDLC 模擬**：使用 SOP 模擬整個軟體公司 — 使用者故事、需求、設計文檔、程式碼、測試。
- **多種文件儲存**：5+ 向量資料庫選項。
- **廣泛的提供者支援**：12+ LLM 提供者（OpenAI、Azure、Anthropic、Gemini、Ollama、Bedrock 等）。
- **Data Interpreter**：用於資料科學任務的專業代理。
- **研究產出**：多篇已發表的論文（AFlow、SPO、SELA、FACT、Data Interpreter）。
- **MGX**：建構於其上的自然語言程式設計產品。

### 相對於 Entelecheia 設計目標的潛在差距

- 僵化的 SOP — 角色和動作是預定義的；自訂角色需要程式碼。
- 完全沒有安全沙箱化。
- 單機架構 — 無分散式代理部署。
- 無基於瀏覽器的代理控制（僅 CLI/API）。
- 記憶模型是相當基礎的每角色佇列。
- 無 MCP/A2A 代理間協定支援。
- 預算追蹤僅限成本，非基於資源/安全。

---

## 4. ChatDev 2.0 (DevAll)

**倉庫**：[OpenBMB/ChatDev](https://github.com/OpenBMB/ChatDev)
**語言**：Python（後端），Vue 3（前端）
**授權**：Apache 2.0
**研究**：多篇 NeurIPS/arxiv 論文

### 架構

- **零程式碼多代理平台**：代理和工作流程完全以 YAML 配置定義。無需程式碼。
- **YAML 驅動的工作流程 DAG**：節點定義代理，邊定義訊息流。支援子圖。Web UI 中的視覺化拖放畫布。
- **核心模組**：`runtime/`（代理執行）、`workflow/`（DAG 編排）、`entity/`（配置）、`server/`（FastAPI + WebSocket）、`frontend/`（Vue 3 Web 控制台）。
- **代理**：在 YAML 節點配置中定義，包含提示、LLM 配置、工具和記憶設定。
- **編排**：多種執行器類型：順序、DAG、並行、循環、動態邊。拓撲建構器將 YAML 配置轉換為可執行圖形。

### 工具暴露

- **函數呼叫系統**：`functions/function_calling/` 包含內建工具（`code_executor`、file、weather、web、video、`deep_research`、uv、user）。
- **自訂工具註冊**：`functions/` 目錄中的 Python 函數自動發現。
- **MCP 支援**：`mcp_example/mcp_server.py` 展示 MCP 整合。
- 工具在 YAML 配置中按節點分配。

### 安全模型

- **程式碼執行**：專用的 `code_executor.py`，具備可配置的執行參數。
- Docker Compose 部署可用。
- **人機迴圈**：`demo_human.yaml` 工作流程、使用者輸入節點、確認流程。
- **WebSocket 串流**：即時日誌監控和產物檢查。
- 對代理無顯式的沙箱化/隔離模型。

### 記憶/上下文

- **多種記憶後端**：簡單記憶、`mem0` 記憶（持久、可學習）、基於檔案的記憶。
- **YAML 中的記憶配置**：`store`、`context_window_size`、每節點記憶類型。
- **上下文重置節點**：顯式的 `context_reset` 工作流程節點。
- **記憶嵌入一致性**：跨會話嵌入一致性的測試。
- **工作區掃描**：用於基於檔案的上下文注入的 `workspace_scanner.py`。

### 獨特功能

- **零程式碼**：無需編寫 Python 程式碼即可建構多代理系統 — YAML + Web UI。
- **拖放 Web 控制台**：視覺化工作流程設計器，即時啟動監控。
- **豐富的工作流程範本**：資料視覺化、3D 生成（Blender）、遊戲開發、深度研究、教學影片。
- **子圖支援**：可重用的子工作流程（`react_agent.yaml`、`reflexion_loop.yaml`）。
- **OpenClaw 整合**：可被 OpenClaw 程式碼代理呼叫以動態建立代理團隊。
- **Python SDK**：用於程式化工作流程執行的 `chatdev` PyPI 套件。
- **研究傳承**：MacNet、Puppeteer、IER、體驗式共同學習 — 全部建構於 ChatDev 上。
- **多代理電子書**：策劃的多代理研究集合。

### 相對於 Entelecheia 設計目標的潛在差距

- YAML 為中心限制了複雜邏輯的表現力。
- 程式碼執行無安全沙箱化。
- Web 控制台是主要介面 — 不太適合無頭/嵌入式使用。
- 類似 Django：固執的專案結構，不如 Python 原生框架靈活。
- 無跨語言代理支援。
- 社群偏向研究導向而非企業導向。

---

## 5. Google ADK（代理開發套件）

**倉庫**：[google/adk-python](https://github.com/google/adk-python)
**語言**：Python（也有 Java、Go 版本）
**授權**：Apache 2.0

### 架構

- **程式碼優先**：代理、工具和編排在 Python 程式碼中定義。
- **基礎抽象**：Agent（藍圖）、Tool（能力）、Runner（引擎）、Session（對話狀態）、Memory（跨會話回憶）、Artifact Service（檔案）。
- **代理類型**：`LlmAgent`（LLM 驅動）、`LoopAgent`、`SequentialAgent`、`ParallelAgent`、`RemoteA2aAgent`。
- **多代理組合**：父代理具有 `sub_agents` 列表。Runner 處理代理間的路由。
- **A2A 協定**：對遠端代理的原生代理對代理通訊協定支援。
- **LangGraph 整合**：`langgraph_agent.py` 用於將 LangGraph 圖形嵌入為代理。

### 工具暴露

- **豐富的工具生態系統**：50+ 內建工具 — Google Search、BigQuery、Bigtable、Spanner、PubSub、Vertex AI Search、MCP 工具、OpenAPI 工具、LangChain 工具、CrewAI 工具、電腦使用、bash、程式碼執行、Google API。
- **工具類型**：`FunctionTool`、`AgentTool`（將代理包裝為工具）、`MCPTool`、`OpenAPITool`、`LangChainTool`、`CrewAiTool`、`SkillToolset`。
- **工具確認（HITL）**：工具執行前的顯式確認流程，具備自訂輸入。
- **工具箱模式**：`toolbox_toolset.py` 用於捆綁工具。

### 安全模型

- **程式碼執行沙箱化**：多種執行器 — `container_code_executor.py`、`unsafe_local_code_executor.py`、`vertex_ai_code_executor.py`、`agent_engine_sandbox_code_executor.py`、`gke_code_executor.py`。
- **認證系統**：完整的 OAuth2 流程、憑證管理、認證預處理器、`authenticated_function_tool.py`。
- **人機迴圈**：工具確認、中斷支援。
- **Skills**：可打包、版本控制的代理能力，具有獨立的執行上下文。
- **代理身份**：用於服務帳戶的 `agent_identity/` 整合。

### 記憶/上下文

- **Session**：每個會話的完整對話歷史，透過 `SessionService` 持久化（記憶體內、SQLite、PostgreSQL、Vertex AI）。
- **Memory**：跨會話回憶，透過 `MemoryService` — 記憶體內、Vertex AI Memory Bank、Vertex AI RAG。
- **上下文壓縮**：用於自動上下文摘要的 `compaction.py` 和 `llm_event_summarizer.py`。
- **產物服務**：非文字資料（檔案、圖像）的獨立處理。
- **上下文快取**：Gemini 專用上下文快取的 `gemini_context_cache_manager.py`。
- **回溯**：能夠將會話倒回到先前呼叫之前的狀態。

### 獨特功能

- **多語言支援**：Python、Java、Go 版本的 ADK。
- **生產部署**：`adk deploy` 到 Cloud Run、Vertex AI Agent Engine、GKE。
- **A2A 協定**：源自 Google — 原生的多供應商代理通訊。
- **ADK Web**：內建的開發/除錯 UI，具備事件追蹤。
- **評估框架**：多層測試（單元、整合、評估）與軌跡評分。
- **外掛系統**：除錯日誌、上下文過濾、多模態結果、儲存產物、BigQuery 分析。
- **規劃器系統**：內建規劃器，採用先規劃後執行模式。
- **Vibe 程式碼支援**：用於 LLM 獲取 ADK 上下文資訊的 `llms.txt` 和 `llms-full.txt`。
- **Skills 系統**：可重用、可打包的代理能力。
- **代理優化**：`agent_optimizer.py` 和 GEPA 提示優化器。

### 相對於 Entelecheia 設計目標的潛在差距

- 對 Google Cloud 的強依賴性（生產功能）。
- Gemini 優化（儘管模型無關）。
- 複雜的程式碼庫，具有多層抽象。
- 僅 FastAPI 用於 API 服務（無替代框架）。
- 無 CLI 原生的代理體驗（以 Web UI 為中心）。
- 記憶/上下文處理是 Gemini 快取感知的，對其他模型較不通泛。

---

## 6. OpenAI Swarm（實驗性，現已廢棄）

**倉庫**：[openai/swarm](https://github.com/openai/swarm)
**語言**：Python
**授權**：MIT
**狀態**：已被 [OpenAI Agents SDK](https://github.com/openai/openai-agents-python) 取代

### 架構

- **極簡原語**：`Agent`（指令 + 函數）和交接（從函數傳回另一個 Agent）。
- **核心循環**（`swarm/core.py` 中約 300 行）：

  1. 從當前 Agent 獲取完成
  1. 執行工具呼叫，附加結果
  1. 如果函數傳回 Agent 則切換 Agent
  1. 更新上下文變數
  1. 重複直到無更多工具呼叫或達到 `max_turns`

- **代理**：僅名稱、模型、指令（字串或可呼叫）、函數列表。無代理階層 — 透過交接進行扁平委派。
- **通訊**：Chat Completions API 訊息。`client.run()` 呼叫之間無狀態。

### 工具暴露

- 工具是普通的 Python 函數。從型別提示和文件字串自動生成模式。
- 分配給 Agent 的所有函數皆暴露給每次 LLM 呼叫。
- 如果函數傳回 `Agent`，則執行轉移（交接）。
- 如果函數簽名中定義了 `context_variables` 參數，則自動填充。

### 安全模型

- **無**。無沙箱化、無隔離、無權限模型。工具在呼叫者的程序中運行。
- 教育/實驗專案，明確不適用於生產環境。

### 記憶/上下文

- **無狀態**：`client.run()` 呼叫之間無狀態。使用者必須傳遞 `messages` 並取回它們。
- **上下文變數**：透過函數呼叫傳遞的簡單字典 — 可被工具函數讀取/寫入。
- 無記憶、無會話持久化、無上下文壓縮。

### 獨特功能

- **極致簡潔**：整個框架約 4 個原始檔。非常適合學習代理編排。
- **交接模式**：優雅 — Agent 僅是「指令 + 工具」，代理透過從工具函數傳回另一個 Agent 來委派。
- **串流支援**：內建串流，使用 `{"delim":"start"}` / `{"delim":"end"}` 標記來標示代理邊界。

### 相對於 Entelecheia 設計目標的潛在差距

- 已廢棄（由 OpenAI Agents SDK 繼承）。
- 完全無安全模型。
- 無狀態/持久化 — 完全無狀態。
- 僅限 OpenAI（Chat Completions API）。
- 除交接外無多代理通訊。
- 實驗/教育性質 — 非生產框架。

---

## 7. Cline

**倉庫**：[cline/cline](https://github.com/cline/cline)
**語言**：TypeScript（VS Code 擴充套件）+ Go（CLI）
**授權**：Apache 2.0

### 架構

- **VS Code 擴充套件** + 獨立 CLI。擴充套件是主要介面；CLI 較新。
- **核心循環**（在 `src/core/` 中）：

  1. 解析使用者任務（文字 + 圖像）
  1. 分析工作區（AST、正則搜尋、檔案讀取）
  1. 在循環中執行工具：檔案建立/編輯、終端命令、瀏覽器動作
  1. 監控輸出（Linter 錯誤、終端輸出、瀏覽器截圖）
  1. 自動修復問題，迭代直到任務完成

- **工具集**：檔案操作（建立、編輯、diff）、終端命令（具備 Shell 整合）、瀏覽器（無頭、點擊/輸入/滾動）、MCP 工具。
- **系統提示**：具備上下文管理指令的詳細提示工程。

### 工具暴露

- 固定的一組內建工具：`read_file`、`write_to_file`、`replace_in_file`、`execute_command`、`browser_action`、`use_mcp_tool` 等。
- **MCP 擴充**：可以按需建立/安裝新的 MCP 伺服器（「新增一個工具來...」）。也支援社群 MCP 伺服器。
- **@-mentions**：`@url`、`@problems`、`@file`、`@folder` 用於上下文注入（降低 API 成本）。
- 每次呼叫的工具數量有限 — 通常 8-12 個內建 + MCP 工具。

### 安全模型

- **所有操作的人機迴圈**：每次檔案變更和終端命令必須由使用者在 GUI 中核准。
- **檢查點系統**：每一步之前的工作區快照。可以隨時 diff/恢復。
- **權限系統**：具備允許/拒絕列表的 `CommandPermissionController`。
- **Cline Ignore**：用於排除敏感檔案的 `.clineignore` 檔案。
- **無沙箱化**：終端命令在使用者的實際環境中執行 — 功能強大但有風險。
- **企業版**：SSO、稽核追蹤、VPC/專用連結、自託管/本地部署。

### 記憶/上下文

- **上下文管理**：專案的 AST 分析、相關檔案的正則搜尋、仔細選擇哪些內容進入上下文視窗。
- **上下文壓縮**：當上下文滿時，生成摘要並壓縮舊對話。
- **檢查點**：用於回滾的完整工作區快照。
- **任務狀態**：`TaskState.ts` 管理對話記憶和工作區狀態。
- **循環偵測**：`loop-detection.ts` 防止無限的工具呼叫循環。

### 獨特功能

- **完整 IDE 整合**：存在於 VS Code 中，可以看到你的整個工作區。
- **瀏覽器自動化**：用於 Web 測試/除錯的 Claude 電腦使用能力。
- **自動修復循環**：監控 Linter/編譯器錯誤並在無使用者介入下自動修復。
- **MCP 工具建立**：可以根據使用者請求即時建立新的 MCP 伺服器。
- **執行時繼續**：非阻塞的終端命令執行。
- **多模型**：OpenRouter、Anthropic、OpenAI、Gemini、Bedrock、Azure、Vertex、Cerebras、Groq、Ollama、LM Studio。
- **Go CLI**：適用於沒有 VS Code 的環境的獨立 CLI 選項。
- **評估框架**：3 層測試（合約、煙霧、E2E/基準）。

### 相對於 Entelecheia 設計目標的潛在差距

- 完整體驗需要綁定 VS Code（CLI 較新且不太成熟）。
- 單代理架構 — 無多代理協作。
- 無定義的代理角色/專業化。
- 每個操作需要人類核准（儘管可自動化）。
- 無互動的長期自主運行。
- 非框架 — 一個應用程式。無法作為程式庫嵌入。
- 僅 TypeScript/Go，無 Python SDK。

---

## 8. Aider

**倉庫**：[Aider-AI/aider](https://github.com/Aider-AI/aider)
**語言**：Python
**授權**：Apache 2.0

### 架構

- **基於終端的結對程式設計**：CLI 工具，在你的倉庫中編輯程式碼。
- **核心循環**（`base_coder.py`）：

  1. 建立倉庫地圖（基於 Tree-sitter AST 的程式碼庫摘要）
  1. 將提示 + 倉庫地圖 + 檔案發送給 LLM
  1. 解析 LLM 回應中的編輯指令（unified diffs、搜尋/替換區塊、整個檔案重寫）
  1. 應用編輯、Lint、執行測試、自動修復失敗
  1. 使用合理的訊息進行 Git 提交

- **多種編輯格式**：`udiff`、`editblock`、`wholefile`、`search_replace`、`diff_fenced`、`editor_editblock`、`editor_whole`、`patch`、`architect`。每個都是具有自己提示範本的獨立編碼器類別。
- **架構師/編輯器模式**：雙代理模式 — 架構師規劃，編輯器實作。在基準測試中證明非常有效。

### 工具暴露

- **無傳統的工具呼叫**。Aider 使用結構化文字回應（非函數呼叫）。
- LLM 以特定格式輸出編輯指令（diff 區塊、搜尋/替換），Aider 解析並套用。
- Shell 命令：LLM 可以請求執行 Shell 命令（使用者確認）。
- Web 擷取：基於 Playwright 的瀏覽器用於閱讀網頁。
- 語音輸入：透過麥克風的語音轉程式碼。
- 圖像：可以將截圖/圖像新增到聊天中以獲取視覺上下文。

### 安全模型

- **無沙箱化**。編輯直接應用到你的檔案。Git 提供安全網。
- **使用者確認**：Shell 命令需要顯式的使用者核准。
- **Git 整合**：所有變更自動提交 — 容易回滾。
- **監控模式**：可以監控檔案變更，並在測試失敗時自動重新套用。
- **Linting/測試**：變更後自動執行配置的 Linter 和測試套件。
- 無權限系統、無隔離、無基於角色的存取。

### 記憶/上下文

- **倉庫地圖**：Tree-sitter AST 分析建立整個程式碼庫的簡潔地圖（函數簽名、類別定義、匯入關係）。這將程式碼庫結構納入上下文，而不包含所有原始碼。
- **聊天歷史**：上下文中的完整對話。
- **檔案選擇**：LLM 透過語法請求特定檔案 — 只有那些檔案的內容被新增到上下文中。
- **上下文視窗管理**：當接近上下文限制時，Aider 丟棄較舊的對話回合並包含摘要。
- **無長期記憶**：會話之間無狀態。每次 `aider` 啟動都是全新的。

### 獨特功能

- **倉庫地圖**：基於 AST 的程式碼庫理解，在程式碼任務上優於基於嵌入的 RAG。
- **多種編輯格式**：適應每個模型最適合的方式（某些模型用 udiff 較好，其他用搜尋/替換等）。
- **架構師/編輯器模式**：規劃和執行的兩個步驟過程，使用獨立的 LLM 呼叫。
- **多語言**：透過 Tree-sitter 支援 100+ 程式語言。
- **LLM 排行榜**：維護跨模型的程式碼編輯公共基準。
- **語音程式碼**：直接在終端中語音轉程式碼。
- **複製貼上模式**：適用於沒有 API 存取的模型的 Web 聊天介面。
- **Git 感知**：自動提交、合理的提交訊息、與現有 git 倉庫協作。
- **自我編寫**：Aider 自身程式碼的 88% 由 Aider 編寫。

### 相對於 Entelecheia 設計目標的潛在差距

- **單檔案焦點**：主要一次編輯一個檔案（儘管倉庫地圖提供上下文）。
- **無多代理系統**：僅架構師/編輯器對。無可自訂的代理角色。
- **無工具生態系統**：無法使用 API、資料庫、Web 服務 — 僅檔案編輯和 Shell。
- **無沙箱化**：直接檔案系統存取 — 功能強大但有風險。
- **無持久狀態**：每個會話獨立。無跨會話學習。
- **無編排原語**：非框架 — 獨立工具。
- **文字解析脆弱性**：依賴 LLM 輸出精確的編輯格式 — 可能因模型偏差而失敗。
- **僅限終端**：在多數工作流程中無法作為程式庫嵌入（儘管有 Python API）。

---

## 比較摘要表

| 維度 | CrewAI | LangGraph | MetaGPT | ChatDev 2.0 | Google ADK | OpenAI Swarm | Cline | Aider |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| **語言** | Python | Python/TS | Python | Python/Vue | Python/Java/Go | Python | TS/Go | Python |
| **框架/程式庫** | 框架 | 框架 | 框架 | 平台 | 框架 | 實驗 | 應用程式 | 應用程式 |
| **架構** | Crews+Flows | StateGraph | 基於 SOP 的角色 | YAML DAG | Runner+Session | 交接循環 | 工具循環 | 編輯解析器 |
| **多代理** | 是（順序/階層） | 是（子圖） | 是（基於角色） | 是（YAML 節點） | 是（sub_agents） | 是（交接） | 否（單一） | 否（單一+架構師） |
| **程式碼沙箱** | 無 | 無 | 無 | 最小 | 是（GKE、Container、Vertex AI） | 無 | 無（檢查點回滾） | 無（git 回滾） |
| **人機迴圈** | 是（任務標記） | 是（中斷） | 否 | 是（工作流程節點） | 是（工具確認） | 否 | 是（每個操作） | 是（Shell 確認） |
| **記憶模型** | 短期 + 可選長期 | 短期 + 長期（Store） | 短期 + 長期 + 工作 | 短期 + mem0 + 檔案 | Session + 跨會話 | 無（無狀態） | 任務狀態 + 檢查點 | 僅聊天歷史 |
| **上下文管理** | 無顯式 | 無顯式 | 訊息過濾 | 可配置視窗 | 自動壓縮 | 無 | AST 分析 + 壓縮 | 倉庫地圖 + 自動丟棄 |
| **工具暴露** | 每代理 | 每節點 | 每角色（動作） | 每 YAML 節點 | 50+ 內建 | 每代理（扁平） | 8-12 固定 | Shell + 編輯格式 |
| **協定支援** | 無 | 無 | 無 | MCP | A2A、MCP、OpenAPI | 無 | MCP | 無 |
| **模型支援** | 多模型 | LangChain 生態系統 | 12+ 提供者 | 可配置 | Gemini 優化、多 | 僅 OpenAI | 10+ 提供者 | 10+ 提供者 |
| **生產就緒** | 是（含 AMP） | 是（LangSmith Deploy） | 有限 | 有限 | 是 | 否 | 是（企業版） | 是 |
| **獨特優勢** | Flows+Crews 雙重性 | 持久執行 | 完整 SDLC 模擬 | 零程式碼視覺化建構器 | A2A 協定、沙箱化執行 | 極簡優雅 | IDE 整合、瀏覽器自動化 | 倉庫地圖、編輯格式 |
| **程式碼規模** | ~50+ 原始檔 | 大 | ~200+ 原始檔 | ~150+ 原始檔 | ~200+ 原始檔 | ~4 原始檔 | ~300+ 原始檔 | ~100 原始檔 |

---

## Entelecheia 路線圖的關鍵收穫

1. **安全/沙箱化**：幾乎所有框架都缺乏沙箱化執行。Google ADK 是值得注意的例外，具有基於容器/Kubernetes 的沙箱化。這是一個重大的差異化機會。

1. **多代理通訊**：只有 Google ADK 具有正式的代理間協定（A2A）。多數框架使用臨時的訊息傳遞。標準化協定（如 A2A）是一個差距。

1. **記憶架構**：多數框架具有基本的短期記憶。很少有具備自動上下文管理的複雜階層式記憶（工作、短期、長期）。MetaGPT 的過濾和 ADK 的壓縮是最佳範例。

1. **工具暴露管理**：所有框架在每次呼叫中都將所有工具暴露給 LLM。沒有框架根據上下文/狀態/安全等級動態子集化工具。這是一個架構差距。

1. **程式碼執行**：只有 ADK 具有生產級沙箱化程式碼執行。ChatDev 有基本的程式碼執行器。Cline/Aider 依賴原生環境。這是整個生態系統中的一個關鍵安全差距。

1. **評估**：ADK 和 Cline 具有正式的評估框架。其他依賴臨時測試或研究基準。嵌入式評估是一個差異化因素。

1. **OpenAI Swarm 的廢棄**為 OpenAI Agents SDK 標誌著一個市場趨勢，朝向生產級框架而非教育實驗。

1. **LangGraph 的持久執行**對於長期運行的代理具有獨特的能力 — 多數框架假設短命任務。

1. **ChatDev 2.0 的零程式碼方法**針對與多數框架根本不同的使用者角色（非開發人員）。這與 Entelecheia 的開發者優先設計是正交的。

1. **Cline 和 Aider** 是應用程式，而非框架。它們展示了緊密工具整合（IDE、git、終端、瀏覽器）的威力，但無法組合成更大的代理系統。

---

## 附錄：Entelecheia 當前狀態提醒

此附錄是上述比較的權威性修正層。

### 目前活躍的內容

- 12 個第一層代理在工作區中編譯。
- 1 個用於 Web 自動化的第二層 crate 活躍中。
- 額外的專業代理以計劃或部分文件形式歸檔，不應被視為已完整交付的執行時期模組。

### 相對成熟的內容

- `packages/scepter`、`packages/shared` 和 `packages/tui`
- 僅執行工具暴露模型
- 基於容器的執行路徑
- 加密的提供者金鑰儲存和 RBAC 相關的認證管道

### 仍為部分的內容

- WebUI 相對於 TUI
- CLI 命令覆蓋率
- 桌面/行動整合（遷移至 [shittim-chest](https://github.com/celestia-island/shittim-chest)）
- RAG 和記憶，目前依賴記憶體內文件、基於雜湊的嵌入和圖形遍歷，而非完全整合的 ONNX + pgvector 堆疊
- 稽核完整性和容器強化
