pub mod selections;
pub mod movement;
pub mod mode;
pub mod core;

#[cfg(test)]
mod test;

#[cfg(test)]
mod movement_test;

#[cfg(test)]
mod selection_test;

#[cfg(test)]
mod fix_test;

#[cfg(test)]
mod word_movement_tests;

#[cfg(test)]
mod selection_operation_tests;

#[cfg(test)]
mod find_movement_tests;

#[cfg(test)]
mod core_tests;

use editor::{Editor, scroll::Autoscroll};
use gpui::{Window, Context, actions};
use crate::{Vim, motion::Motion};

actions!(
    helix,
    [
        // Selection manipulation
        CollapseSelection,
        FlipSelections,
        MergeSelections,
        MergeConsecutiveSelections,
        KeepPrimarySelection,
        RemovePrimarySelection,
        TrimSelections,
        AlignSelections,
        
        // Selection rotation
        RotateSelectionsForward,
        RotateSelectionsBackward,
        RotateSelectionContentsForward,
        RotateSelectionContentsBackward,
        
        // Copy selections
        CopySelectionOnNextLine,
        CopySelectionOnPrevLine,
        
        // Match mode operations
        MatchBrackets,
        SurroundAdd,
        SurroundReplace,
        SurroundDelete,
        TextObjectAround,
        TextObjectInside,
    ]
);

pub fn register(editor: &mut Editor, cx: &mut Context<Vim>) {
    // Register movement system
    movement::register(editor, cx);
    
    // Register mode switching
    mode::register(editor, cx);
    
    // Selection manipulation
    Vim::action(editor, cx, helix_collapse_selection);
    Vim::action(editor, cx, helix_flip_selections);
    Vim::action(editor, cx, helix_merge_selections);
    Vim::action(editor, cx, helix_merge_consecutive_selections);
    Vim::action(editor, cx, helix_keep_primary_selection);
    Vim::action(editor, cx, helix_remove_primary_selection);
    Vim::action(editor, cx, helix_trim_selections);
    Vim::action(editor, cx, helix_align_selections);
    
    // Selection rotation
    Vim::action(editor, cx, helix_rotate_selections_forward);
    Vim::action(editor, cx, helix_rotate_selections_backward);
    Vim::action(editor, cx, helix_rotate_selection_contents_forward);
    Vim::action(editor, cx, helix_rotate_selection_contents_backward);
    
    // Copy selections
    Vim::action(editor, cx, helix_copy_selection_on_next_line);
    Vim::action(editor, cx, helix_copy_selection_on_prev_line);
    
    // Match mode operations
    Vim::action(editor, cx, helix_match_brackets);
    Vim::action(editor, cx, helix_surround_add);
    Vim::action(editor, cx, helix_surround_replace);
    Vim::action(editor, cx, helix_surround_delete);
    Vim::action(editor, cx, helix_text_object_around);
    Vim::action(editor, cx, helix_text_object_inside);
}

// Pure selection manipulation functions that work directly with editor state
// These bypass vim infrastructure entirely to avoid mode conflicts

fn helix_collapse_selection(
    vim: &mut Vim,
    _: &CollapseSelection,
    window: &mut Window,
    cx: &mut Context<Vim>,
) {
    vim.update_editor(window, cx, |_, editor, window, cx| {
        editor.change_selections(Some(Autoscroll::fit()), window, cx, |s| {
            s.move_with(|_, selection| {
                let cursor = selection.head();
                selection.collapse_to(cursor, selection.goal);
            });
        });
    });
}

fn helix_flip_selections(
    vim: &mut Vim,
    _: &FlipSelections,
    window: &mut Window,
    cx: &mut Context<Vim>,
) {
    vim.update_editor(window, cx, |_, editor, window, cx| {
        editor.change_selections(Some(Autoscroll::fit()), window, cx, |s| {
            s.move_with(|_, selection| {
                let head = selection.head();
                let tail = selection.tail();
                selection.set_head(tail, selection.goal);
                selection.set_tail(head, selection.goal);
            });
        });
    });
}

