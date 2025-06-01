use editor::{scroll::Autoscroll};
use gpui::{Window, Context};
use language::Point;
use crate::Vim;

pub fn trim_selections(vim: &mut Vim, window: &mut Window, cx: &mut Context<Vim>) {
    vim.update_editor(window, cx, |_, editor, win, cx| {
        let buffer = editor.buffer().read(cx).snapshot(cx);
        let selections = editor.selections.all_adjusted(cx);
        let mut new_ranges = Vec::new();
        
        for selection in selections {
            let text = buffer.text_for_range(selection.range()).collect::<String>();
            
            // Skip empty selections or whitespace-only selections
            if selection.is_empty() || text.chars().all(|c| c.is_whitespace()) {
                continue;
            }
            
            // Find the trimmed boundaries
            let start_trim = text.chars().take_while(|c| c.is_whitespace()).count();
            let end_trim = text.chars().rev().take_while(|c| c.is_whitespace()).count();
            
            if start_trim > 0 || end_trim > 0 {
                let start_offset = buffer.point_to_offset(selection.start);
                let end_offset = buffer.point_to_offset(selection.end);
                let new_start = start_offset + start_trim;
                let new_end = end_offset.saturating_sub(end_trim);
                
                if new_start < new_end {
                    new_ranges.push(new_start..new_end);
                }
            } else {
                let start_offset = buffer.point_to_offset(selection.start);
                let end_offset = buffer.point_to_offset(selection.end);
                new_ranges.push(start_offset..end_offset);
            }
        }
        
        if !new_ranges.is_empty() {
            editor.change_selections(Some(Autoscroll::fit()), win, cx, |s| {
                s.select_ranges(new_ranges);
            });
        }
    });
}

pub fn align_selections(vim: &mut Vim, window: &mut Window, cx: &mut Context<Vim>) {
    vim.update_editor(window, cx, |_, editor, _win, cx| {
        let buffer = editor.buffer().read(cx).snapshot(cx);
        let selections = editor.selections.all_adjusted(cx);
        
        if selections.len() <= 1 {
            return;
        }
        
        // For simplicity, just add spaces after each selection to align to a common width
        let max_width = selections
            .iter()
            .map(|selection| {
                buffer.text_for_range(selection.range()).collect::<String>().chars().count()
            })
            .max()
            .unwrap_or(0);
        
        let mut edits = Vec::new();
        for selection in &selections {
            let text = buffer.text_for_range(selection.range()).collect::<String>();
            let current_width = text.chars().count();
            
            if current_width < max_width {
                let spaces_needed = max_width - current_width;
                let spaces = " ".repeat(spaces_needed);
                let end_offset = buffer.point_to_offset(selection.end);
                edits.push((end_offset..end_offset, spaces));
            }
        }
        
        if !edits.is_empty() {
            editor.edit(edits, cx);
        }
    });
}

pub fn rotate_selections(vim: &mut Vim, window: &mut Window, cx: &mut Context<Vim>, forward: bool) {
    vim.update_editor(window, cx, |_, editor, win, cx| {
        let selections = editor.selections.all_adjusted(cx);
        if selections.len() <= 1 {
            return;
        }
        
        let current_ranges: Vec<_> = selections.iter().map(|s| {
            let start_offset = editor.buffer().read(cx).snapshot(cx).point_to_offset(s.start);
            let end_offset = editor.buffer().read(cx).snapshot(cx).point_to_offset(s.end);
            start_offset..end_offset
        }).collect();
        
        let mut new_ranges = current_ranges.clone();
        
        if forward {
            // Move last selection to front
            if let Some(last) = new_ranges.pop() {
                new_ranges.insert(0, last);
            }
        } else {
            // Move first selection to back
            if !new_ranges.is_empty() {
                let first = new_ranges.remove(0);
                new_ranges.push(first);
            }
        }
        
        editor.change_selections(Some(Autoscroll::fit()), win, cx, |s| {
            s.select_ranges(new_ranges);
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
        
        // Create edits to replace each selection with the rotated content
        let edits: Vec<_> = selections
            .iter()
            .zip(contents.iter())
            .map(|(selection, new_content)| {
                let start_offset = buffer.point_to_offset(selection.start);
                let end_offset = buffer.point_to_offset(selection.end);
                (start_offset..end_offset, new_content.clone())
            })
            .collect();
        
        if !edits.is_empty() {
            editor.edit(edits, cx);
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
            
            // Calculate new positions on target line, clamping to line bounds
            let target_line_len = buffer.line_len(multi_buffer::MultiBufferRow(target_row));
            let new_start_col = start_point.column.min(target_line_len);
            let new_end_col = end_point.column.min(target_line_len);
            
            let new_start_point = Point::new(target_row, new_start_col);
            let new_end_point = Point::new(target_row, new_end_col);
            
            // Convert points to offsets
            let new_start_offset = buffer.point_to_offset(new_start_point);
            let new_end_offset = buffer.point_to_offset(new_end_point);
            
            // Ensure we have at least a cursor position, even if selection collapses
            if new_start_offset <= new_end_offset {
                new_ranges.push(new_start_offset..new_end_offset.max(new_start_offset));
            } else {
                // If start > end due to clamping, create a cursor at the end position
                new_ranges.push(new_end_offset..new_end_offset);
            }
        }
        
        if new_ranges.len() > selections.len() {
            editor.change_selections(Some(Autoscroll::fit()), window, cx, |s| {
                s.select_ranges(new_ranges);
            });
        }
    });
}