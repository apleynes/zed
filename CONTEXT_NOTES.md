# Context Notes: Helix Implementation in Zed

## Project Overview

**Goal**: Implement native Helix editor features in Zed's vim mode
**Status**: Comprehensive user keymap exists (infogulch/zed-helix-keymap), need native implementation
**Approach**: Build incrementally on existing infrastructure, not from scratch

## Helix Editor Core Concepts

### Philosophy Differences from Vim
- **Vim**: `action + motion/object` (e.g., `dw` = delete word)
- **Helix**: `selection + action` (e.g., `wd` = select word, then delete)
- **Selection-first editing**: All operations start with selections
- **Multiple selections as default**: Cursors are 1-character selections
- **Tree-sitter integration**: Syntax-aware selection and navigation
- **No plugin system**: Built-in features only

### Key Helix Features
1. Selection manipulation (select, split, merge, align)
2. Multiple selection editing
3. Tree-sitter syntax node navigation
4. Minor modes (goto, space, match, view, window)
5. Shell integration (pipe selections through commands)
6. Search with multiple selections
7. Advanced text objects

## Zed's Vim Infrastructure

### Core Files and Architecture
```
zed/crates/vim/src/
├── vim.rs           # Main Vim struct, mode management, settings
├── state.rs         # Mode enum, operators, registers, global state
├── motion.rs        # Motion system, movement commands
├── normal.rs        # Normal mode commands and registration
├── visual.rs        # Visual selection handling
├── object.rs        # Text objects (word, paragraph, brackets, etc.)
├── helix.rs         # Basic helix mode implementation (minimal)
├── normal/          # Normal mode command implementations
│   ├── search.rs    # Search functionality
│   ├── delete.rs    # Delete operations
│   ├── yank.rs      # Yank operations
│   └── ...
└── ...
```

### Key Structs and Enums
```rust
// From state.rs
pub enum Mode {
    Normal, Insert, Replace, Visual, VisualLine, VisualBlock, HelixNormal
}

// From vim.rs
pub(crate) struct Vim {
    pub(crate) mode: Mode,
    operator_stack: Vec<Operator>,
    pub(crate) replacements: HashMap<String, String>,
    // ... more fields
}
```

### Action Pattern
```rust
// Actions are defined using macros
actions!(vim, [ActionName1, ActionName2]);

// Registration in vim mode
Vim::action(editor, cx, |vim, action: &ActionName, window, cx| {
    // Implementation
});

// Keymap binding in vim.json
"key": "vim::ActionName"
```

## Existing Capabilities

### What Already Works (via infogulch keymap)
- **Complete Helix keymap**: 90%+ functionality via `workspace::SendKeystrokes`
- **Selection operations**: Using existing editor selection system
- **Tree-sitter navigation**: `[ x` and `] x` for syntax nodes
- **Minor modes**: Implemented via key sequences
- **Multi-cursor operations**: Using built-in editor features

### Native Zed Selection Commands
```rust
// From editor/src/actions.rs
SelectNext, SelectPrevious, SelectAll, SelectLine
SelectLargerSyntaxNode, SelectSmallerSyntaxNode  
SplitSelectionIntoLines
AddSelectionAbove, AddSelectionBelow
SelectAllMatches
// Many more selection commands available
```

### Current HelixNormal Implementation
- Basic mode in `state.rs`
- Minimal `helix.rs` with word motions and `HelixDelete`
- Word motion tests exist
- Selection-expanding motions for some word movements

## What's Missing for Native Implementation

### High Priority
1. **Selection manipulation commands**: 
   - `SelectRegex` (s), `SplitSelectionOnRegex` (S)
   - `AlignSelections` (&), `MergeSelections` (Alt--)
   - `KeepSelections` (K), `RemoveSelections` (Alt-K)

2. **Enhanced helix motions**:
   - Selection-aware versions of all movements
   - Proper word boundary detection for Helix semantics

3. **Selection state management**:
   - `CollapseSelection` (;), `FlipSelections` (Alt-;)
   - `KeepPrimarySelection` (,), `RemovePrimarySelection` (Alt-,)