fn helix_merge_selections(
    vim: &mut Vim,
    _: &MergeSelections,
    window: &mut Window,
    cx: &mut Context<Vim>,
) {
    vim.update_editor(window, cx, |_, editor, window, cx| {
        let buffer = editor.buffer().read(cx).snapshot(cx);
        let selections = editor.selections.all_adjusted(cx);
        if selections.len() <= 1 {
            return;
        }
        
        let first = selections.first().unwrap();
        let last = selections.last().unwrap();
        let merged_start = first.start.min(last.start);
        let merged_end = first.end.max(last.end);
        
        let start_offset = buffer.point_to_offset(merged_start);
        let end_offset = buffer.point_to_offset(merged_end);
        
        editor.change_selections(Some(Autoscroll::fit()), window, cx, |s| {
            s.select_ranges([start_offset..end_offset]);
        });
    });
}

fn helix_merge_consecutive_selections(
    vim: &mut Vim,
    _: &MergeConsecutiveSelections,
    window: &mut Window,
    cx: &mut Context<Vim>,
) {
    vim.update_editor(window, cx, |_, editor, window, cx| {
        let buffer = editor.buffer().read(cx).snapshot(cx);
        let selections = editor.selections.all_adjusted(cx);
        
        if selections.len() <= 1 {
            return;
        }
        
        let mut merged_ranges = Vec::new();
        let mut current_group = Vec::new();
        
        for selection in selections {
            if current_group.is_empty() {
                current_group.push(selection);
            } else {
                let last_selection = current_group.last().unwrap();
                let last_end_row = last_selection.end.row;
                let current_start_row = selection.start.row;
                
                // Check if this selection is on the next consecutive line
                if current_start_row == last_end_row + 1 || current_start_row == last_end_row {
                    current_group.push(selection);
                } else {
                    // Gap found, finalize current group
                    if current_group.len() > 1 {
                        // Merge the group into one selection
                        let first = current_group.first().unwrap();
                        let last = current_group.last().unwrap();
                        let start_offset = buffer.point_to_offset(first.start);
                        let end_offset = buffer.point_to_offset(last.end);
                        merged_ranges.push(start_offset..end_offset);
                    } else {
                        // Single selection, keep as is
                        let selection = current_group.first().unwrap();
                        let start_offset = buffer.point_to_offset(selection.start);
                        let end_offset = buffer.point_to_offset(selection.end);
                        merged_ranges.push(start_offset..end_offset);
                    }
                    current_group.clear();
                    current_group.push(selection);
                }
            }
        }
        
        // Handle the last group
        if !current_group.is_empty() {
            if current_group.len() > 1 {
                let first = current_group.first().unwrap();
                let last = current_group.last().unwrap();
                let start_offset = buffer.point_to_offset(first.start);
                let end_offset = buffer.point_to_offset(last.end);
                merged_ranges.push(start_offset..end_offset);
            } else {
                let selection = current_group.first().unwrap();
                let start_offset = buffer.point_to_offset(selection.start);
                let end_offset = buffer.point_to_offset(selection.end);
                merged_ranges.push(start_offset..end_offset);
            }
        }
        
        if !merged_ranges.is_empty() {
            editor.change_selections(Some(Autoscroll::fit()), window, cx, |s| {
                s.select_ranges(merged_ranges);
            });
        }
    });
}

fn helix_keep_primary_selection(
    vim: &mut Vim,
    _: &KeepPrimarySelection,
    window: &mut Window,
    cx: &mut Context<Vim>,
) {
    vim.update_editor(window, cx, |_, editor, window, cx| {
        let selections = editor.selections.all_adjusted(cx);
        if let Some(primary) = selections.iter().next() {
            editor.change_selections(Some(Autoscroll::fit()), window, cx, |s| {
                s.select_ranges([primary.range()]);
            });
        }
    });
}

fn helix_remove_primary_selection(
    vim: &mut Vim,
    _: &RemovePrimarySelection,
    window: &mut Window,
    cx: &mut Context<Vim>,
) {
    vim.update_editor(window, cx, |_, editor, window, cx| {
        let selections = editor.selections.all_adjusted(cx);
        if selections.len() <= 1 {
            return; // Can't remove the only selection
        }
        
        let remaining: Vec<_> = selections.iter().skip(1).map(|s| s.range()).collect();
        editor.change_selections(Some(Autoscroll::fit()), window, cx, |s| {
            s.select_ranges(remaining);
        });
    });
}

fn helix_trim_selections(
    vim: &mut Vim,
    _: &TrimSelections,
    window: &mut Window,
    cx: &mut Context<Vim>,
) {
    selections::trim_selections(vim, window, cx);
}

