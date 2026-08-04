+++
name = "check_wait"
agent = "neikos"

[description]
en = "Check if a wait timer has elapsed"
zh-Hans = "检查等待计时器是否已结束"
zh-Hant = "檢查等待計時器是否已結束"
ja = "待機タイマーが経過したかを確認"
ko = "대기 타이머가 경과했는지 확인"
fr = "Vérifier si un minuteur d'attente est écoulé"
es = "Verificar si un temporizador de espera ha finalizado"
ru = "Проверить, истёк ли таймер ожидания"
+++

# check_wait

Polls a wait handle created by `wait` to determine if the specified duration has elapsed. Returns the handle, whether it is ready, and the remaining seconds.

## Parameters

| Name | Type | Required | Default | Description |
| --- | --- | --- | --- | --- |
| `handle` | string | yes | — | The wait handle UUID returned by `wait` |

## Returns

### On Success (ready)

```json
{
  "handle": "0192a3b4-...",
  "ready": true,
  "remaining_seconds": 0
}
```

### On Success (still waiting)

```json
{
  "handle": "0192a3b4-...",
  "ready": false,
  "remaining_seconds": 23
}
```

### On Failure

Returns `{ ok: false, data: null, error: string }` — e.g. missing handle or unknown handle ID.

## Examples

### Example 1: Check a registered wait

```json
{ "handle": "0192a3b4-5678-..." }
```

## Important Notes

- The handle is automatically removed from the registry once it becomes ready (first `check_wait` that returns `ready: true`).
- Subsequent `check_wait` calls for the same handle after it's been removed will return an error.
