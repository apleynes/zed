# Plan for Incorporating Full Helix Featureset into Zed

## Executive Summary

Helix is a "post-modern" modal editor inspired by Kakoune that follows a **selection-first editing model** where selections are made before actions, opposite to vim's action-first model. Research shows that a comprehensive user-verified Helix keymap already exists for Zed (infogulch/zed-helix-keymap), proving most Helix functionality can be achieved with existing Zed infrastructure. This updated plan focuses on making Helix features native and robust rather than rebuilding from scratch.

## Core Philosophy Differences

### Vim vs Helix Mental Model
- **Vim**: `action + motion/object` (e.g., `dw` = delete word)
- **Helix**: `selection + action` (e.g., `wd` = select word, then delete)

### Key Helix Characteristics
1. **Selection-first editing**: All operations start with selections
2. **Multiple selections as default**: Cursors are 1-character selections  
3. **Visual feedback**: See selections before acting
4. **Tree-sitter integration**: Syntax-aware selection and navigation
5. **Built-in features**: No plugin system needed

## Current State Analysis

### What Already Works (via infogulch keymap)
- **Complete Helix keymap**: User-verified implementation covering 90%+ of Helix functionality
- **Selection-first editing**: Using `workspace::SendKeystrokes` to chain commands
- **Multiple selections**: Using existing editor selection system
- **Tree-sitter navigation**: `[ x` and `] x` for syntax node selection
- **Minor modes**: Space, window, goto, and match modes via key sequences
- **Search with selections**: Using existing vim search + editor selection commands
- **Visual selection manipulation**: Line selection, multi-cursor operations

### What Exists in Zed Core
- **Rich selection system**: `SelectLargerSyntaxNode`, `SelectSmallerSyntaxNode`, `SplitSelectionIntoLines`
- **Multi-cursor support**: `AddSelectionAbove`, `AddSelectionBelow`, `SelectAllMatches`
- **Tree-sitter integration**: Syntax-aware selection and navigation
- **Basic `HelixNormal` mode**: Minimal implementation with some word motions
- **Advanced editor operations**: Line manipulation, text objects, folding

### What's Missing for Native Implementation
- **Helix-specific selection manipulation**: Align, merge, filter selections natively
- **Selection-aware motions**: Motions that extend selections instead of moving cursor
- **Native minor mode system**: Proper state management for goto/space/match modes
- **Shell integration**: Pipe selections through external commands
- **Selection regex operations**: Native split/select by regex
- **Improved helix delete**: Current `HelixDelete` is basic

## Implementation Strategy

### Phase 1: Enhanced Selection System (3-4 weeks)

#### 1.1 Selection Manipulation Commands (`zed/crates/vim/src/selection.rs` - new file)

**Priority: HIGH** - Core to Helix philosophy

```rust
actions!(vim, [
    // Native replacements for keymap workarounds
    SelectRegex,              // s - select all regex matches in selections
    SplitSelectionOnRegex,    // S - split selections on regex matches  
    AlignSelections,          // & - align selections in columns
    MergeSelections,          // Alt-- - merge all selections
    MergeConsecutiveSelections, // Alt-_ - merge consecutive selections
    TrimSelections,           // _ - trim whitespace from selections
    
    // Selection filtering (currently missing)
    KeepSelections,           // K - keep selections matching regex
    RemoveSelections,         // Alt-K - remove selections matching regex
    
    // Selection state management
    CollapseSelection,        // ; - collapse to cursor
    FlipSelections,           // Alt-; - flip selection direction
    KeepPrimarySelection,     // , - keep only primary selection
    RemovePrimarySelection,   // Alt-, - remove primary selection
    
    // Multi-cursor enhancements
    CopySelectionOnNextLine,  // C - extend selection to next line
    CopySelectionOnPrevLine,  // Alt-C - extend selection to prev line
    RotateSelections,         // ( ) - rotate primary selection
    RotateSelectionContents,  // Alt-( Alt-) - rotate content between selections
]);
```

**Implementation Approach:**
- Build on existing `editor.change_selections()` infrastructure
- Use Zed's regex engine for pattern matching
- Leverage text layout system for alignment
- Extend selection state tracking in vim mode

#### 1.2 Enhanced Helix Motions (`zed/crates/vim/src/helix.rs` - extend existing)

**Priority: HIGH** - Core movement behavior

- Improve existing `helix_move_cursor()` to properly extend selections
- Add selection-aware versions of all motion commands
- Implement proper word boundary detection for Helix semantics
- Fix edge cases in current helix word motions

### Phase 2: Minor Mode System (2-3 weeks)

#### 2.1 Native Mode Management (`zed/crates/vim/src/minor_modes.rs` - new file)

**Priority: MEDIUM** - Currently handled via keymap but could be more robust

```rust
#[derive(Clone, Debug)]
pub enum MinorMode {
    None,
    Goto,
    Space, 
    Match,
    View,
    Window,
}

actions!(vim, [
    EnterGotoMode,
    EnterSpaceMode,
    EnterMatchMode,
    EnterViewMode,
    EnterWindowMode,
    ExitMinorMode,
]);
```

**Implementation Approach:**
- Add minor mode state to `Vim` struct
- Implement timeout-based mode exit
- Add mode-specific key context handling
- Maintain compatibility with existing keymap approach

#### 2.2 Enhanced Goto/Space Mode Actions

**Priority: LOW** - Mostly working via existing keymap

- Convert keymap sequences to native actions where beneficial
- Add missing LSP integration commands
- Improve picker integration

