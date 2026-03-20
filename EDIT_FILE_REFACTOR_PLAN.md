# Edit File Single-Request Refactor: Implementation Plan

**Date**: 2025-01-XX  
**Status**: Draft / Planning  
**Branch**: Working branch (compilation verified in debug mode)

---

## Executive Summary

The `edit_file` tool currently triggers **two LLM requests** per edit:
1. First request: Model decides to call `edit_file` with path + description
2. Second request: `EditAgent` injects format instructions and model generates actual edits

This is wasteful, particularly for Google models which don't support input caching. This plan documents a refactor to support **single-request edits** while maintaining backward compatibility.

---

## Background & Investigation

### The Double-Request Architecture

When investigating `crates/agent/src/tools/edit_file_tool.rs`, we discovered:

```rust
// Current flow in EditFileTool::run()
let (output, mut events) = if matches!(input.mode, EditFileMode::Edit) {
    edit_agent.edit(buffer.clone(), input.display_description.clone(), &request, cx)
} else {
    edit_agent.overwrite(...)
}
```

The `EditAgent::request()` method (in `edit_agent.rs:646-716`):
1. Strips `ToolUse` from the last Assistant message
2. **Injects a hidden User message** with format instructions (from `edit_file_prompt_xml.hbs`)
3. Makes a second LLM request
4. Parses the response for `<old_text>`/`<new_text>` tags

**Why this exists**: Different models prefer different formats:
- Claude/GPT: XML tags (`<old_text>`/`<new_text>`)
- Google/Gemini: Diff-fenced format (`<<<<<<< SEARCH`)

The 77% → 98% pass rate improvement for Google models justified the complexity.

### Tools Analysis

| Tool | Double-Request? | Pattern |
|------|----------------|---------|
| `edit_file` | ✅ **YES** | Uses `EditAgent` |
| `create_file` | ✅ **YES** | Via `EditAgent::overwrite()` |
| `read_file` | ❌ No | Direct execution |
| `grep` | ❌ No | Direct execution |
| `terminal` | ❌ No | Direct execution |

**Only `EditAgent`-based tools have this issue.**

### Format Selection Logic

```rust
// edit_parser.rs:88-96
pub fn from_model(model: Arc<dyn LanguageModel>) -> anyhow::Result<Self> {
    if model.provider_id().0 == "google" || model.id().0.to_lowercase().contains("gemini") {
        Ok(EditFormat::DiffFenced)  // Google gets diff-fenced
    } else {
        Ok(EditFormat::XmlTags)      // Everyone else gets XML
    }
}
```

**`ZED_EDIT_FORMAT` environment variable exists but is unused** (marked `#[allow(dead_code)]`).

### StreamingEditFileTool

There is already a `StreamingEditFileTool` that avoids the double-request by having the model stream edits directly in the tool call. However:
- It's behind a feature flag (`agent-stream-edits`)
- The flag is **staff-only** (`enabled_for_staff() -> true`)
- Not exposed to general users

---

## Options Considered

### Option 1: Wire Up `ZED_EDIT_FORMAT` (2 lines)
**Change**: Call `from_env()` instead of `from_model()`  
**Pros**: Minimal code change  
**Cons**: 
- Still double-request (just uses XML instead of diff-fenced)
- Env var is global, not per-project
- Doesn't actually solve the core problem

**Verdict**: ❌ Rejected - doesn't achieve single-request

### Option 2: Use `StreamingEditFileTool` as Default
**Change**: Remove feature flag, make streaming tool the default  
**Pros**: 
- Already implemented
- Truly single-request
- No conversation manipulation

**Cons**:
- Large behavior change
- May not handle all edge cases yet
- Requires model streaming support

**Verdict**: ❌ Rejected - too risky, needs more validation

### Option 3: Expose Format in Tool Description (Chosen Approach)
**Change**: 
1. Add format instructions to tool description
2. Add `edits` field to tool input
3. Parse edits directly if provided
4. Fall back to `EditAgent` if not provided

**Pros**:
- **Truly single-request** when model provides edits
- **Backward compatible** - falls back to `EditAgent` when needed
- **Explicit** - model knows format upfront
- **Minimal risk** - opt-in behavior via new field

**Cons**:
- More code changes than Option 1
- Requires updating tool schema

**Verdict**: ✅ **Chosen** - best balance of improvement and safety

### Option 4: Multiple Tool Variants
**Change**: Expose `edit_file_xml` and `edit_file_diff` as separate tools  
**Pros**: Model chooses appropriate tool  
**Cons**: Confusing, pollutes tool namespace, model might choose wrong one

**Verdict**: ❌ Rejected - too complex

---

## Decision & Rationale

We chose **Option 3** because it:

1. **Solves the core problem**: Single-request when model provides edits
2. **Maintains safety**: Falls back to proven `EditAgent` behavior
3. **Is incremental**: Doesn't require massive refactoring
4. **Enables future work**: Once validated, can become the default

The approach treats `EditAgent` as a **polyfill** for models/agents that don't yet support direct edit provision, while enabling modern models to provide edits directly.

---

## Detailed Implementation Plan

### Phase 1: Force XML Format

**File**: `crates/agent/src/edit_agent/edit_parser.rs`

**Change 1.1**: Force XML in `from_model()`
```rust
pub fn from_model(_model: Arc<dyn LanguageModel>) -> anyhow::Result<Self> {
    // Always use XML for consistency
    Ok(EditFormat::XmlTags)
}
```

