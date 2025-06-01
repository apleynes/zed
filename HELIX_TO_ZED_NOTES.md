# Helix to Zed Implementation Analysis

## Executive Summary

After analyzing the Helix codebase, it's clear that implementing Helix features in Zed requires a fundamental paradigm shift from vim's action+motion model to Helix's selection+action model. The current approach of extending vim mode actions is fundamentally flawed because vim actions are designed to complete operations and return to normal mode, while Helix operations work on existing selections without mode changes.

## Core Architectural Differences

### Vim vs Helix Mental Models

**Vim (Action + Motion/Object)**:
- Commands like `dw` (delete word) combine an action with a motion
- Actions typically change modes or complete operations
- Operators wait for motions/objects to complete the command
- Mode switching is frequent and intentional

**Helix (Selection + Action)**:
- First make selections (e.g., `w` to select word)
- Then apply actions (e.g., `d` to delete selections)
- All operations work on existing selections
- No mode changes during normal editing operations
- Multiple selections are the default state

### Key Implementation Insight

Helix's "match mode" (`m`) is **not a persistent mode** - it's a **sub-keymap** that handles the next keystroke and returns to normal operation. This is implemented using Helix's keymap trie system:

```rust
"m" => { "Match"
    "m" => match_brackets,
    "s" => surround_add,
    "r" => surround_replace,
    "d" => surround_delete,
    "a" => select_textobject_around,
    "i" => select_textobject_inner,
},
```

## Helix Feature Analysis

### 1. Selection Manipulation

**Helix Implementation Patterns**:
- All functions operate on `doc.selection(view.id)` directly
- Use `doc.set_selection(view.id, new_selection)` to update
- Transform selections using `selection.transform(|range| ...)`
- No mode switching - just selection state changes

**Key Functions Analyzed**:

```rust
// Simple selection collapse
fn collapse_selection(cx: &mut Context) {
    let (view, doc) = current!(cx.editor);
    let text = doc.text().slice(..);
    let selection = doc.selection(view.id).clone().transform(|range| {
        let pos = range.cursor(text);
        Range::new(pos, pos)
    });
    doc.set_selection(view.id, selection);
}

// Flip selection direction
fn flip_selections(cx: &mut Context) {
    let (view, doc) = current!(cx.editor);
    let selection = doc.selection(view.id).clone().transform(|range| range.flip());
    doc.set_selection(view.id, selection);
}

// Merge all selections
fn merge_selections(cx: &mut Context) {
    let (view, doc) = current!(cx.editor);
    let selection = doc.selection(view.id).clone().merge_ranges();
    doc.set_selection(view.id, selection);
}
```

### 2. Content Rotation

**Advanced Selection Content Manipulation**:

```rust
fn reorder_selection_contents(cx: &mut Context, strategy: ReorderStrategy) {
    let (view, doc) = current!(cx.editor);
    let text = doc.text().slice(..);
    let selection = doc.selection(view.id);
    
    // Extract content from each selection
    let mut fragments: Vec<_> = selection
        .slices(text)
        .map(|fragment| fragment.chunks().collect())
        .collect();
    
    // Reorder the content
    for chunk in fragments.chunks_mut(group) {
        match strategy {
            ReorderStrategy::RotateForward => chunk.rotate_right(1),
            ReorderStrategy::RotateBackward => chunk.rotate_left(1),
            ReorderStrategy::Reverse => chunk.reverse(),
        };
    }
    
    // Apply changes back to document
    let transaction = Transaction::change(
        doc.text(),
        selection.ranges().iter().zip(fragments)
            .map(|(range, fragment)| (range.from(), range.to(), Some(fragment))),
    );
    doc.apply(&transaction, view.id);
}
```

### 3. Text Objects and Matching

**Match Brackets Implementation**:

```rust
fn match_brackets(cx: &mut Context) {
    let (view, doc) = current!(cx.editor);
    let text = doc.text();
    let text_slice = text.slice(..);
    
    let selection = doc.selection(view.id).clone().transform(|range| {
        let pos = range.cursor(text_slice);
        if let Some(matched_pos) = match_brackets::find_matching_bracket_fuzzy(
            syntax, text.slice(..), pos
        ) {
            Range::new(range.anchor, matched_pos).with_direction(range.direction())
        } else {
            range
        }
    });
    
    doc.set_selection(view.id, selection);
}
```

**Surround Operations**:
- Use `surround::get_surround_pos()` to find surround positions
- Apply `Transaction::change()` to modify text
- Update selections to match the new text structure
- No mode changes, just direct text and selection manipulation

### 4. Shell Integration

**Shell Command Pattern**:

```rust
fn shell_pipe(cx: &mut Context) {
    shell_prompt(cx, "pipe:".into(), ShellBehavior::Replace);
}

fn shell_prompt(cx: &mut Context, prompt: Cow<'static, str>, behavior: ShellBehavior) {
    ui::prompt(cx, prompt, Some('|'), ui::completers::shell, move |cx, input, event| {
        if event != PromptEvent::Validate { return; }
        shell(cx, input, &behavior);
    });
}
```

**Shell Execution**:
- Operates on each selection independently
- Uses async shell execution with proper error handling
- Replaces/inserts/appends based on behavior
- Maintains selection structure after operations

### 5. Regex Selection Operations

**Select Regex Pattern**:

```rust
fn select_regex(cx: &mut Context) {
    ui::regex_prompt(cx, "select:".into(), Some(reg), ui::completers::none, 
        move |cx, regex, event| {
            let (view, doc) = current!(cx.editor);
            let text = doc.text().slice(..);
            
            let selection = doc.selection(view.id);
            let mut new_ranges = Vec::new();
            
            for range in selection.iter() {
                let fragment = range.fragment(text);
                for mat in regex.find_iter(&fragment) {
                    let start = range.from() + mat.start();
                    let end = range.from() + mat.end();
                    new_ranges.push(Range::new(start, end));
                }
            }
            
            if !new_ranges.is_empty() {
                doc.set_selection(view.id, Selection::new(new_ranges, 0));
            }
        }
    );
}
```

## Implementation Strategy for Zed

### 1. Abandon Vim Action Extension Approach

**Problem with Current Approach**:
- Vim actions are designed to complete and return to normal mode
- Trying to preserve `HelixNormal` mode while using vim infrastructure causes conflicts
- Vim's `observe_keystrokes` and mode switching logic interferes

**Solution**:
- Create pure selection-manipulation functions that work directly with Zed's editor
- Bypass vim mode infrastructure entirely for Helix operations
- Use Zed's native selection system without vim action patterns

### 2. Implement Sub-Keymap System

**Instead of Persistent Match Mode**:

```rust
// In vim.json keymap
{
  "context": "vim_mode == helix_normal && !menu",
  "bindings": {
    "m": {
      "m": "helix::MatchBrackets",
      "s": "helix::SurroundAdd", 
      "r": "helix::SurroundReplace",
      "d": "helix::SurroundDelete",
      "a": "helix::TextObjectAround",
      "i": "helix::TextObjectInside"
    }
  }
}
```

**Implementation Pattern**:

```rust
// Direct selection manipulation without vim infrastructure
impl Vim {
    pub fn helix_match_brackets(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.update_editor(window, cx, |_, editor, window, cx| {
            editor.change_selections(Some(Autoscroll::fit()), window, cx, |s| {
                s.move_with(|map, selection| {
                    let pos = selection.head();
                    if let Some(matched_pos) = find_matching_bracket(map, pos) {
                        selection.collapse_to(matched_pos);
                    }
                });
            });
        });
    }
}
```

### 3. Create Helix-Specific Module Structure

**Recommended File Organization**:

