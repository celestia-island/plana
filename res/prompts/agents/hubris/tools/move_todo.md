+++
name = "move_todo"
agent = "hubris"

[description]
en = "Move TODO items from one parent node to another, reorganizing task hierarchy structure. Supports precise position control."
zh-Hans = "将待办事项从一个父节点移动到另一个父节点，重新组织任务层级结构。支持精确位置控制。"
zh-Hant = "將待辦事項從一個父節點移動到另一個父節點，重新組織任務階層結構。支援精確位置控制。"
ja = "TODOアイテムをある親ノードから別の親ノードに移動し、タスク階層構造を再編成します。正確な位置制御をサポートします。"
ko = "TODO 항목을 한 부모 노드에서 다른 부모 노드로 이동하여 작업 계층 구조를 재편성합니다. 정밀한 위치 제어를 지원합니다."
fr = "Déplacer les éléments TODO d'un nœud parent à un autre, réorganisant la structure hiérarchique des tâches. Prend en charge le contrôle précis de la position."
es = "Mover elementos TODO de un nodo padre a otro, reorganizando la estructura jerárquica de tareas. Admite control preciso de posición."
ru = "Переместить элементы TODO из одного родительского узла в другой, реорганизуя иерархическую структуру задач. Поддерживается точное управление позицией."
+++

# move_todo

## Description

Move TODO items from one parent node to another, reorganizing task hierarchy structure. Supports precise position control.

## Parameters

- `todo_id`: ID of the TODO item to move
- `new_parent_id`: ID of the new parent node. `null` means move to root level
- position: Position under the new parent node. Default: `last`
- `reference_id`: Reference node ID (used for `after` or `before` position)

> **Parameter Format**:
> All parameters are passed as a JSON object via native function calling (`tool_calls`). See the tool definition for the JSON Schema.

### Position Parameter Details

| Value | Description |
| --- | --- |
| `first` | As the first child node |
| `last` | As the last child node |
| `after` | After the reference_id node |
| `before` | Before the reference_id node |

## Return Results

Returns complete information of the TODO item after moving:

```text
Operation successful

success: true
todo:
  id: todo_abc123
  title: Moved task
  parent_id: todo_new_parent
  position: 2
  status: in_progress
  priority: high
  updated_at: 2024-01-20T16:00:00Z
old_parent_id: "todo_old_parent"
new_parent_id: "todo_new_parent"
message: "Task successfully moved to new position"
```

## Use Cases

### Task Reorganization

- Reorganize project structure
- Adjust task categorization
- Optimize task hierarchy

### Task Promotion/Demotion

- Promote subtasks to top-level tasks
- Demote top-level tasks to subtasks
- Adjust task granularity

### Project Migration

- Move tasks from one project to another
- Merge tasks from multiple projects
- Split large projects

### Priority Adjustment

- Move tasks to higher-priority parent nodes
- Adjust task order within the same level

## Examples

### Example 1: Move task to root level

```text
Operation successful

todo_id: "todo_abc123"
new_parent_id: None
```

### Example 2: Move task to new parent node

```text
Operation successful

todo_id: "todo_abc123"
new_parent_id: "todo_xyz789"
position: "last"
```

### Example 3: Precise position control

```text
Operation successful

todo_id: "todo_abc123"
new_parent_id: "todo_xyz789"
position: "after"
reference_id: "todo_def456"
```

### Example 4: Move to first position

```text
Operation successful

todo_id: "todo_abc123"
new_parent_id: "todo_xyz789"
position: "first"
```

## Important Notes

- **Cycle Detection**: Cannot move a task under its own subtask (will form a cycle)
- **Permission Check**: Must have permission to move the task
- **Reference Update**: Move operation automatically updates all related parent-child relationships
- **Status Retention**: Move operation does not change task status, priority, or other attributes
- **Subtask Following**: When moving a task, all its subtasks will move together
- **Position Recalculation**: Positions of tasks at the same level will be automatically recalculated

## Error Handling

| Error Code | Description | Solution |
| --- | --- | --- |
| CIRCULAR_REFERENCE | Cannot move under own subtask | Choose a different target parent node |
| TODO_NOT_FOUND | Task does not exist | Check todo_id |
| PARENT_NOT_FOUND | Parent node does not exist | Check new_parent_id |
| PERMISSION_DENIED | No move permission | Contact administrator |
| INVALID_POSITION | Invalid position parameter | Check position and reference_id |

## Best Practices

1. **Check before moving**: Use `list_todo` to confirm the target position before moving
1. **Avoid frequent moves**: Plan the structure before moving
1. **Batch operations**: When moving multiple tasks, start from the bottommost level
1. **Documentation**: Record reasons for major structure adjustments
