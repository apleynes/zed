# Helix Selection Implementation Testing Guide

## Overview

This guide covers testing the Phase 1.1 implementation of Helix selection manipulation commands in Zed's vim mode. The implementation includes native selection operations that were previously only available through the infogulch/zed-helix-keymap.

## What's Been Implemented

### Core Selection Commands

The following Helix selection commands have been implemented as native Zed actions:

- **CollapseSelection** (`;`) - Collapse selection to cursor position
- **FlipSelections** (`Alt-;`) - Flip selection cursor and anchor ⚠️ *Has issues*
- **MergeSelections** (`Alt--`) - Merge all selections into one
- **MergeConsecutiveSelections** (`Alt-_`) - Merge consecutive/overlapping selections
- **KeepPrimarySelection** (`,`) - Keep only the primary selection
- **RemovePrimarySelection** (`Alt-,`) - Remove the primary selection
- **CopySelectionOnNextLine** (`C`) - Copy selection to next line
- **CopySelectionOnPrevLine** (`Alt-C`) - Copy selection to previous line
- **RotateSelectionsBackward** (`(`) - Move last selection to front
- **RotateSelectionsForward** (`)`) - Move first selection to end

### TODO/Unimplemented Commands

- **SelectRegex** (`s`) - Select all regex matches in selections
- **SplitSelectionOnRegex** (`S`) - Split selections on regex matches
- **AlignSelections** (`&`) - Align selections in columns
- **TrimSelections** (`_`) - Trim whitespace from selections
- **KeepSelections** (`K`) - Keep selections matching regex
- **RemoveSelections** (`Alt-K`) - Remove selections matching regex
- **RotateSelectionContentsBackward** (`Alt-(`) - Rotate text between selections
- **RotateSelectionContentsForward** (`Alt-)`) - Rotate text between selections

## How to Test

### Prerequisites

1. Build Zed with the new implementation:
   ```bash
   cd zed
   cargo build --release
   ```

2. Enable Helix mode by switching to `HelixNormal` mode (this may require additional setup)

### Testing Working Features

#### 1. Collapse Selection (`;`)

1. Create a multi-character selection in helix mode
2. Press `;`
3. **Expected**: Selection collapses to cursor position

**Test Case**:
```
Initial: The qu«ick ˇ»brown
Press: ;
Result:  The quˇick brown
```

#### 2. Merge Selections (`Alt--`)

1. Create multiple selections (e.g., using `Ctrl-D` or similar)
2. Press `Alt--`
3. **Expected**: All selections merge into one spanning selection

#### 3. Merge Consecutive Selections (`Alt-_`)

1. Create multiple overlapping or touching selections
2. Press `Alt-_` 
3. **Expected**: Only consecutive selections merge, others remain separate

#### 4. Keep Primary Selection (`,`)

1. Create multiple selections
2. Press `,`
3. **Expected**: Only the primary (most recently created) selection remains

#### 5. Copy Selection to Next Line (`C`)

1. Create a selection on a line
2. Press `C`
3. **Expected**: Original selection remains, new selection appears on next line at same column positions

**Test Case**:
```
Initial: The qu«ick ˇ»brown
         fox jumps over
Press: C
Result:  The qu«ick ˇ»brown
         fox ju«mps ˇ»over
```

#### 6. Selection Rotation (`(` and `)`)

1. Create multiple selections
2. Press `(` or `)`
3. **Expected**: Selection order changes (primary selection rotates)

### Testing Key Bindings

Verify these key bindings work in `vim_mode == helix_normal` context:

```json
"s": "vim::SelectRegex",              // TODO
"shift-s": "vim::SplitSelectionOnRegex", // TODO  
"alt-minus": "vim::MergeSelections",  // ✅ Working
"alt-_": "vim::MergeConsecutiveSelections", // ✅ Working
"&": "vim::AlignSelections",          // TODO
"_": "vim::TrimSelections",           // TODO
";": "vim::CollapseSelection",        // ✅ Working
"alt-;": "vim::FlipSelections",       // ⚠️ Has issues
",": "vim::KeepPrimarySelection",     // ✅ Working
"alt-,": "vim::RemovePrimarySelection", // ✅ Working
"shift-c": "vim::CopySelectionOnNextLine", // ✅ Working
"alt-c": "vim::CopySelectionOnPrevLine",   // ✅ Working
"(": "vim::RotateSelectionsBackward", // ✅ Working
")": "vim::RotateSelectionsForward",  // ✅ Working
"alt-(": "vim::RotateSelectionContentsBackward", // TODO
"alt-)": "vim::RotateSelectionContentsForward",  // TODO
"shift-k": "vim::KeepSelections",     // TODO
"alt-k": "vim::RemoveSelections",     // TODO
```

## Running Automated Tests

### Test Individual Functions

```bash
# Test collapse selection (should pass)
cargo test --package vim test_collapse_selection

# Test copy selection on next line (should pass) 
cargo test --package vim test_copy_selection_on_next_line

# Test flip selections (currently fails)
cargo test --package vim test_flip_selections
```

### Run All Selection Tests

```bash
cargo test --package vim selection
```

## Known Issues

### 1. FlipSelections Rope Offset Error

**Issue**: `test_flip_selections` fails with rope offset assertion error
**Symptom**: `assertion failed: end_offset >= self.offset`
**Cause**: `swap_head_tail()` may create invalid selection state
**Status**: Needs investigation

### 2. Unimplemented Regex Features

Several commands require regex input prompts which aren't implemented yet:
- `SelectRegex` (`s`)
- `SplitSelectionOnRegex` (`S`) 
- `KeepSelections` (`K`)
- `RemoveSelections` (`Alt-K`)

### 3. Complex Text Manipulation

Advanced features need additional implementation:
- `AlignSelections` - requires buffer manipulation
- `TrimSelections` - requires text analysis
- Selection content rotation - requires text swapping

## Development Notes

### File Structure

- **Core implementation**: `zed/crates/vim/src/selection.rs`
- **Key bindings**: `zed/assets/keymaps/vim.json`
- **Registration**: Added to `zed/crates/vim/src/vim.rs`

### Architecture

- Uses existing `editor.change_selections()` infrastructure
- Follows established vim action registration pattern
- Maintains compatibility with existing vim mode

### Performance Considerations

- Operations work on all selections simultaneously
- Uses efficient anchor/point conversions
- Avoids unnecessary buffer snapshots

## Next Steps

1. **Fix FlipSelections**: Debug rope offset issue
2. **Implement regex prompts**: Add UI for regex input
3. **Add alignment logic**: Implement column alignment
4. **Content rotation**: Implement text swapping between selections
5. **Enhanced testing**: Add more comprehensive test coverage

## Comparison with Existing Keymap

The native implementation should provide:
- Better performance (no `workspace::SendKeystrokes` overhead)
- Proper undo granularity
- More reliable operation
- Foundation for additional Helix features

Users can still use the infogulch/zed-helix-keymap as a fallback for unimplemented features.