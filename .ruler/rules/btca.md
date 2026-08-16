# btca (Installed)

Query framework/library source code directly instead of web searches. btca clones repos locally and searches actual implementations - use it for accurate, up-to-date answers about project dependencies.

## Usage

```bash
btca ask -r <resource> -q "<question>"

# Multiple resources:
btca ask -r vue -r electron -q "How do I integrate Vue with Electron?"
```

## Available Resources

See `btca.config.jsonc` in the project root for the full list of configured resources and their source repositories.

To add a new resource, append to the `resources` array in `btca.config.jsonc`:
```jsonc
{
  "name": "resourceName",
  "type": "git",
  "url": "https://github.com/org/repo",
  "branch": "main",
  "searchPath": "optional/subdir",  // optional: limit search to specific path
  "specialNotes": "optional context"  // optional: hints for the AI
}
```

## When to Use

- Framework API questions (prefer over web search)
- "How does X work internally?"
- Verifying correct usage patterns
- Checking current implementation details

## Fallback

If btca fails or lacks a resource, use web search targeting official docs.
