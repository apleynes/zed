pub mod selections;
pub mod movement;
pub mod mode;
pub mod core;
pub mod regex_selection;
pub mod word_movement_tests;
pub mod debug_harness;
pub mod verification;
pub mod match_mode;

// Re-export commonly used match_mode types for tests
// pub use match_mode::{SelectTextObjectChar};

#[cfg(test)]
mod test;

#[cfg(test)]
mod movement_test;

#[cfg(test)]
mod selection_test;

#[cfg(test)]
mod fix_test;

#[cfg(test)]
mod selection_operation_tests;

#[cfg(test)]
mod find_movement_tests;

#[cfg(test)]
mod core_tests;

#[cfg(test)]
mod regex_selection_tests;

mod boundary_debug;

use editor::{Editor, scroll::Autoscroll};
use gpui::{Window, Context, actions};
use crate::{Vim, motion::Motion};

// Additional imports for ported functionality
// use regex::Regex;
// use anyhow;
// use editor::ToOffset;

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
        
        // General operations
        SelectAll,
        HelixYank,
        HelixReplace,
        HelixReplaceWithYanked,
        
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
    mode::register(editor, cx);
    movement::register(editor, cx);
    selections::register(editor, cx);
    regex_selection::register(editor, cx);
    match_mode::register(editor, cx);
    
    // Register mode switching
    mode::register(editor, cx);
    
    // Register regex selection operations
    regex_selection::register(editor, cx);
    
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
    
    // General operations
    Vim::action(editor, cx, helix_select_all);
    Vim::action(editor, cx, helix_yank);
    Vim::action(editor, cx, helix_replace);
    Vim::action(editor, cx, helix_replace_with_yanked);
    
    // Note: Match mode operations are registered in match_mode::register()
    // Removed duplicate registrations that were overriding the real implementations
}

// Pure selection manipulation functions that work directly with editor state
// These bypass vim infrastructure entirely to avoid mode conflicts

