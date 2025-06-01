# Helix to Zed Implementation Analysis

## Executive Summary

After analyzing the Helix codebase, the fundamental insight is that Helix implements a **selection + action** paradigm that is **separate from cursor movement**. Basic cursor movements work exactly like vim, but Helix provides explicit selection operations that create multi-selections for powerful editing operations.

## Corrected Understanding of Helix Behavior

### Fundamental Architecture

**Helix is NOT "always selecting"** - this is a common misunderstanding. Instead:

1. **Normal cursor movement** works exactly like vim - moves cursor without creating selections
2. **Explicit selection operations** create selections that can then be acted upon
3. **Select mode** (`v`) makes movements extend selections (like vim visual mode)
4. **Selection + action** paradigm applies only when selections are explicitly created

### Helix vs Vim Comparison

**Vim**: `action + motion/object` (e.g., `dw` = delete word)
**Helix**: `selection + action` (e.g., `w` selects word, then `d` deletes all selections)

**Key Difference**: Helix separates the selection creation from the action, allowing for:
- Multiple selections created before acting
- Visual feedback of what will be affected
- Reusable selections for multiple operations

### Helix Movement Behavior (from codebase analysis)

```rust
// Basic movements use Movement::Move (like vim)
fn move_char_left(cx: &mut Context) {
    move_impl(cx, move_horizontally, Direction::Backward, Movement::Move)
}

// Select mode movements use Movement::Extend  
fn extend_char_left(cx: &mut Context) {
    move_impl(cx, move_horizontally, Direction::Backward, Movement::Extend)
}
```

### Cursor Position Logic

From `helix-core/src/selection.rs`:

```rust
/// Gets the left-side position of the block cursor.
pub fn cursor(self, text: RopeSlice) -> usize {
    if self.head > self.anchor {
        prev_grapheme_boundary(text, self.head)
    } else {
        self.head
    }
}
```

**Key insight**: Cursor is at the **left edge** of selection, not always at head.

## Helix Mode System

From `helix-view/src/document.rs`:

```rust
pub enum Mode {
    Normal = 0,    // Cursor movements, no selection extension
    Select = 1,    // Movements extend selections 
    Insert = 2,    // Text insertion
}
```

**Mode Switching**:
- `v` enters Select mode (like vim visual)
- `Esc` exits back to Normal mode
- Movements in Select mode use `Movement::Extend`
- Movements in Normal mode use `Movement::Move`

## Implementation Strategy for Zed

### Directory Structure

Create a modular helix subsystem within vim crate:

```
zed/crates/vim/src/
├── helix/
│   ├── mod.rs              # Public interface and registration
│   ├── movement.rs         # Helix-style movement commands (reuse vim motions)
│   ├── selection.rs        # Selection manipulation commands
│   ├── text_objects.rs     # Text object selection
│   ├── match_mode.rs       # Match mode operations (surround, brackets)
│   ├── goto_mode.rs        # Goto mode operations  
│   ├── space_mode.rs       # Space mode operations
│   ├── view_mode.rs        # View mode operations
│   ├── shell.rs           # Shell integration
│   └── search.rs          # Search and regex operations
├── vim.rs                 # Main vim integration
└── ...                    # Existing vim files
```

### Core Implementation Principles

#### 1. Reuse Vim Infrastructure

**What to reuse**:
- All basic motion functions (`move_char_left`, `move_next_word_start`, etc.)
- Text object detection logic
- Search and regex functionality
- Most of the editor interaction patterns

**What to modify**:
- Add `Movement::Extend` variants for selection mode
- Create explicit selection manipulation operations
- Add helix-specific minor mode system

#### 2. Separate Cursor Movement from Selection

```rust
// Normal helix movement (reuses vim motion)
fn helix_move_char_left(vim: &mut Vim, window: &mut Window, cx: &mut Context<Vim>) {
    // Use existing vim motion but without selection extension
    vim_move_char_left(vim, window, cx); // Reuse vim implementation
}

// Helix select mode movement (extends selection)
fn helix_extend_char_left(vim: &mut Vim, window: &mut Window, cx: &mut Context<Vim>) {
    vim.update_editor(window, cx, |_, editor, window, cx| {
        editor.change_selections(Some(Autoscroll::fit()), window, cx, |s| {
            s.move_with(|map, selection| {
                // Extend selection left
                let new_pos = movement::move_left(map, selection.head());
                selection.set_head(new_pos);
            });
        });
    });
}
```

#### 3. Selection Manipulation Operations

```rust
// Explicit selection operations (separate from movement)
actions!(helix, [
    SelectWord,              // w - select current word
    SelectRegex,             // s - select regex matches
    SplitSelectionOnRegex,   // S - split selections on regex
    CollapseSelection,       // ; - collapse to cursor
    FlipSelections,          // Alt-; - flip selection direction
    MergeSelections,         // Alt-- - merge all selections
    AlignSelections,         // & - align selections
    // ... more selection operations
]);
```