### Medium Priority
1. **Minor mode system**: Native state management vs keymap sequences
2. **Shell integration**: `ShellPipe` (|), `ShellKeepPipe` ($), etc.
3. **Advanced selection operations**: Join, rotate, filter

### Low Priority
1. **Keymap optimization**: Replace `workspace::SendKeystrokes` with native actions
2. **Documentation and help system**
3. **Enhanced configuration options**

## Technical Implementation Guidelines

### Selection Manipulation Pattern
```rust
// Use editor.change_selections() infrastructure
vim.update_editor(window, cx, |_, editor, window, cx| {
    editor.change_selections(Some(Autoscroll::fit()), window, cx, |s| {
        s.move_with(|map, selection| {
            // Modify selection logic here
        })
    });
});
```

### Mode Management
```rust
// Current mode switching
vim.switch_mode(Mode::HelixNormal, false, window, cx);

// For minor modes, extend Vim struct:
pub(crate) struct Vim {
    // existing fields...
    minor_mode: MinorMode,
    minor_mode_timeout: Option<Instant>,
}
```

### Tree-sitter Integration
- Build on existing `editor::SelectLargerSyntaxNode`
- Use `zed/crates/language` tree-sitter infrastructure
- Extend for sibling/children selection

## Keymap Integration

### Current Helix Context
```json
{
  "context": "vim_mode == helix_normal && !menu",
  "bindings": {
    // Extensive keymap in infogulch implementation
    // Uses workspace::SendKeystrokes for complex operations
  }
}
```

### Performance Issues with Current Keymap
- `workspace::SendKeystrokes` adds artificial delays
- Complex key sequence parsing overhead
- No native undo granularity for complex operations
- Less reliable than direct action calls

## Implementation Priorities

### Phase 1: Core Selection System
1. Create `zed/crates/vim/src/selection.rs`
2. Implement regex-based selection operations
3. Add alignment and filtering commands
4. Enhance existing `helix.rs` motions

### Phase 2: Minor Modes
1. Create `zed/crates/vim/src/minor_modes.rs`
2. Add timeout-based mode management
3. Implement proper state tracking

### Phase 3: Shell Integration
1. Create `zed/crates/vim/src/shell.rs`
2. Use Zed's process spawning infrastructure
3. Add safety measures and error handling

### Phase 4: Optimization
1. Replace keymap sequences with native actions
2. Improve performance for many selections
3. Add documentation and help

## Key Design Decisions

### Compatibility Strategy
- Maintain existing vim mode functionality
- Allow coexistence of keymap and native implementations
- Provide migration path for existing users
- Use separate key contexts to avoid conflicts

### Performance Considerations
- Batch operations for multiple selections
- Use efficient iteration patterns
- Add progress indicators for long operations
- Implement selection deduplication

### Security for Shell Integration
- Command allowlisting system
- User confirmation for dangerous operations
- Clear feedback about command execution
- Sandbox where possible

## Testing Strategy

### Existing Tests
- Basic helix word motion tests in `helix.rs`
- Extensive vim mode test suite in various files
- Selection manipulation tests in editor

### Required New Tests
- Selection regex operations
- Multi-selection manipulation
- Shell command integration
- Minor mode state management
- Performance tests for large selection counts

## Files to Modify/Create

### New Files
- `zed/crates/vim/src/selection.rs` - Selection manipulation commands
- `zed/crates/vim/src/minor_modes.rs` - Minor mode system
- `zed/crates/vim/src/shell.rs` - Shell integration

### Files to Extend
- `zed/crates/vim/src/helix.rs` - Enhance existing motions
- `zed/crates/vim/src/vim.rs` - Add minor mode state, settings
- `zed/crates/vim/src/state.rs` - Extend mode system if needed
- `zed/assets/keymaps/vim.json` - Add native action bindings

### Integration Points
- `zed/crates/editor` - Selection system, tree-sitter integration
- `zed/crates/language` - Tree-sitter queries and parsing
- `zed/crates/terminal` - Process spawning for shell commands