```
zed/crates/vim/src/
├── helix/
│   ├── mod.rs              # Public interface and registration
│   ├── selections.rs       # Selection manipulation commands
│   ├── text_objects.rs     # Text object selection
│   ├── surrounds.rs        # Surround operations
│   ├── shell.rs           # Shell integration
│   └── movement.rs        # Helix-style movement commands
└── vim.rs                 # Main vim integration
```

### 4. Direct Editor Integration Pattern

**Core Implementation Pattern**:

```rust
pub mod helix {
    use editor::{Editor, scroll::Autoscroll};
    use gpui::{Window, Context, actions};
    
    actions!(helix, [
        CollapseSelection,
        FlipSelections,
        MergeSelections,
        MatchBrackets,
        SurroundAdd,
        // ... other commands
    ]);
    
    pub fn register(editor: &mut Editor, cx: &mut Context<Vim>) {
        // Register actions that work directly with editor selections
        Vim::action(editor, cx, |vim, _: &CollapseSelection, window, cx| {
            vim.update_editor(window, cx, |_, editor, window, cx| {
                editor.change_selections(Some(Autoscroll::fit()), window, cx, |s| {
                    s.move_with(|map, selection| {
                        let cursor = selection.head();
                        selection.collapse_to(cursor);
                    });
                });
            });
        });
    }
}
```

### 5. Shell Integration Approach

**Safe Shell Execution**:

```rust
pub async fn execute_shell_command(
    command: &str, 
    input: Option<String>
) -> Result<String, anyhow::Error> {
    // Use Zed's existing process spawning infrastructure
    // Add proper sandboxing and security measures
    // Handle streaming for large outputs
}

pub fn shell_pipe(&mut self, window: &mut Window, cx: &mut Context<Self>) {
    // Get selections
    // Prompt for command
    // Execute async and replace selections with output
    // Maintain selection structure
}
```

## Key Recommendations

### 1. Pure Selection Operations
- Implement all Helix commands as pure selection manipulation functions
- Use Zed's `editor.change_selections()` infrastructure directly
- Avoid vim action patterns that expect mode changes

### 2. Sub-Keymap Implementation
- Use nested keymap objects instead of persistent modal states
- Leverage Zed's existing keymap trie system
- No need for timeout or escape handling - keymap naturally handles this

### 3. Bypass Vim Infrastructure
- Don't extend vim actions for Helix functionality
- Create separate `helix::` namespace for commands
- Register actions that work directly with editor state

### 4. Leverage Existing Zed Features
- Use existing selection manipulation infrastructure
- Build on tree-sitter integration for text objects
- Utilize process spawning for shell commands
- Extend regex engine for selection operations

### 5. Performance Considerations
- Batch selection operations for multiple cursors
- Use efficient anchor/point conversions
- Implement proper undo granularity for complex operations
- Add progress indicators for long-running shell commands

## Migration Path

### Phase 1: Core Selection Operations
1. Create `zed/crates/vim/src/helix/` module structure
2. Implement basic selection manipulation (collapse, flip, merge, etc.)
3. Add proper keymap bindings using sub-keymap syntax
4. Test with simple operations to ensure no mode conflicts

### Phase 2: Text Objects and Matching
1. Implement bracket matching using existing vim infrastructure
2. Add text object selection using tree-sitter integration
3. Implement surround operations using Zed's text manipulation
4. Create sub-keymap for match mode operations

### Phase 3: Advanced Features
1. Add regex-based selection operations with UI prompts
2. Implement shell integration with proper security measures
3. Add selection content rotation and advanced manipulations
4. Optimize performance for large numbers of selections

### Phase 4: Integration and Polish
1. Add comprehensive test coverage
2. Implement proper error handling and user feedback
3. Add documentation and help systems
4. Performance optimization and edge case handling

This approach eliminates the fundamental incompatibility between vim's action+motion paradigm and Helix's selection+action paradigm by implementing Helix features as pure selection operations that bypass vim's mode and action infrastructure entirely.