fn helix_align_selections(
    vim: &mut Vim,
    _: &AlignSelections,
    window: &mut Window,
    cx: &mut Context<Vim>,
) {
    selections::align_selections(vim, window, cx);
}

fn helix_rotate_selections_forward(
    vim: &mut Vim,
    _: &RotateSelectionsForward,
    window: &mut Window,
    cx: &mut Context<Vim>,
) {
    selections::rotate_selections(vim, window, cx, true);
}

fn helix_rotate_selections_backward(
    vim: &mut Vim,
    _: &RotateSelectionsBackward,
    window: &mut Window,
    cx: &mut Context<Vim>,
) {
    selections::rotate_selections(vim, window, cx, false);
}

fn helix_rotate_selection_contents_forward(
    vim: &mut Vim,
    _: &RotateSelectionContentsForward,
    window: &mut Window,
    cx: &mut Context<Vim>,
) {
    selections::rotate_selection_contents(vim, window, cx, true);
}

fn helix_rotate_selection_contents_backward(
    vim: &mut Vim,
    _: &RotateSelectionContentsBackward,
    window: &mut Window,
    cx: &mut Context<Vim>,
) {
    selections::rotate_selection_contents(vim, window, cx, false);
}

fn helix_copy_selection_on_next_line(
    vim: &mut Vim,
    _: &CopySelectionOnNextLine,
    window: &mut Window,
    cx: &mut Context<Vim>,
) {
    selections::copy_selection_on_line(vim, window, cx, true);
}

fn helix_copy_selection_on_prev_line(
    vim: &mut Vim,
    _: &CopySelectionOnPrevLine,
    window: &mut Window,
    cx: &mut Context<Vim>,
) {
    selections::copy_selection_on_line(vim, window, cx, false);
}

// Match mode operations - implemented as direct actions without persistent state
fn helix_match_brackets(
    vim: &mut Vim,
    _: &MatchBrackets,
    window: &mut Window,
    cx: &mut Context<Vim>,
) {
    vim.update_editor(window, cx, |_, editor, window, cx| {
        let snapshot = editor.snapshot(window, cx);
        let selections = editor.selections.all_adjusted(cx);
        let mut new_ranges = Vec::new();
        
        for selection in selections {
            let cursor_offset = snapshot.buffer_snapshot.point_to_offset(selection.head());
            
            // Try to find matching bracket using Zed's built-in functionality
            if let Some((opening_range, closing_range)) = snapshot
                .buffer_snapshot
                .innermost_enclosing_bracket_ranges(cursor_offset..cursor_offset + 1, None)
            {
                // Determine which bracket to jump to
                let target_offset = if opening_range.contains(&cursor_offset) {
                    // Cursor is on opening bracket, jump to closing bracket
                    closing_range.start
                } else if closing_range.contains(&cursor_offset) {
                    // Cursor is on closing bracket, jump to opening bracket  
                    opening_range.start
                } else {
                    // Cursor is inside brackets, jump to closing bracket
                    closing_range.start
                };
                
                // In Helix style, just move cursor to the matching bracket position
                new_ranges.push(target_offset..target_offset);
            } else {
                // No matching bracket found, keep original selection
                let start_offset = snapshot.buffer_snapshot.point_to_offset(selection.start);
                let end_offset = snapshot.buffer_snapshot.point_to_offset(selection.end);
                new_ranges.push(start_offset..end_offset);
            }
        }
        
        if !new_ranges.is_empty() {
            editor.change_selections(Some(Autoscroll::fit()), window, cx, |s| {
                s.select_ranges(new_ranges);
            });
        }
    });
}

fn helix_surround_add(
    vim: &mut Vim,
    _: &SurroundAdd,
    window: &mut Window,
    cx: &mut Context<Vim>,
) {
    // Basic implementation: add parentheses around selections for now
    // In the future, this should prompt for the surround character
    vim.update_editor(window, cx, |_, editor, _window, cx| {
        let buffer = editor.buffer().read(cx).snapshot(cx);
        let selections = editor.selections.all_adjusted(cx);
        let mut edits = Vec::new();
        
        for selection in &selections {
            let start_offset = buffer.point_to_offset(selection.start);
            let end_offset = buffer.point_to_offset(selection.end);
            
            // Add opening parenthesis at start
            edits.push((start_offset..start_offset, "(".to_string()));
            // Add closing parenthesis at end
            edits.push((end_offset..end_offset, ")".to_string()));
        }
        
        if !edits.is_empty() {
            editor.edit(edits, cx);
        }
    });
}

