use editor::{scroll::Autoscroll};
use gpui::{Window, Context};
use language::Point;
use crate::Vim;

pub fn trim_selections(vim: &mut Vim, window: &mut Window, cx: &mut Context<Vim>) {
    vim.update_editor(window, cx, |_, editor, window, cx| {
        let buffer = editor.buffer().read(cx).snapshot(cx);
        let selections = editor.selections.all_adjusted(cx);
        let mut new_ranges = Vec::new();
        
        for selection in selections {
            // Handle empty selections (cursors) - keep them as is
            if selection.is_empty() {
                let offset = buffer.point_to_offset(selection.start);
                new_ranges.push(offset..offset);
                continue;
            }
            
            let start_offset = buffer.point_to_offset(selection.start);
            let text = buffer.text_for_range(selection.range()).collect::<String>();
            
            // Skip whitespace-only selections - they become cursors at the start
            if text.chars().all(|c| c.is_whitespace()) {
                new_ranges.push(start_offset..start_offset);
                continue;
            }
            
            // Find trimmed boundaries by skipping whitespace
            let start_trim = text.chars().take_while(|c| c.is_whitespace()).count();
            let end_trim = text.chars().rev().take_while(|c| c.is_whitespace()).count();
            
            // Calculate new boundaries in terms of character offsets
            let total_chars = text.chars().count();
            let trimmed_start_char = start_trim;
            let trimmed_end_char = total_chars.saturating_sub(end_trim);
            
            if trimmed_start_char >= trimmed_end_char {
                // Selection becomes empty after trimming
                new_ranges.push(start_offset..start_offset);
                continue;
            }
            
            // Convert character positions back to byte offsets
            let mut char_indices: Vec<_> = text.char_indices().collect();
            char_indices.push((text.len(), '\0')); // Add sentinel for end position
            
            let start_byte_offset = char_indices[trimmed_start_char].0;
            let end_byte_offset = if trimmed_end_char < char_indices.len() - 1 {
                char_indices[trimmed_end_char].0
            } else {
                text.len()
            };
            
            let new_start = start_offset + start_byte_offset;
            let new_end = start_offset + end_byte_offset;
            
            new_ranges.push(new_start..new_end);
        }
        
        if !new_ranges.is_empty() {
            editor.change_selections(Some(Autoscroll::fit()), window, cx, |s| {
                s.select_ranges(new_ranges);
            });
        }
    });
}

pub fn align_selections(vim: &mut Vim, window: &mut Window, cx: &mut Context<Vim>) {
    vim.update_editor(window, cx, |_, editor, window, cx| {
        let buffer = editor.buffer().read(cx).snapshot(cx);
        let selections = editor.selections.all_adjusted(cx);
        
        if selections.len() <= 1 {
            return;
        }
        
        // Find the maximum width across all selections
        let max_width = selections
            .iter()
            .map(|selection| {
                buffer.text_for_range(selection.range()).collect::<String>().chars().count()
            })
            .max()
            .unwrap_or(0);
        
        let mut edits = Vec::new();
        let mut new_ranges = Vec::new();
        let mut spaces_added_cumulative = 0;
        
        for selection in &selections {
            let text = buffer.text_for_range(selection.range()).collect::<String>();
            let current_width = text.chars().count();
            let start_offset = buffer.point_to_offset(selection.start);
            let end_offset = buffer.point_to_offset(selection.end);
            
            if current_width < max_width {
                let spaces_needed = max_width - current_width;
                let spaces = " ".repeat(spaces_needed);
                edits.push((end_offset..end_offset, spaces));
                
                // New selection should include the added spaces
                let new_start = start_offset + spaces_added_cumulative;
                let new_end = end_offset + spaces_added_cumulative + spaces_needed;
                new_ranges.push(new_start..new_end);
                spaces_added_cumulative += spaces_needed;
            } else {
                // No spaces added, but adjust for previous additions
                let new_start = start_offset + spaces_added_cumulative;
                let new_end = end_offset + spaces_added_cumulative;
                new_ranges.push(new_start..new_end);
            }
        }
        
        if !edits.is_empty() {
            editor.edit(edits, cx);
            
            // Update selections to include the added spaces
            editor.change_selections(Some(Autoscroll::fit()), window, cx, |s| {
                s.select_ranges(new_ranges);
            });
        }
    });
}

pub fn rotate_selections(vim: &mut Vim, window: &mut Window, cx: &mut Context<Vim>, forward: bool) {
    vim.update_editor(window, cx, |_, editor, window, cx| {
        let selections = editor.selections.all_adjusted(cx);
        if selections.len() <= 1 {
            return;
        }
        
        // Get current selection ranges and preserve their order
        let ranges: Vec<_> = selections.iter().map(|s| {
            let start_offset = editor.buffer().read(cx).snapshot(cx).point_to_offset(s.start);
            let end_offset = editor.buffer().read(cx).snapshot(cx).point_to_offset(s.end);
            start_offset..end_offset
        }).collect();
        
        // Calculate the new primary selection index
        // In Zed, the primary is always first, so we need to rotate which selection becomes first
        let current_primary = 0; // Primary is always first in Zed's selection list
        let new_primary = if forward {
            (current_primary + 1) % ranges.len()
        } else {
            if current_primary == 0 {
                ranges.len() - 1
            } else {
                current_primary - 1
            }
        };
        
        // Reorder selections to make the new primary selection first
        // This simulates Helix's primary_index rotation by reordering the selections
        let mut reordered_ranges = Vec::new();
        reordered_ranges.push(ranges[new_primary].clone());
        
        // Add all other selections in their original order, skipping the new primary
        for (i, range) in ranges.iter().enumerate() {
            if i != new_primary {
                reordered_ranges.push(range.clone());
            }
        }
        
        editor.change_selections(Some(Autoscroll::fit()), window, cx, |s| {
            s.select_ranges(reordered_ranges);
        });
    });
}

