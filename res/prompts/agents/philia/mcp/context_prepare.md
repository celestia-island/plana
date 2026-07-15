+++
name = "context_prepare"
agent = "philia"

[description]
en = "Prepare context for LLM by retrieving relevant memories from the cognitive memory system."
zhs = "通过从认知记忆系统检索相关记忆来为LLM准备上下文。"
zht = "透過從認知記憶系統檢索相關記憶來為LLM準備上下文。"
ja = "認知メモリシステムから関連メモリを取得してLLMのコンテキストを準備します。"
ko = "인지 메모리 시스템에서 관련 메모리를 검색하여 LLM 컨텍스트를 준비합니다."
fr = "Préparez le contexte LLM en récupérant les souvenirs pertinents du système de mémoire cognitive."
es = "Prepare el contexto LLM recuperando recuerdos relevantes del sistema de memoria cognitiva."
ru = "Подготовьте контекст LLM, извлекая релевантные воспоминания из когнитивной системы памяти."
+++

## Parameters

| Name | Type | Required | Description |
| --- | --- | --- | --- |
| `query` | string | yes | Context query |
| `max_nodes` | number | no | Maximum nodes to return (default: 10) |

## Example

```typescript
const result = context_prepare({
  query: 'User preferences and project context',
  max_nodes: 10
});
```