fn helix_surround_replace(
    vim: &mut Vim,
    _: &SurroundReplace,
    window: &mut Window,
    cx: &mut Context<Vim>,
) {
    // Basic implementation: find and replace surrounding brackets/quotes
    // For now, just expand selections to include surrounding chars if they exist
    vim.update_editor(window, cx, |_, editor, window, cx| {
        let buffer = editor.buffer().read(cx).snapshot(cx);
        let selections = editor.selections.all_adjusted(cx);
        let mut new_ranges = Vec::new();
        
        for selection in &selections {
            let start_offset = buffer.point_to_offset(selection.start);
            let end_offset = buffer.point_to_offset(selection.end);
            
            // Try to expand selection to include surrounding characters
            let expanded_start = if start_offset > 0 { start_offset - 1 } else { start_offset };
            let expanded_end = if end_offset < buffer.len() { end_offset + 1 } else { end_offset };
            
            new_ranges.push(expanded_start..expanded_end);
        }
        
        if !new_ranges.is_empty() {
            editor.change_selections(Some(Autoscroll::fit()), window, cx, |s| {
                s.select_ranges(new_ranges);
            });
        }
    });
}

fn helix_surround_delete(
    vim: &mut Vim,
    _: &SurroundDelete,
    window: &mut Window,
    cx: &mut Context<Vim>,
) {
    // Basic implementation: remove first and last character from selections
    vim.update_editor(window, cx, |_, editor, _window, cx| {
        let buffer = editor.buffer().read(cx).snapshot(cx);
        let selections = editor.selections.all_adjusted(cx);
        let mut edits = Vec::new();
        
        for selection in &selections {
            let start_offset = buffer.point_to_offset(selection.start);
            let end_offset = buffer.point_to_offset(selection.end);
            
            // Only process if selection is at least 2 characters
            if end_offset > start_offset + 1 {
                // Remove first character
                edits.push((start_offset..start_offset + 1, "".to_string()));
                // Remove last character (adjust for first deletion)
                edits.push(((end_offset - 1)..(end_offset), "".to_string()));
            }
        }
        
        if !edits.is_empty() {
            editor.edit(edits, cx);
        }
    });
}

fn helix_text_object_around(
    vim: &mut Vim,
    _: &TextObjectAround,
    window: &mut Window,
    cx: &mut Context<Vim>,
) {
    // Basic implementation: select around word boundaries
    vim.update_editor(window, cx, |_, editor, window, cx| {
        editor.change_selections(Some(Autoscroll::fit()), window, cx, |s| {
            s.move_with(|map, selection| {
                // Use vim's word object logic to select around current word
                if let Some(range) = (crate::object::Object::Word { ignore_punctuation: false }).range(map, selection.clone(), true) {
                    selection.start = range.start;
                    selection.end = range.end;
                }
            });
        });
    });
}

fn helix_text_object_inside(
    vim: &mut Vim,
    _: &TextObjectInside,
    window: &mut Window,
    cx: &mut Context<Vim>,
) {
    // Basic implementation: select inside word boundaries
    vim.update_editor(window, cx, |_, editor, window, cx| {
        editor.change_selections(Some(Autoscroll::fit()), window, cx, |s| {
            s.move_with(|map, selection| {
                // Use vim's word object logic to select inside current word
                if let Some(range) = (crate::object::Object::Word { ignore_punctuation: false }).range(map, selection.clone(), false) {
                    selection.start = range.start;
                    selection.end = range.end;
                }
            });
        });
    });
}

impl Vim {
    /// Legacy helix motion interface - now delegates to proper movement system
    pub fn helix_normal_motion(
        &mut self,
        motion: Motion,
        times: Option<usize>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let count = times.unwrap_or(1);
        for _ in 0..count {
            // Use the proper helix movement system
            // In HelixNormal mode: just move cursor
            // In HelixSelect mode: extend selection
            let extend = self.is_helix_select_mode();
            self.helix_move_cursor(motion.clone(), extend, window, cx);
        }
    }
}