fn helix_collapse_selection(
    vim: &mut Vim,
    _: &CollapseSelection,
    window: &mut Window,
    cx: &mut Context<Vim>,
) {
    vim.update_editor(window, cx, |_, editor, _window, cx| {
        // Extract buffer data to calculate cursor positions
        let buffer = editor.buffer().read(cx);
        let snapshot = buffer.snapshot(cx);
        let rope_text = rope::Rope::from(snapshot.text());
        
        editor.change_selections(Some(Autoscroll::fit()), _window, cx, |s| {
            s.move_with(|map, selection| {
                // Calculate the proper cursor position using Helix semantics
                let cursor_pos = if selection.is_empty() {
                    // Empty selection: cursor is at head
                    selection.head()
                } else {
                    // Non-empty selection: calculate cursor position like Helix
                    let anchor_byte_offset = selection.tail().to_offset(map, editor::Bias::Left);
                    let head_byte_offset = selection.head().to_offset(map, editor::Bias::Left);
                    
                    let anchor_offset = core::byte_offset_to_char_index(&rope_text, anchor_byte_offset);
                    let head_offset = core::byte_offset_to_char_index(&rope_text, head_byte_offset);
                    
                    // Create Helix range and get cursor position
                    let helix_range = core::Range::new(anchor_offset, head_offset);
                    let cursor_char_index = helix_range.cursor(&rope_text);
                    
                    // Convert back to Zed coordinates (no adjustment needed)
                    let cursor_byte_offset = core::char_index_to_byte_offset(&rope_text, cursor_char_index);
                    let cursor_point = snapshot.offset_to_point(cursor_byte_offset);
                    let cursor_display = editor::DisplayPoint::new(editor::display_map::DisplayRow(cursor_point.row), cursor_point.column);
                    
                    cursor_display
                };
                
                selection.collapse_to(cursor_pos, selection.goal);
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
    vim.update_editor(window, cx, |_, editor, _window, cx| {
        editor.change_selections(Some(Autoscroll::fit()), _window, cx, |s| {
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
    vim.update_editor(window, cx, |_, editor, _window, cx| {
        let selections = editor.selections.all_adjusted(cx);
        if selections.len() <= 1 {
            return;
        }
        
        // Find the overall range from first to last selection
        let first_selection = selections.iter().min_by_key(|s| s.start);
        let last_selection = selections.iter().max_by_key(|s| s.end);
        
        if let (Some(first), Some(last)) = (first_selection, last_selection) {
            // Reset primary index since we're creating a new single merged selection (like Helix merge_ranges)
            selections::reset_primary_selection_index();
            
            let merged_range = first.start..last.end;
            editor.change_selections(Some(Autoscroll::fit()), _window, cx, |s| {
                s.select_ranges([merged_range]);
            });
        }
    });
}

fn helix_merge_consecutive_selections(
    vim: &mut Vim,
    _: &MergeConsecutiveSelections,
    window: &mut Window,
    cx: &mut Context<Vim>,
) {
    vim.update_editor(window, cx, |_, editor, _window, cx| {
        let selections = editor.selections.all_adjusted(cx);
        if selections.len() <= 1 {
            return;
        }
        
        let buffer = editor.buffer().read(cx).snapshot(cx);
        let mut merged_ranges = Vec::new();
        let mut current_range: Option<std::ops::Range<usize>> = None;
        
        // Sort selections by start position and convert to offsets
        let mut sorted_selections: Vec<_> = selections.iter().map(|selection| {
            let start_offset = buffer.point_to_offset(selection.start);
            let end_offset = buffer.point_to_offset(selection.end);
            start_offset..end_offset
        }).collect();
        sorted_selections.sort_by_key(|range| range.start);
        
        for range in sorted_selections {
            match current_range {
                None => current_range = Some(range),
                Some(ref mut current) => {
                    // Check if ranges are consecutive (adjacent with no gap)
                    if current.end == range.start {
                        // Merge consecutive ranges
                        current.end = range.end;
                    } else {
                        // Not consecutive, save current and start new
                        merged_ranges.push(current.clone());
                        *current = range;
                    }
                }
            }
        }
        
        // Don't forget the last range
        if let Some(last_range) = current_range {
            merged_ranges.push(last_range);
        }
        
        // Reset primary index since we're creating new selections (like Helix merge_consecutive_ranges)
        selections::reset_primary_selection_index();
        
        editor.change_selections(Some(Autoscroll::fit()), _window, cx, |s| {
            s.select_ranges(merged_ranges);
        });
    });
}

fn helix_keep_primary_selection(
    vim: &mut Vim,
    _: &KeepPrimarySelection,
    window: &mut Window,
    cx: &mut Context<Vim>,
) {
    vim.update_editor(window, cx, |_, editor, _window, cx| {
        let selections = editor.selections.all_adjusted(cx);
        if let Some(primary) = selections.iter().next() {
            // Reset primary index since we're creating a new single selection (like Helix Selection::single)
            selections::reset_primary_selection_index();
            
            editor.change_selections(Some(Autoscroll::fit()), _window, cx, |s| {
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
    eprintln!("helix_remove_primary_selection called");
    vim.update_editor(window, cx, |_, editor, _window, cx| {
        let selections = editor.selections.all_adjusted(cx);
        eprintln!("Remove primary: selections.len() = {}", selections.len());
        if selections.len() <= 1 {
            eprintln!("Remove primary: Early return - only {} selections", selections.len());
            return; // Can't remove the only selection
        }
        
        // Get the current primary index from our tracking and validate it
        let primary_index = selections::get_primary_selection_index();
        eprintln!("Remove primary: primary_index = {}", primary_index);
        let primary_index = if primary_index >= selections.len() {
            // Primary index is out of bounds, reset to 0 and use 0
            eprintln!("Remove primary: primary_index out of bounds, resetting to 0");
            selections::set_primary_selection_index(0);
            0
        } else {
            primary_index
        };
        
        // Create remaining selections by skipping the primary
        let remaining: Vec<_> = selections
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != primary_index)
            .map(|(_, s)| s.range())
            .collect();
        
        eprintln!("Remove primary: remaining.len() = {}", remaining.len());
        
        // Update the primary index for the remaining selections
        if primary_index > 0 && primary_index >= remaining.len() {
            // If we removed the last selection, make the new last selection primary
            selections::set_primary_selection_index(remaining.len() - 1);
        } else if primary_index > 0 {
            // If we removed a middle selection, keep the same index (which now points to the next selection)
            selections::set_primary_selection_index(primary_index.min(remaining.len() - 1));
        } else {
            // If we removed the first selection, the new first becomes primary
            selections::set_primary_selection_index(0);
        }
        
        editor.change_selections(Some(Autoscroll::fit()), _window, cx, |s| {
            s.select_ranges(remaining);
        });
        
        // Debug: Check if the change actually took effect
        let new_selections = editor.selections.all_adjusted(cx);
        eprintln!("Remove primary: new_selections.len() after change = {}", new_selections.len());
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

// Note: Match mode functions are implemented in match_mode.rs
// Removed stub functions that were overriding the real implementations

fn helix_select_all(
    vim: &mut Vim,
    _: &SelectAll,
    window: &mut Window,
    cx: &mut Context<Vim>,
) {
    vim.update_editor(window, cx, |_, editor, _window, cx| {
        editor.select_all(&editor::actions::SelectAll {}, _window, cx);
    });
}

fn helix_yank(
    vim: &mut Vim,
    _: &HelixYank,
    window: &mut Window,
    cx: &mut Context<Vim>,
) {
    vim.update_editor(window, cx, |_, editor, _window, cx| {
        // Helix-style copy that doesn't change modes or affect cursor position
        let selections = editor.selections.all_adjusted(cx);
        let buffer = editor.buffer().read(cx).snapshot(cx);
        
        // Collect text from all selections
        let mut clipboard_text = Vec::new();
        for selection in selections.iter() {
            let text = buffer.text_for_range(selection.range()).collect::<String>();
            clipboard_text.push(text);
        }
        
        // Join with newlines for multi-selection copy
        let final_text = clipboard_text.join("\n");
        
        // Copy to system clipboard
        cx.write_to_clipboard(gpui::ClipboardItem::new_string(final_text));
    });
}

fn helix_replace(
    vim: &mut Vim,
    _: &HelixReplace,
    window: &mut Window,
    cx: &mut Context<Vim>,
) {
    // Push the HelixReplace operator - this will cause vim to wait for character input
    vim.push_operator(crate::state::Operator::HelixReplace, window, cx);
}

fn helix_replace_with_yanked(
    vim: &mut Vim,
    _: &HelixReplaceWithYanked,
    window: &mut Window,
    cx: &mut Context<Vim>,
) {
    vim.update_editor(window, cx, |_, editor, _window, cx| {
        // Get the yanked text from the clipboard
        let clipboard_text = cx.read_from_clipboard()
            .and_then(|item| item.text().map(|t| t.to_string()))
            .unwrap_or_default();
        
        if clipboard_text.is_empty() {
            return;
        }
        
        editor.transact(_window, cx, |editor, _window, cx| {
            let selections = editor.selections.all_adjusted(cx);
            let buffer = editor.buffer().read(cx).snapshot(cx);
            let mut edits = Vec::new();
            
            // Replace each selection with the yanked text
            for selection in selections.iter() {
                if !selection.is_empty() {
                    let _start_offset = buffer.point_to_offset(selection.start);
                    let _end_offset = buffer.point_to_offset(selection.end);
                    let start_anchor = buffer.anchor_before(selection.start);
                    let end_anchor = buffer.anchor_before(selection.end);
                    
                    edits.push((start_anchor..end_anchor, clipboard_text.clone()));
                }
            }
            
            if !edits.is_empty() {
                editor.edit(edits, cx);
            }
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