### Phase 3: Shell and Advanced Features (3-4 weeks)

#### 3.1 Shell Integration (`zed/crates/vim/src/shell.rs` - new file)

**Priority: MEDIUM** - Unique Helix feature not in current keymap

```rust
actions!(vim, [
    ShellPipe,              // | - pipe selections through command
    ShellPipeTo,            // Alt-| - pipe to command, ignore output
    ShellInsertOutput,      // ! - insert command output before selections
    ShellAppendOutput,      // Alt-! - append command output after selections  
    ShellKeepPipe,          // $ - keep selections where command succeeds
]);
```

**Implementation Approach:**
- Use Zed's process spawning infrastructure
- Implement async command execution with progress indicators
- Add proper error handling and user feedback
- Support for streaming large outputs

#### 3.2 Advanced Selection Operations

**Priority: MEDIUM** - Some gaps in current capabilities

- Selection join operations (`J`, `Alt-J`)
- Advanced search with multiple selections
- Selection-aware increment/decrement operations
- Improved text object selection in helix mode

### Phase 4: Integration and Polish (2-3 weeks)

#### 4.1 Native Keymap Integration

**Priority: HIGH** - Remove dependency on workspace::SendKeystrokes

- Convert key sequences to native action bindings
- Improve key context management for helix mode
- Add proper mode indicators and feedback
- Optimize performance for complex selection operations

#### 4.2 Settings and Configuration

**Priority: LOW** - Nice to have improvements

```rust
struct VimSettings {
    // existing fields...
    pub helix_mode_enabled: bool,
    pub helix_selection_timeout_ms: u64,
    pub helix_tree_sitter_enabled: bool,
    pub helix_shell_timeout_ms: u64,
}
```

#### 4.3 Documentation and Discoverability

- Add helix mode documentation
- Implement key hint system for minor modes
- Add helix-specific help commands
- Migration guide from vim to helix mode

## Technical Implementation Details

### Leveraging Existing Infrastructure

**Selection System:**
- Build on `editor.change_selections()` foundation
- Extend `SplitSelectionIntoLines` pattern for regex operations
- Use existing multi-cursor rendering and editing

**Tree-sitter Integration:**
- Enhance existing `SelectLargerSyntaxNode`/`SelectSmallerSyntaxNode` 
- Add sibling selection using tree-sitter queries
- Build textobject system on existing tree-sitter infrastructure

**Command System:**
- Use proven action pattern from existing vim implementation
- Leverage GPUI's action dispatch system
- Maintain compatibility with existing vim commands

### Key Technical Challenges

#### Challenge 1: Selection Model Compatibility
**Problem**: Helix assumes everything is a selection, Zed/vim has cursor concept
**Solution**: 
- Extend selection handling to treat 1-char selections as cursors
- Modify rendering to show appropriate visual feedback
- Maintain backward compatibility with vim mode

#### Challenge 2: Performance with Many Selections
**Problem**: Complex operations on hundreds of selections could be slow
**Solution**:
- Use batch operations for selection manipulation
- Implement selection deduplication and merging
- Add progress indicators for long-running operations
- Leverage Rust's efficient iterator patterns

#### Challenge 3: Shell Integration Security
**Problem**: Arbitrary shell command execution needs safety measures
**Solution**:
- Implement command allowlisting system
- Add user confirmation for dangerous operations
- Sandbox shell execution where possible
- Provide clear feedback about what commands will run

## Migration from Existing Keymap

### Backward Compatibility Strategy
- Existing infogulch keymap continues to work
- New native actions can coexist with keymap sequences
- Gradual migration path for users
- Settings to choose between keymap and native implementation

### Performance Improvements
- Replace `workspace::SendKeystrokes` with direct action calls
- Eliminate artificial key sequence delays
- Reduce keymap parsing overhead
- Native undo/redo granularity for complex operations

## Implementation Priorities

### Must-Have (Phase 1)
1. **Native selection manipulation** - Core to Helix experience
2. **Improved helix motions** - Foundation for all other features
3. **Selection regex operations** - Key differentiator from vim

### Should-Have (Phase 2-3)
1. **Minor mode system** - Better than keymap sequences
2. **Shell integration** - Unique powerful feature
3. **Advanced selection operations** - Complete the feature set

### Nice-to-Have (Phase 4)
1. **Perfect keymap integration** - Polish and performance
2. **Enhanced documentation** - User experience
3. **Advanced configuration** - Power user features

## Success Metrics

1. **Feature Parity**: 100% compatibility with infogulch keymap functionality
2. **Performance**: 2x faster than keymap-based implementation for complex operations
3. **User Experience**: Seamless transition from existing keymap users
4. **Stability**: No regressions in existing vim mode functionality
5. **Adoption**: Positive feedback from Helix community

## Conclusion

This updated plan builds incrementally on the proven foundation of the existing user-verified Helix keymap while adding native implementations of core Helix concepts. Rather than rebuilding everything from scratch, we focus on:

1. **Making existing functionality native and robust**
2. **Adding missing features that can't be achieved via keymaps**
3. **Improving performance and user experience**
4. **Maintaining backward compatibility**

The key insight is that Helix's power lies in its selection model and tree-sitter integration, both of which are already largely achievable in Zed. The implementation focuses on making these features first-class citizens rather than workarounds, while adding the few missing pieces like shell integration and advanced selection manipulation.

This approach minimizes risk while maximizing benefit, ensuring that existing Helix users can transition smoothly while getting the performance and integration benefits of native implementation.