pub fn rotate_selection_contents(vim: &mut Vim, window: &mut Window, cx: &mut Context<Vim>, forward: bool) {
    vim.update_editor(window, cx, |_, editor, _window, cx| {
        let buffer = editor.buffer().read(cx).snapshot(cx);
        let selections = editor.selections.all_adjusted(cx);
        
        if selections.len() <= 1 {
            return;
        }
        
        // Extract text content from each selection
        let mut contents: Vec<String> = selections
            .iter()
            .map(|selection| buffer.text_for_range(selection.range()).collect())
            .collect();
        
        // Rotate the contents
        if forward {
            if let Some(last) = contents.pop() {
                contents.insert(0, last);
            }
        } else {
            if !contents.is_empty() {
                let first = contents.remove(0);
                contents.push(first);
            }
        }
        
        // Calculate new selection ranges before editing
        let mut edits = Vec::new();
        let mut new_ranges = Vec::new();
        let mut cumulative_offset = 0i32;
        
        for (selection, new_content) in selections.iter().zip(contents.iter()) {
            let start_offset = buffer.point_to_offset(selection.start);
            let end_offset = buffer.point_to_offset(selection.end);
            let original_len = end_offset - start_offset;
            
            // Adjust start position for cumulative changes
            let adjusted_start = (start_offset as i32 + cumulative_offset) as usize;
            let adjusted_end = adjusted_start + new_content.len();
            
            edits.push((start_offset..end_offset, new_content.clone()));
            new_ranges.push(adjusted_start..adjusted_end);
            
            // Track cumulative offset change
            let size_diff = (new_content.len() as i32) - (original_len as i32);
            cumulative_offset += size_diff;
        }
        
        if !edits.is_empty() {
            editor.edit(edits, cx);
            
            // Apply the pre-calculated selection ranges
            editor.change_selections(Some(Autoscroll::fit()), _window, cx, |s| {
                s.select_ranges(new_ranges);
            });
        }
    });
}

pub fn copy_selection_on_line(vim: &mut Vim, window: &mut Window, cx: &mut Context<Vim>, next_line: bool) {
    vim.update_editor(window, cx, |_, editor, window, cx| {
        let buffer = editor.buffer().read(cx).snapshot(cx);
        let selections = editor.selections.all_adjusted(cx);
        let mut new_ranges = Vec::new();
        
        // Keep original selections
        for selection in &selections {
            let start_offset = buffer.point_to_offset(selection.start);
            let end_offset = buffer.point_to_offset(selection.end);
            new_ranges.push(start_offset..end_offset);
        }
        
        // Add copied selections on adjacent lines
        for selection in &selections {
            let start_point = selection.start;
            let end_point = selection.end;
            
            let target_row = if next_line {
                start_point.row + 1
            } else {
                if start_point.row > 0 { start_point.row - 1 } else { continue; }
            };
            
            // Check if target line exists
            if target_row > buffer.max_point().row {
                continue;
            }
            
            // Get the actual line text to determine proper bounds
            let target_line_len = buffer.line_len(multi_buffer::MultiBufferRow(target_row));
            
            // Calculate the selection width in the original line
            let original_width = end_point.column.saturating_sub(start_point.column);
            
            // Calculate new positions on target line, clamping to line bounds
            let new_start_col = start_point.column.min(target_line_len);
            let new_end_col = (start_point.column + original_width).min(target_line_len);
            
            // Ensure we have at least a cursor position if we can't maintain the full width
            let final_end_col = if new_end_col <= new_start_col && original_width > 0 {
                // If we can't maintain width, just place cursor at the start position
                new_start_col
            } else {
                new_end_col
            };
            
            let new_start_point = Point::new(target_row, new_start_col);
            let new_end_point = Point::new(target_row, final_end_col);
            
            // Convert points to offsets
            let new_start_offset = buffer.point_to_offset(new_start_point);
            let new_end_offset = buffer.point_to_offset(new_end_point);
            
            // Add the new selection
            if new_start_offset <= new_end_offset {
                new_ranges.push(new_start_offset..new_end_offset);
            } else {
                // Fallback to cursor at start position
                new_ranges.push(new_start_offset..new_start_offset);
            }
        }
        
        // Update selections with both original and copied selections
        if new_ranges.len() > selections.len() {
            editor.change_selections(Some(Autoscroll::fit()), window, cx, |s| {
                s.select_ranges(new_ranges);
            });
        }
    });
}