#### 4. Minor Mode System (Sub-keymaps)

Instead of persistent modal states, use nested keymap objects:

```json
{
  "context": "vim_mode == helix_normal && !menu",
  "bindings": {
    "m": {
      "m": "helix::MatchBrackets",
      "s": "helix::SurroundAdd",
      "r": "helix::SurroundReplace",
      "d": "helix::SurroundDelete"
    },
    "g": {
      "g": "helix::GotoFileStart", 
      "e": "helix::GotoFileEnd",
      "d": "helix::GotoDefinition"
    },
    "space": {
      "f": "helix::FilePicker",
      "b": "helix::BufferPicker"
    }
  }
}
```

### Key Implementation Changes Needed

#### 1. Fix Current Zed HelixNormal Mode

**Problem**: Current implementation always extends selections during movement
**Solution**: Make basic movements work like vim (cursor-only), reserve selection extension for explicit operations

#### 2. Add Selection Mode

```rust
// Add to vim/src/state.rs Mode enum
pub enum Mode {
    Normal,
    Insert, 
    Replace,
    Visual,
    VisualLine,
    VisualBlock,
    HelixNormal,    // Normal helix mode (cursor movement)
    HelixSelect,    // Helix selection mode (extend selections)
}
```

#### 3. Implement Selection Operations

```rust
// Selection creation (separate from movement)
impl Vim {
    fn select_word(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.update_editor(window, cx, |_, editor, window, cx| {
            editor.change_selections(Some(Autoscroll::fit()), window, cx, |s| {
                s.move_with(|map, selection| {
                    // Find word boundaries and create selection
                    let (start, end) = find_word_boundaries(map, selection.head());
                    selection.set_anchor(start);
                    selection.set_head(end);
                });
            });
        });
    }
}
```

### Helix Features Analysis from Codebase

#### 1. Selection System

**Core pattern from helix**:
```rust
// Transform selections
let selection = doc.selection(view.id).clone().transform(|range| {
    // Modify range logic here
    new_range
});
doc.set_selection(view.id, selection);
```

**Selection manipulation operations**:
- `select_regex` - find all regex matches in selections
- `split_selection` - split selections on regex/newlines
- `merge_selections` - combine selections
- `collapse_selection` - collapse to cursor position
- `flip_selections` - swap anchor and head

#### 2. Shell Integration

```rust
fn shell_pipe(cx: &mut Context) {
    shell_prompt(cx, "pipe:".into(), ShellBehavior::Replace);
}
```

**Pattern**: Prompt for command, execute on each selection, replace/insert/append results

#### 3. Text Objects and Matching

```rust
fn match_brackets(cx: &mut Context) {
    let selection = doc.selection(view.id).clone().transform(|range| {
        let pos = range.cursor(text_slice);
        if let Some(matched_pos) = find_matching_bracket(syntax, text, pos) {
            Range::new(range.anchor, matched_pos)
        } else {
            range
        }
    });
    doc.set_selection(view.id, selection);
}
```

### Migration Strategy

#### Phase 1: Fix Basic Movement (1-2 weeks)
1. Modify current HelixNormal mode to use cursor movement only
2. Add HelixSelect mode for selection extension
3. Fix existing helix motion implementations
4. Add proper mode switching (`v` for select mode)

#### Phase 2: Selection Operations (2-3 weeks)  
1. Implement core selection manipulation commands
2. Add regex-based selection operations
3. Create alignment and filtering operations
4. Add content rotation and advanced operations

#### Phase 3: Minor Modes (1-2 weeks)
1. Implement sub-keymap system for goto/space/match modes
2. Add text object selection operations
3. Implement bracket matching and surrounds
4. Add tree-sitter based selections

#### Phase 4: Shell Integration (2-3 weeks)
1. Implement safe shell command execution
2. Add selection piping operations
3. Create filtering and transformation commands
4. Add progress indicators and error handling

#### Phase 5: Polish and Optimization (1-2 weeks)
1. Performance optimization for many selections
2. Better error handling and user feedback
3. Comprehensive testing and documentation
4. Integration with existing vim features

### Success Metrics

1. **Vim compatibility**: No regressions in existing vim functionality
2. **Movement behavior**: Basic movements work like vim (cursor only)
3. **Selection operations**: Explicit selection creation and manipulation
4. **Performance**: Efficient handling of multiple selections
5. **User experience**: Smooth transition between cursor and selection operations

## Conclusion

The key insight is that Helix's power comes from **separating selection creation from actions**, not from "always selecting". Basic movements should work exactly like vim, but Helix provides powerful explicit selection operations that enable multi-cursor editing workflows.

This approach allows:
- **Familiar movement**: vim users feel at home with basic navigation
- **Powerful selection**: explicit multi-selection creation and manipulation  
- **Visual feedback**: see what will be affected before acting
- **Reusable operations**: create selections once, apply multiple actions

The implementation should focus on reusing vim's solid motion foundation while adding Helix's selection manipulation capabilities as a separate layer.