**Rationale**: XML is more widely supported than diff-fenced. We can make this configurable later via `ZED_EDIT_FORMAT` if needed.

**Change 1.2** (Optional): Enable `from_env()`
```rust
// Remove #[allow(dead_code)]
pub fn from_env(model: Arc<dyn LanguageModel>) -> anyhow::Result<Self> {
    let default = EditFormat::XmlTags;
    std::env::var("ZED_EDIT_FORMAT")
        .map_or(Ok(default), |s| EditFormat::from_str(&s))
}
```

### Phase 2: Extend Tool Input

**File**: `crates/agent/src/tools/edit_file_tool.rs`

**Change 2.1**: Add `edits` field to input struct
```rust
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct EditFileToolInput {
    pub display_description: String,
    pub path: PathBuf,
    pub mode: EditFileMode,
    /// XML-formatted edits. If provided, applies directly without EditAgent.
    #[serde(default)]
    pub edits: Option<String>,
}
```

**Change 2.2**: Add to partial input (for streaming)
```rust
struct EditFileToolPartialInput {
    // ... existing fields ...
    #[serde(default)]
    pub edits: Option<String>,
}
```

### Phase 3: Update Tool Description

**File**: `crates/agent/src/tools/edit_file_tool.rs`

Add format instructions to the doc comment:

```rust
/// This is a tool for creating a new file or editing an existing file.
///
/// BEFORE USING THIS TOOL:
/// 1. Use `read_file` to understand the file's contents
/// 2. Verify the directory exists (for new files) with `list_directory`
///
/// EDIT FORMAT:
/// When editing (mode="edit"), provide edits in this XML format:
///
/// ```xml
/// <edits>
/// <old_text line=10>
/// OLD TEXT HERE (exactly match existing content, including indentation)
/// </old_text>
/// <new_text>
/// NEW TEXT HERE (replacement content)
/// </new_text>
/// </edits>
/// ```
///
/// - Include line= attribute on <old_text> (1-based line number)
/// - Content must exactly match the file (including indentation)
/// - For multiple edits, repeat <old_text>/<new_text> pairs
/// - Edits are sequential; each assumes previous edits are applied
///
/// CREATE/OVERWRITE:
/// For mode="create" or mode="overwrite", provide full file content in `edits`.
```

### Phase 4: Modify Tool Execution

**File**: `crates/agent/src/tools/edit_file_tool.rs`

**Change 4.1**: Update `run()` method
```rust
fn run(...) -> Task<Result<...>> {
    cx.spawn(async move |cx| {
        let input = input.recv().await?;
        
        // NEW: Direct edit path
        if let Some(edits_str) = &input.edits {
            return apply_edits_directly(input, edits_str, &self, cx).await;
        }
        
        // FALLBACK: Original EditAgent flow
        let edit_format = EditFormat::from_model(model.clone())?;
        let edit_agent = EditAgent::new(...);
        // ... existing code ...
    })
}
```

**Change 4.2**: Add `apply_edits_directly()` helper
```rust
async fn apply_edits_directly(
    input: EditFileToolInput,
    edits_str: &str,
    tool: &EditFileTool,
    cx: &mut AsyncApp,
) -> Result<EditFileToolOutput, EditFileToolOutput> {
    // 1. Resolve path and open buffer
    // 2. Parse edits_str using EditParser
    // 3. Apply edits to buffer
    // 4. Save and return success
}
```

**Key implementation details**:
- Reuse existing `EditParser` for parsing XML
- Reuse existing buffer editing logic from `EditAgent`
- Handle errors gracefully (return `EditFileToolOutput::Error`)

### Phase 5: Testing Strategy

**Unit Tests**:
- Parse valid XML edits
- Parse invalid XML (expect error)
- Apply single edit
- Apply multiple edits sequentially
- Empty edits handling

**Integration Tests**:
- Direct edit path (with `edits` field)
- Fallback path (without `edits` field)
- Error propagation

**Manual Tests**:
- Test with Claude (XML format)
- Test with Google models (now forced to XML)
- Verify backward compatibility

---

## Risks & Mitigations

| Risk | Mitigation |
|------|-----------|
| XML format fails for Google models | Keep fallback to `EditAgent` |
| Malformed XML from model | Robust error handling, descriptive messages |
| Breaking existing behavior | `edits` is optional, fallback preserves behavior |
| Performance regression | Direct path skips LLM call (should be faster) |

---

## Success Criteria

1. ✅ Single-request edits work when `edits` field provided
2. ✅ Backward compatible: existing flows still work
3. ✅ All existing tests pass
4. ✅ New tests cover direct edit path
5. ✅ Documentation updated

---

## Future Work

Once this is validated:
1. Make direct edits the default (require `edits` field)
2. Deprecate `EditAgent` path
3. Remove double-request entirely
4. Consider re-enabling `ZED_EDIT_FORMAT` for format selection

---

## References

- Original double-request introduced: PR #29733 (StreamingEditFileTool)
- Diff-fenced format added: PR #32737 (for Google models)
- Deadlock fix: PR #47232 (nested request rate limiting)
- Key files:
  - `crates/agent/src/tools/edit_file_tool.rs`
  - `crates/agent/src/edit_agent.rs`
  - `crates/agent/src/edit_agent/edit_parser.rs`
  - `crates/agent/src/templates/edit_file_prompt_xml.hbs`
