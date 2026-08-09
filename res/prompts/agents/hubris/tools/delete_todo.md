+++
name = "delete_todo"
agent = "hubris"

[description]
en = "Delete the specified TODO item and all its sub-items. Supports cascade deletion to ensure task tree integrity."
zh-Hans = "删除指定的待办事项及其所有子项。支持级联删除以确保任务树的完整性。"
zh-Hant = "刪除指定的待辦事項及其所有子項。支援級聯刪除以確保任務樹的完整性。"
ja = "指定されたTODOアイテムとすべてのサブアイテムを削除します。タスクツリーの整合性を確保するためのカスケード削除をサポートします。"
ko = "지정된 TODO 항목과 모든 하위 항목을 삭제합니다. 작업 트리 무결성을 보장하기 위해 연쇄 삭제를 지원합니다."
fr = "Supprimer l'élément TODO spécifié et tous ses sous-éléments. Prend en charge la suppression en cascade pour garantir l'intégrité de l'arborescence des tâches."
es = "Eliminar el elemento TODO especificado y todos sus subelementos. Admite eliminación en cascada para garantizar la integridad del árbol de tareas."
ru = "Удалить указанный элемент TODO и все его подэлементы. Поддерживается каскадное удаление для обеспечения целостности дерева задач."
+++

# delete_todo

## Description

Delete the specified TODO item and all its sub-items. Supports cascade deletion to ensure task tree integrity.

## Parameters

- `todo_id`: ID of the TODO item to delete
- cascade: Whether to cascade delete all sub-items. Default is `true`
- force: Force delete without prompting for confirmation. Default is `false`

> **Parameter Format**:
> All parameters are passed as a JSON object via native function calling (`tool_calls`). See the tool definition for the JSON Schema.

## Return Results

Returns the result of the deletion operation:

```text
Operation successful

success: true
deleted_id: "todo_abc123"
deleted_count: 5
deleted_items: "[item1, item2]"
message: "Successfully deleted 1 main task and 4 subtasks"
```

## Use Cases

### Task Cleanup

- Delete cancelled or no longer needed tasks
- Clean up incorrectly created duplicate tasks
- Remove outdated task items

### Project Reorganization

- Delete entire task branches
- Clean up test or demo data
- Adjust task structure

### Maintenance Operations

- Batch clean up invalid tasks
- Optimize task tree structure
- Release system resources

## Examples

### Example 1: Delete single task (no subtasks)

```text
Operation successful

todo_id: "todo_xyz789"
cascade: false
```

### Example 2: Cascade delete task and its subtasks

```text
Operation successful

todo_id: "todo_abc123"
cascade: true
```

### Example 3: Force delete (skip confirmation)

```text
Operation successful

todo_id: "todo_def456"
cascade: true
force: true
```

## Important Notes

- **Irrecoverable**: Deletion operation is permanent and cannot be undone
- **Cascade Deletion**: When `cascade=true`, all subtasks and subtasks of subtasks will be deleted
- **Reference Check**: Check if other tasks reference this task before deletion
- **Permission Requirements**: Must have permission to delete the task
- **Recommendation**: Before deleting important tasks, use `list_todo` to confirm the task structure
- **Soft Delete**: Some implementations may support soft delete, moving tasks to a recycle bin rather than permanent deletion

## Error Handling

| Error Code | Description | Solution |
| --- | --- | --- |
| TODO_NOT_FOUND | Specified TODO item does not exist | Check if todo_id is correct |
| HAS_DEPENDENCIES | Task is depended on by other tasks | Delete or modify dependency relationships first |
| PERMISSION_DENIED | No deletion permission | Contact administrator for permission |
