# Manual Testing Notes for Helix Selection Implementation

## Known Issues

### RotateSelectionContents Position Issue ⚠️

**Issue**: RotateSelectionContents* (`Alt-(` and `Alt-)`) rotates the text content between selections but does not move the selection boundaries to follow the rotated content.

**Current Behavior**: 
- Content rotates correctly between selections
- Selection boundaries remain in original positions
- Cursors don't follow the rotated words

**Expected Behavior**:
- Content should rotate between selections
- Selection boundaries should move with the rotated content
- Cursors should follow the words to their new positions

**Status**: ✅ Content rotation working, ⚠️ Selection positioning needs refinement
**Priority**: Medium - functionality works but could be more intuitive
**Notes**: 
- Simple cases (2 selections) work correctly
- Complex cases with multiple selections and buffer edits have boundary calculation issues
- Implementation works but doesn't match full Helix behavior
- Can be improved in future iterations

**Workaround**: Use simple rotation cases or manually adjust selection positions after rotation

