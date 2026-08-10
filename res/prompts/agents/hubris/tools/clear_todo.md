+++
name = "clear_todo"
agent = "hubris"

[description]
en = "Batch clean up or archive completed TODO items. Supports preview mode to view items to be cleaned before execution."
+++

# clear_todo

## Description

Batch clean up or archive completed TODO items. Supports preview mode to view items to be cleaned before execution.

## Parameters

- mode: Cleanup mode: `delete` (delete), `archive` (archive). Default: `archive`
- status: Status to clean. Default: `completed`
- `older_than`: Clean items completed specified number of days ago
- tags: Only clean items containing specified tags
- confirm: Confirm execution. `false` is preview mode. Default: `false`
- `dry_run`: Dry run, do not actually execute. Default: `false`

> **Parameter Format**:
> All parameters are passed as a JSON object via native function calling (`tool_calls`). See the tool definition for the JSON Schema.

## Return Results

### Preview Mode (confirm=false)

```text
Operation successful

preview: true
mode: "archive"
to_process: 23
items: "[item1, item2]"
message: "Preview: Will archive 23 tasks completed more than 7 days ago"
```

### Execution Mode (confirm=true)

```text
Operation successful

preview: false
mode: "archive"
processed: 23
failed: 0
items_processed: "[example1, example2]"
message: "Successfully archived 23 tasks"
execution_time: "1.23s"
```

## Use Cases

### Regular Maintenance

- Clean up completed tasks weekly
- Archive historical task records
- Keep task list tidy

### Storage Optimization

- Delete unnecessary historical data
- Release system resources
- Improve query performance

### Project Wrap-up

- Clean up completed project tasks
- Archive project records
- Prepare for new projects

## Examples

### Example 1: Preview cleanup of tasks completed 30 days ago

```text
Operation successful

mode: "archive"
status: "completed"
older_than: 30
confirm: false
```

### Example 2: Execute cleanup

```text
Operation successful

mode: "delete"
status: "completed"
older_than: 60
confirm: true
```

### Example 3: Clean up completed tasks with specific tags

```text
Operation successful

mode: "archive"
tags: "[important, pending]"
status: "completed"
confirm: true
```

### Example 4: Clean up cancelled tasks

```text
Operation successful

mode: "delete"
status: "cancelled"
older_than: 90
confirm: true
```

## Cleanup Mode Description

| Mode | Description | Impact |
| --- | --- | --- |
| archive | Archive tasks | Tasks moved to archive area, recoverable |
| delete | Permanently delete | Tasks permanently deleted, unrecoverable |

## Important Notes

- **Strongly recommended** to use preview mode (`confirm=false`) first to view items to be cleaned
- Archived tasks can be recovered through special queries
- Deletion operation is permanent and cannot be undone
- `older_than` parameter is based on task's `completed_at` or `updated_at` time
- When cleaning subtasks, parent task status is not affected
- It is recommended to perform cleanup operations regularly (e.g., weekly or monthly)

## Best Practices

1. **Preview first**: Always preview before executing
1. **Regular cleanup**: Establish a regular cleanup schedule
1. **Reasonable archiving**: Prioritize archiving over deletion
1. **Tag filtering**: Use tags to precisely control cleanup scope
1. **Retention period**: Set reasonable retention periods based on business needs
