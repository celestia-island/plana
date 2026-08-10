+++
name = "git_push_branch"
agent = "neikos"

[description]
en = "Push a git branch from within a container using libgit2"
+++

# git_push_branch

Pushes a specified local branch from within a container's Git repository to a remote. Uses the libgit2 library internally, so no system-level Git CLI is required inside the container. Supports configurable remote names and returns the push result including any upstream feedback.

## Parameters

- **`container_id`** (required, string): The unique identifier of the container containing the Git repository.
- **branch** (required, string): The name of the local branch to push (e.g., `"feature/auth"`).
- **remote** (optional, string): The name of the remote to push to. Default: `"origin"`.

## Returns

### On Success

Returns `{ ok: true, data: { container_id: string, branch: string, remote: string, status: string }, error: null }`.

### On Failure

Returns `{ ok: false, data: null, error: string }`.

## Examples

### Example 1: Push a feature branch to origin

```text
container_id: "a1b2c3d4e5f6"
branch: "feature/auth"
```

Returns:

```json
{
  "container_id": "a1b2c3d4e5f6",
  "branch": "feature/auth",
  "remote": "origin",
  "status": "pushed"
}
```

### Example 2: Push to a custom remote

```text
container_id: "a1b2c3d4e5f6"
branch: "hotfix/crash-fix"
remote: "upstream"
```

Returns:

```json
{
  "container_id": "a1b2c3d4e5f6",
  "branch": "hotfix/crash-fix",
  "remote": "upstream",
  "status": "pushed"
}
```

### Example 3: Branch does not exist

```text
container_id: "a1b2c3d4e5f6"
branch: "nonexistent-branch"
```

Returns:

```json
{
  "error": "Branch nonexistent-branch not found in repository"
}
```

## Important Notes

- Uses libgit2 internally — the container does not need Git CLI installed.
- The container must contain an initialized Git repository with the specified branch.
- Authentication credentials must be pre-configured in the container's Git setup (SSH keys, tokens, etc.).
- If the remote rejects the push (e.g., non-fast-forward), the operation returns a failure with the remote's rejection message.
