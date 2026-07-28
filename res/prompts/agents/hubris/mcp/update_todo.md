+++
name = "update_todo"
agent = "hubris"

[description]
en = "Update properties of existing TODO items such as status, priority, description, etc. Supports partial updates - only provide fields that need to be modified."
zhs = "更新现有待办事项的属性，如状态、优先级、描述等。支持部分更新——只需提供需要修改的字段。"
zht = "更新現有待辦事項的屬性，如狀態、優先順序、描述等。支援部分更新——只需提供需要修改的欄位。"
ja = "既存のTODOアイテムのステータス、優先度、説明などのプロパティを更新します。部分更新をサポート——変更が必要なフィールドのみ指定してください。"
ko = "기존 TODO 항목의 상태, 우선순위, 설명 등 속성을 업데이트합니다. 부분 업데이트를 지원합니다 - 수정이 필요한 필드만 제공하면 됩니다."
fr = "Mettre à jour les propriétés des éléments TODO existants tels que le statut, la priorité, la description, etc. Prend en charge les mises à jour partielles - ne fournir que les champs à modifier."
es = "Actualizar propiedades de elementos TODO existentes como estado, prioridad, descripción, etc. Admite actualizaciones parciales: solo proporcione los campos que necesitan ser modificados."
ru = "Обновить свойства существующих элементов TODO, такие как статус, приоритет, описание и т.д. Поддерживается частичное обновление — указывайте только поля, которые необходимо изменить."
+++

# update_todo

## Description

Update properties of existing TODO items such as status, priority, description, etc. Supports partial updates - only provide fields that need to be modified.

## Parameters

- `todo_id`: ID of the TODO item to update
- title: New title
- description: New description
- status: New status: `pending`, `in_progress`, `completed`, `cancelled`
- priority: New priority: `low`, `medium`, `high`, `critical`
- `due_date`: New due date (ISO 8601 format)
- tags: New tag list (completely replaces existing tags)
- metadata: New metadata (merged into existing metadata)
- progress: Completion progress (0-100 percentage)

> **Parameter Format**:
> All parameters are passed as a JSON object via native function calling (`tool_calls`). See the tool definition for the JSON Schema.

## Return Results

Returns the updated TODO item object:

```text
Operation successful

id: "todo_123456"
title: "Updated task title"
description: "Updated description"
parent_id: "todo_789"
priority: "high"
status: "in_progress"
progress: 60
due_date: "2024-12-31T23:59:59Z"
tags: "[important, pending]"
metadata:
  assignee: jane.doe@example.com
created_at: "2024-01-15T10:30:00Z"
updated_at: "2024-01-20T14:45:00Z"
children: []
```

## Use Cases

### Status Synchronization

- Mark task as started (`pending` → `in_progress`)
- Mark task as completed (`in_progress` → `completed`)
- Cancel tasks that are no longer needed (`cancelled`)

### Progress Tracking

- Update task completion percentage
- Record milestone achievements
- Reflect actual work progress

### Priority Adjustment

- Increase priority of urgent tasks
- Decrease priority of non-critical tasks
- Reorder based on business requirements

### Information Updates

- Modify task description to reflect requirement changes
- Adjust due date
- Add or update tags and metadata

## Examples

### Example 1: Update task status

```text
Operation successful

todo_id: "todo_abc123"
status: "in_progress"
progress: 30
```

### Example 2: Adjust priority and due date

```text
Operation successful

todo_id: "todo_xyz789"
priority: "critical"
due_date: "2024-02-28T18:00:00Z"
description: "Urgent fix: Payment gateway integration issue"
```

### Example 3: Update metadata and tags

```text
Operation successful

todo_id: "todo_def456"
tags: "[important, pending]"
metadata:
  assignee: mike.smith@example.com
  reviewer: sarah.jones@example.com
  estimated_hours: 8
```

### Example 4: Complete task

```text
Operation successful

todo_id: "todo_ghi789"
status: "completed"
progress: 100
metadata:
  completed_by: john.doe@example.com
  completed_at: 2024-01-20T16:00:00Z
```

## Important Notes

- Only provide fields that need to be updated; other fields remain unchanged
- `tags` completely replaces existing tags; to add tags, include original tags
- `metadata` merges with existing metadata; same keys will be overwritten
- When updating status to `completed`, it is recommended to also set `progress` to 100
- Update operation automatically updates the `updated_at` timestamp
