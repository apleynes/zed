# Edit File Agent Hijacking Bug

## Discovery

**Date**: 2025-01-XX  
**Discovered By**: User observation during agent testing  
**Component**: `EditAgent::edit` → `apply_edit_chunks` → `EditParser`

## The Bug

When an AI agent (like Claude) calls the `edit_file` tool and includes XML-formatted content (such as `<old_text>...</old_text><new_text>...</new_text>`) in the tool parameters, the content gets **consumed/parsed by the EditParser** instead of being written to the file.

### What Happens

1. Agent calls `edit_file` tool with content containing XML tags
2. Zed routes the tool call through `EditAgent::edit` 
3. `EditAgent::edit` treats the agent's tool call content as an "LLM completion stream"
4. The content is fed through `apply_edit_chunks` → `EditParser`
5. The `EditParser` extracts only the content inside `<new_text>` tags
6. The actual XML the agent intended to write is lost/consumed

### Example

**Agent tries to write this to a file:**
```xml
<old_text>modified content</old_text><new_text>further modified content</new_text>
```

**What actually gets written:**
```
further modified content
```

**What the agent sees in their history:**
- The XML content appears briefly in the UI stream
- Then it "disappears" or gets replaced
- The agent has no record of what happened

## Root Cause

The architecture conflates two different use cases:

| Use Case | Expected Flow | Actual Flow (Bug) |
|----------|--------------|-------------------|
| LLM generates edits | LLM outputs XML → EditParser extracts edits → Apply to file | ✓ Correct |
| Agent writes XML to file | Agent content → Write directly to file | ✗ Content routed through EditParser |

The `EditAgent::edit` pathway was designed for the first use case (LLM generating edits), but it's also being used for the second use case (agent writing file content), causing the agent's content to be incorrectly parsed.

## Impact

- AI agents cannot reliably write XML content to files
- Any file containing `<old_text>` or `<new_text>` tags will be corrupted
- Agents are unaware the hijacking occurred (no error message)
- The UI shows the content briefly then replaces it, confusing the agent

## Solution

The single-request refactor fixes this by:

1. Adding an `edits` field to `EditFileToolInput` that accepts XML directly
2. When `edits` is provided, bypassing `EditAgent::edit` entirely
3. Calling `apply_edit_chunks` directly with the XML as a stream
4. This allows the XML to be parsed as edit instructions (when intended) rather than being hijacked

### Before (Bug)
```
Agent calls edit_file → Zed routes to EditAgent::edit → Content treated as LLM response → EditParser consumes XML
```

### After (Fix)
```
Agent calls edit_file with edits field → Direct call to apply_edit_chunks → XML parsed intentionally → Edits applied
```

## Additional Benefit

This fix also resolves the original double-request problem:

| Issue | Before | After |
|-------|--------|-------|
| Double LLM request | 2 calls per edit | 1 call per edit |
| Agent XML hijacking | Yes | No |
| Performance | Slow (2× latency) | Fast (1× latency) |
| Reliability | Low (hijacking) | High (direct) |

