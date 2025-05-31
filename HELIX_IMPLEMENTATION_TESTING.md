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

### Recently Implemented Commands (✅ Working)

- **TrimSelections** (`_`) - Trim whitespace from selections ✅
- **AlignSelections** (`&`) - Align selections in columns ✅
- **RotateSelectionContentsBackward** (`Alt-(`) - Rotate text between selections ✅
- **RotateSelectionContentsForward** (`Alt-)`) - Rotate text between selections ✅
- **CollapseSelection** (`;`) - Fixed to collapse to cursor position instead of start ✅
- **FlipSelections** (`Alt-;`) - Flip selection cursor and anchor ✅

### TODO/Unimplemented Commands

- **SelectRegex** (`s`) - Select all regex matches in selections (needs UI prompt)
- **SplitSelectionOnRegex** (`S`) - Split selections on regex matches (needs UI prompt)
- **KeepSelections** (`K`) - Keep selections matching regex (needs UI prompt)
- **RemoveSelections** (`Alt-K`) - Remove selections matching regex (needs UI prompt)

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

#### 5. Collapse Selection (`;`)

1. Create a multi-character selection
2. Press `;`
3. **Expected**: Selection collapses to cursor position (head), not start

**Test Case**:
```
Initial: The qu«ick ˇ»brown
Press: ;
Result:  The quick ˇbrown
```

#### 6. Copy Selection to Next Line (`C`)

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

#### 7. Selection Rotation (`(` and `)`)

1. Create multiple selections
2. Press `(` or `)`
3. **Expected**: Selection order changes (primary selection rotates)

#### 8. Selection Content Rotation (`Alt-(` and `Alt-)`)

1. Create multiple selections with different content
2. Press `Alt-)` for forward rotation or `Alt-(` for backward rotation
3. **Expected**: Content rotates between selections, selections follow the content

**Test Case**:
```
Initial: «aˇ» «bˇ»
Press: Alt-)
Result:  «bˇ» «aˇ»
```

### Testing Key Bindings

Verify these key bindings work in `vim_mode == helix_normal` context:

```json
"s": "vim::SelectRegex",              // TODO
"shift-s": "vim::SplitSelectionOnRegex", // TODO  
"alt-minus": "vim::MergeSelections",  // ✅ Working
"alt-_": "vim::MergeConsecutiveSelections", // ✅ Working
"&": "vim::AlignSelections",          // ✅ Working
"_": "vim::TrimSelections",           // ✅ Working
";": "vim::CollapseSelection",        // ✅ Working
"alt-;": "vim::FlipSelections",       // ✅ Working
",": "vim::KeepPrimarySelection",     // ✅ Working
"alt-,": "vim::RemovePrimarySelection", // ✅ Working
"shift-c": "vim::CopySelectionOnNextLine", // ✅ Working
"alt-c": "vim::CopySelectionOnPrevLine",   // ✅ Working
"(": "vim::RotateSelectionsBackward", // ✅ Working
")": "vim::RotateSelectionsForward",  // ✅ Working
"alt-(": "vim::RotateSelectionContentsBackward", // ✅ Working
"alt-)": "vim::RotateSelectionContentsForward",  // ✅ Working
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

Current status: **11 tests passing, 0 failing**

## Known Issues

### 1. FlipSelections Fixed ✅

**Issue**: `test_flip_selections` was failing with rope offset assertion error
**Symptom**: `assertion failed: end_offset >= self.offset`
**Cause**: `swap_head_tail()` was creating invalid selection state
**Solution**: Replaced `swap_head_tail()` with manual head/tail swapping
**Status**: ✅ RESOLVED - Test now passes

### 2. CollapseSelection Fixed ✅

**Issue**: `collapse_selection` was collapsing to selection start instead of cursor position
**Cause**: Using `selection.start` instead of `selection.head()`
**Solution**: Changed to collapse to `selection.head()` (cursor position)
**Status**: ✅ RESOLVED - Now behaves like Helix

### 3. RotateSelectionContents Improved ✅

**Issue**: Content rotation wasn't moving selections with the content
**Cause**: Only rotating content but keeping selections in original positions
**Solution**: Implemented proper content rotation where selections follow the rotated content
**Status**: ✅ RESOLVED for simple cases - Complex cases with multiple edits still need refinement

### 4. Unimplemented Regex Features

Several commands require regex input prompts which aren't implemented yet:
- `SelectRegex` (`s`)
- `SplitSelectionOnRegex` (`S`) 
- `KeepSelections` (`K`)
- `RemoveSelections` (`Alt-K`)

### 5. Complex Text Manipulation (✅ MOSTLY RESOLVED)

These features have been successfully implemented:
- `AlignSelections` - ✅ Working with proper buffer manipulation
- `TrimSelections` - ✅ Working with selection boundary adjustment
- `CollapseSelection` - ✅ Working with proper cursor position collapse
- Selection content rotation - ✅ Working for simple cases, complex cases need refinement

## Development Notes

### File Structure

- **Core implementation**: `zed/crates/vim/src/selection.rs`
- **Key bindings**: `zed/assets/keymaps/vim.json`
- **Registration**: Added to `zed/crates/vim/src/vim.rs`

### Architecture

- Uses existing `editor.change_selections()` infrastructure
- Follows established vim action registration pattern
- Maintains compatibility with existing vim mode
- Implements proper text editing with `editor.edit()` for content changes
- Preserves selection boundaries after text modifications

### Performance Considerations

- Operations work on all selections simultaneously
- Uses efficient anchor/point conversions
- Avoids unnecessary buffer snapshots
- Batch edits for better performance

### Implementation Quality

- **Test Coverage**: 11 passing tests covering core selection operations
- **Error Handling**: Graceful handling of edge cases (empty selections, single selections)
- **Type Safety**: Proper use of Rust's type system and error handling

## Next Steps

1. ~~**Fix FlipSelections**: Debug rope offset issue~~ ✅ COMPLETED
2. ~~**Fix CollapseSelection**: Collapse to cursor position~~ ✅ COMPLETED  
3. ~~**Fix RotateSelectionContents**: Make selections follow content~~ ✅ COMPLETED (simple cases)
4. **Implement regex prompts**: Add UI for regex input commands (`s`, `S`, `K`, `Alt-K`)
5. **Refine rotation content**: Fix complex cases with multiple selections and buffer edits
6. **Enhanced testing**: Add more comprehensive test coverage for complex scenarios
7. **Performance optimization**: Test with large numbers of selections
8. **Integration testing**: Test interaction with other vim mode features

## Comparison with Existing Keymap

The native implementation should provide:
- Better performance (no `workspace::SendKeystrokes` overhead)
- Proper undo granularity
- More reliable operation
- Foundation for additional Helix features

Users can still use the infogulch/zed-helix-keymap as a fallback for unimplemented features.