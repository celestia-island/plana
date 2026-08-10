+++
name = "create_todo"
agent = "hubris"


[description]
en = "Create TODO task items with hierarchical relationships."
[[related_tools]]
name = "update_todo"
description = "Update TODO status/attributes"


[[related_tools]]
name = "delete_todo"
description = "Delete the specified TODO item"


[[related_tools]]
name = "list_todo"
description = "Multi-view query of TODOs"


[[related_tools]]
name = "move_todo"
description = "Move TODO item to a new parent node"
+++

# create_todo

Create TODO task items with hierarchical relationships.

## Feature Description

The `create_todo` tool is used to create new task items in the TODO tree. It supports creating subtasks under a specified parent node, and allows setting task properties such as title, description, priority, status, due date, and tags. This tool is the core tool for work plan execution, used to build hierarchical task structures.

## Invocation Methods

### Basic Invocation

```text
create_todo <title>
```

### Full Invocation

```text
create_todo
  title: <task title>
  description: <task description>
  parent_id: <parent TODO item ID>
  priority: <priority>
  status: <status>
  due_date: <due date>
  tags: <tag list>
  metadata: <custom metadata>
```

## Parameter Description

- title: Title of the TODO task item [required]
- description: Detailed description of the TODO task item
- `parent_id`: ID of the parent TODO item. If not specified, it will be created at the root level
- priority: Priority: `low`, `medium`, `high`, `critical`, default is medium
- status: Status: `pending`, `in_progress`, `completed`, default is pending
- `due_date`: Due date (ISO 8601 format)
- tags: Tag list for categorization and filtering, default is []
- metadata: Custom metadata key-value pairs, default is {}

## Return Results

### On Success

```text
TODO item created successfully

ID: todo_123456
Title: <task title>
Description: <task description>
Parent: <parent_id or root level>
Priority: <priority>
Status: <status>
Due Date: <due_date>
Tags: <tags>
Created At: <ISO 8601 timestamp>
Updated At: <ISO 8601 timestamp>
Subtask Count: 0
```

### On Failure

```text
TODO item creation failed

Error: <error message>

Possible causes:
- parent_id references a non-existent TODO item
- title is empty or incorrectly formatted
- priority or status value is not within the allowed range
```

## Use Cases

- **Task Planning**: Break down large projects into manageable subtasks, create Work Breakdown Structures (WBS)
- **Project Management**: Create milestones and stage goals, organize team task assignments
- **Workflows**: Define standardized workflow steps, establish checklists
- **Progress Tracking**: Build hierarchical task trees, track task status in real-time

## Usage Examples

### Example 1: Create Root Level TODO

Invocation:

```text
create_todo Complete project proposal
  description: Prepare and submit Q2 project proposal document
  priority: high
  due_date: 2024-03-15T18:00:00Z
  tags: project, documentation
```

Return:

```text
TODO item created successfully

ID: todo_20240115103000
Title: Complete project proposal
Description: Prepare and submit Q2 project proposal document
Parent: Root level
Priority: high
Status: pending
Due Date: 2024-03-15T18:00:00Z
Tags: project, documentation
Created At: 2024-01-15T10:30:00Z
Updated At: 2024-01-15T10:30:00Z
Subtask Count: 0
```

Description: Creates a high-priority project proposal task at the root level, with a due date and related tags set.

### Example 2: Create Subtask

Invocation:

```text
create_todo Write technical plan
  description: Detailed technical implementation plan
  parent_id: todo_abc123
  priority: medium
  tags: technical, planning
```

Return:

```text
TODO item created successfully

ID: todo_20240115103500
Title: Write technical plan
Description: Detailed technical implementation plan
Parent: todo_abc123
Priority: medium
Status: pending
Due Date: (not set)
Tags: technical, planning
Created At: 2024-01-15T10:35:00Z
Updated At: 2024-01-15T10:35:00Z
Subtask Count: 0
```

Description: Creates a subtask under the specified parent task, inheriting the parent task's context, used to refine work content.

### Example 3: Create Task with Metadata

Invocation:

```text
create_todo Code review
  description: Review code implementation of new features
  parent_id: todo_xyz789
  priority: high
  metadata:
    assignee: john.doe@example.com
    estimated_hours: 4
    repository: backend-api
```

Return:

```text
TODO item created successfully

ID: todo_20240115104000
Title: Code review
Description: Review code implementation of new features
Parent: todo_xyz789
Priority: high
Status: pending
Due Date: (not set)
Tags: (none)
Metadata:
  assignee: john.doe@example.com
  estimated_hours: 4
  repository: backend-api
Created At: 2024-01-15T10:40:00Z
Updated At: 2024-01-15T10:40:00Z
Subtask Count: 0
```

Description: Creates a task with custom metadata for storing business-related additional information such as assignee and estimated hours.

### Example 4: Create Urgent Task

Invocation:

```text
create_todo Fix production bug
  description: User login functionality is abnormal, needs immediate fix
  priority: critical
  status: in_progress
  tags: bug, production, urgent
  metadata:
    issue_id: ISSUE-1234
    severity: P0
```

Return:

```text
TODO item created successfully

ID: todo_20240115104500
Title: Fix production bug
Description: User login functionality is abnormal, needs immediate fix
Parent: Root level
Priority: critical
Status: in_progress
Due Date: (not set)
Tags: bug, production, urgent
Metadata:
  issue_id: ISSUE-1234
  severity: P0
Created At: 2024-01-15T10:45:00Z
Updated At: 2024-01-15T10:45:00Z
Subtask Count: 0
```

Description: Creates an urgent task, directly set to `in_progress` status indicating it has already started processing, and records the related Issue ID and severity.

## Important Notes

- **Parent Node Validation**: `parent_id` must reference an existing TODO item, otherwise creation will fail
- **Status Consistency**: Creating subtasks does not automatically update the parent task's status; `update_todo` must be called manually
- **Description Recommendation**: It is recommended to add detailed `description` and `tags` for complex tasks to facilitate future management and queries
- **Metadata Usage**: Use `metadata` to store custom information related to business, such as assignee, estimated hours, associated Issue, etc.
- **Priority Selection**: Choose the appropriate priority based on the urgency and importance of the task; avoid abusing `critical`
- **Tag Convention**: Use a unified tag naming convention to facilitate categorization and filtering

## Container-Aware Usage

- The `claimed_by` field can be auto-filled with your container's badge (e.g. "#042")
- To create a TODO for another container, use `deliver_message()` instead
- Items with `[From #xxx]` prefix in the title are messages from other containers
