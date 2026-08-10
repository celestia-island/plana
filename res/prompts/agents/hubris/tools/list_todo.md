+++
name = "list_todo"
agent = "hubris"

[description]
en = "Query and display a list of TODO items in multiple view modes. Supports flexible filtering, sorting, and grouping to meet different display needs."
+++

# list_todo

## Description

Query and display a list of TODO items in multiple view modes. Supports flexible filtering, sorting, and grouping to meet different display needs.

## Parameters

- view: View mode: `tree` (tree), `list` (list), `flat` (flat), `kanban` (kanban). Default: `tree`
- status: Filter by status: `pending`, `in_progress`, `completed`, `cancelled`
- priority: Filter by priority: `low`, `medium`, `high`, `critical`
- tags: Filter by tags (supports multiple tags)
- `parent_id`: Specify parent node ID, query subtasks under that node
- depth: Hierarchy depth limit (tree view only). `-1` means unlimited
- `sort_by`: Sort field: `created`, `updated`, `priority`, `due_date`, `title`
- `sort_order`: Sort order: `asc` (ascending), `desc` (descending). Default: `asc`
- search: Search keyword (search in title and description)
- limit: Return result count limit
- offset: Pagination offset

> **Parameter Format**:
> All parameters are passed as a JSON object via native function calling (`tool_calls`). See the tool definition for the JSON Schema.

## Return Results

Returns data in different formats based on view mode:

### Tree View (tree)

```text
Operation successful

view: "tree"
total: 15
items: "[item1, item2]"}]
```

### List View (list)

```text
Operation successful

view: "list"
total: 15
items: "[item1, item2]"
page:
  limit: 50
  offset: 0
  has_more: False
```

### Kanban View (kanban)

```text
Operation successful

view: "kanban"
columns:
  pending: "[example1, example2]"
  in_progress: []
  completed: []
```

## Use Cases

### Task Overview

- View overall project task structure
- Understand task hierarchy relationships
- Visualize task distribution

### Status Monitoring

- Track task completion progress
- Identify blocked tasks
- Monitor overdue tasks

### Report Generation

- Generate task lists
- Create kanban views
- Export task data

### Filter Query

- View urgent tasks by priority
- View by tag category
- Search for specific tasks

## Examples

### Example 1: View all high-priority tasks

```text
Operation successful

view: "list"
priority: "high"
sort_by: "due_date"
sort_order: "asc"
```

### Example 2: View subtask tree for a specific project

```text
Operation successful

view: "tree"
parent_id: "todo_project_123"
depth: 3
```

### Example 3: Generate kanban view

```text
Operation successful

view: "kanban"
tags: "[important, pending]"
status: "[example1, example2]"
```

### Example 4: Search tasks

```text
Operation successful

view: "list"
search: "payment function"
limit: 20
```

### Example 5: View overdue tasks

```text
Operation successful

view: "list"
status: "[example1, example2]"
sort_by: "due_date"
sort_order: "asc"
limit: 10
```

## View Mode Description

| View | Description | Use Cases |
| --- | --- | --- |
| tree | Tree hierarchy structure | View task hierarchy, project structure |
| list | Flat list | Quick browsing, paginated queries |
| flat | Flat list with indentation | Simple hierarchy display |
| kanban | Kanban board grouped by status | Task flow, progress tracking |

## Important Notes

- `depth` parameter is only effective for `tree` view
- Case-insensitive when using `search`
- Multiple filter conditions are AND relationship
- `limit` default value is 50, maximum value is 1000
- Kanban view defaults to sorting by priority

## Container-Aware Return Values

- `claimed_by` contains the badge ID of the responsible container (e.g. "#042")
- Items with `[From #xxx]` prefix are messages from other containers
- Filter by `claimed_by` to see only your container's tasks
