use crate::{
    Vim,
};
use editor::{scroll::Autoscroll, Editor, ToOffset, Bias};
use gpui::{actions, Window, Context};
use language::{SelectionGoal, Point};

actions!(
    helix_match_mode,
    [
        MatchBrackets,
        SurroundAdd,
        SurroundReplace,
        SurroundDelete,
        SelectTextObjectAround,
        SelectTextObjectInside,
    ]
);

/// Register match mode actions
pub fn register(editor: &mut Editor, cx: &mut Context<Vim>) {
    Vim::action(editor, cx, helix_match_brackets);
    Vim::action(editor, cx, helix_surround_add);
    Vim::action(editor, cx, helix_surround_replace);
    Vim::action(editor, cx, helix_surround_delete);
    Vim::action(editor, cx, helix_select_text_object_around);
    Vim::action(editor, cx, helix_select_text_object_inside);
}

/// Match brackets functionality - jump to matching bracket
fn helix_match_brackets(
    vim: &mut Vim,
    _: &MatchBrackets,
    window: &mut Window,
    cx: &mut Context<Vim>,
) {
    vim.update_editor(window, cx, |_, editor, window, cx| {
        editor.change_selections(Some(Autoscroll::fit()), window, cx, |s| {
            s.move_with(|display_map, selection| {
                // Get current cursor position as display point
                let cursor_display_point = selection.head();
                
                // Convert to point for line boundary calculations
                let point = cursor_display_point.to_point(display_map);
                
                // Use the same approach as the original vim implementation
                // Get line boundaries for bracket search
                let mut line_end = display_map.next_line_boundary(point).0;
                if line_end == point {
                    line_end = display_map.max_point().to_point(display_map);
                }
                let line_range = display_map.prev_line_boundary(point).0..line_end;
                let visible_line_range = line_range.start..Point::new(
                    line_range.end.row, 
                    line_range.end.column.saturating_sub(1)
                );
                
                // Use bracket_ranges instead of enclosing_bracket_ranges
                let ranges = display_map.buffer_snapshot.bracket_ranges(visible_line_range.clone());
                
                if let Some(ranges) = ranges {
                    let line_range_offsets = line_range.start.to_offset(&display_map.buffer_snapshot)
                        ..line_range.end.to_offset(&display_map.buffer_snapshot);
                    let mut closest_pair_destination = None;
                    let mut closest_distance = usize::MAX;
                    
                    // Convert cursor display point to offset for comparison
                    let cursor_offset = cursor_display_point.to_offset(display_map, Bias::Left);
                    
                    for (open_range, close_range) in ranges {
                        // Skip HTML/XML tags (like original implementation)
                        if display_map.buffer_snapshot.chars_at(open_range.start).next() == Some('<') {
                            continue;
                        }
                        
                        // Check if cursor is on opening bracket
                        if open_range.contains(&cursor_offset) && line_range_offsets.contains(&open_range.start) {
                            closest_pair_destination = Some(close_range.start);
                            break;
                        }
                        
                        // Check if cursor is on closing bracket  
                        if close_range.contains(&cursor_offset) && line_range_offsets.contains(&close_range.start) {
                            closest_pair_destination = Some(open_range.start);
                            break;
                        }
                        
                        // Find closest bracket pair if cursor is not on a bracket
                        if (open_range.contains(&cursor_offset) || open_range.start >= cursor_offset)
                            && line_range_offsets.contains(&open_range.start)
                        {
                            let distance = open_range.start.saturating_sub(cursor_offset);
                            if distance < closest_distance {
                                closest_pair_destination = Some(close_range.start);
                                closest_distance = distance;
                                continue;
                            }
                        }
                        
                        if (close_range.contains(&cursor_offset) || close_range.start >= cursor_offset)
                            && line_range_offsets.contains(&close_range.start)
                        {
                            let distance = close_range.start.saturating_sub(cursor_offset);
                            if distance < closest_distance {
                                closest_pair_destination = Some(close_range.start);  // Move to closing bracket
                                closest_distance = distance;
                                continue;
                            }
                        }
                    }
                    
                    if let Some(destination_offset) = closest_pair_destination {
                        // Convert offset back to display point
                        let destination_point = display_map.buffer_snapshot.offset_to_point(destination_offset);
                        let destination_display_point = display_map.point_to_display_point(destination_point, Bias::Left);
                        selection.collapse_to(destination_display_point, SelectionGoal::None);
                    }
                }
            })
        });
    });
}

/// Surround add functionality - surround current selection with character pair
fn helix_surround_add(
    vim: &mut Vim,
    _: &SurroundAdd,
    window: &mut Window,
    cx: &mut Context<Vim>,
) {
    // Use Zed's existing operator system for character input
    // Now extended to support HelixNormal and HelixSelect modes
    vim.push_operator(crate::state::Operator::AddSurrounds { target: None }, window, cx);
}

/// Surround replace functionality - replace surrounding characters
fn helix_surround_replace(
    _vim: &mut Vim,
    _: &SurroundReplace,
    _window: &mut Window,
    _cx: &mut Context<Vim>,
) {
    // TODO: Implement surround replace functionality
    // This should prompt for the character to replace, then prompt for the replacement character
    // For now, just show a placeholder message - but we need to avoid using vim operators
    // because they force return to Normal mode instead of HelixNormal
}

/// Surround delete functionality - delete surrounding characters
fn helix_surround_delete(
    vim: &mut Vim,
    _: &SurroundDelete,
    window: &mut Window,
    cx: &mut Context<Vim>,
) {
    // Use Zed's existing operator system for character input
    vim.push_operator(crate::state::Operator::DeleteSurrounds, window, cx);
}

/// Select text object around functionality
fn helix_select_text_object_around(
    _vim: &mut Vim,
    _: &SelectTextObjectAround,
    _window: &mut Window,
    _cx: &mut Context<Vim>,
) {
    // TODO: Implement text object around selection
    // This should prompt for a text object character and select around it
    // For now, just show a placeholder message - but we need to avoid using vim operators
    // because they force return to Normal mode instead of HelixNormal
}

/// Select text object inside functionality
fn helix_select_text_object_inside(
    _vim: &mut Vim,
    _: &SelectTextObjectInside,
    _window: &mut Window,
    _cx: &mut Context<Vim>,
) {
    // TODO: Implement text object inside selection
    // This should prompt for a text object character and select inside it
    // For now, just show a placeholder message - but we need to avoid using vim operators
    // because they force return to Normal mode instead of HelixNormal
} 