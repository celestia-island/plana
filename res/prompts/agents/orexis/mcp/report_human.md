+++
name = "report_human"
agent = "orexis"

[description]
en = "Send a direct reply to the user and TERMINATE the skill chain — no further skills run. Use for conversational responses, opinions, chitchat, or any reply that doesn't need downstream processing."
zhs = "直接回复用户并终止技能链——后续技能不再执行。用于对话回复、观点、闲聊等不需要后续处理的场景。"
zht = "直接回覆用戶並終止技能鏈——後續技能不再執行。用於對話回覆、觀點、閒聊等不需要後續處理的場景。"
ja = "ユーザーに直接返信し、スキルチェーンを終了します。後続のスキルは実行されません。会話応答、意見、雑談など、後続処理が不要な返信に使用します。"
ko = "사용자에게 직접 응답하고 스킬 체인을 종료합니다. 추가 스킬이 실행되지 않습니다. 대화 응답, 의견, 잡담 등 후속 처리가 필요 없는 응답에 사용하세요."
fr = "Envoyer une réponse directe à l'utilisateur et TERMINER la chaîne de compétences — aucune autre compétence ne s'exécute. Utiliser pour les réponses conversationnelles, les opinions, les discussions informelles ou toute réponse ne nécessitant pas de traitement supplémentaire."
es = "Enviar una respuesta directa al usuario y TERMINAR la cadena de habilidades — no se ejecutan más habilidades. Usar para respuestas conversacionales, opiniones, charla informal o cualquier respuesta que no necesite procesamiento adicional."
ru = "Отправить прямой ответ пользователю и ЗАВЕРШИТЬ цепочку навыков — последующие навыки не выполняются. Используйте для разговорных ответов, мнений, непринуждённой беседы или любого ответа, не требующего дальнейшей обработки."
+++

# report_human

## Description

Generates a human-readable report, typically used at the end of a skill chain to deliver results to the user. This variant includes compliance-aware formatting for Orexis-mediated workflows.

## Parameters

| Name | Type | Required | Description |
| --- | --- | --- | --- |
| `summary` | string | yes | Short one-line summary of the result |
| `body` | string \| null | no | Detailed multi-line report body (Markdown) |
| `text` | string \| null | no | Plain text fallback for terminals without Markdown rendering |
| `mode` | string \| null | no | Set to `"reply"` to signal reply-mode termination |

## Returns

Returns the tool result as a JSON object with `ok` (boolean) and `data` fields.
