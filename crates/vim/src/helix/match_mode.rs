use crate::{
    Vim, object::Object,
};
use editor::{scroll::Autoscroll, Editor, ToOffset, Bias};
use gpui::{actions, impl_actions, Window, Context};
use language::{SelectionGoal, Point};
use serde::Deserialize;
use schemars::JsonSchema;

#[derive(Clone, Debug, PartialEq, Deserialize, JsonSchema)]
pub struct SelectTextObjectChar {
    pub char: char,
    pub around: bool,
}

actions!(
    helix_match_mode,
    [
        MatchBrackets,
        SurroundAdd,
        SurroundReplace,
        SurroundDelete,
        SelectTextObjectAround,
        SelectTextObjectInside,
        CancelTextObject,
    ]
);

impl_actions!(helix_match_mode, [SelectTextObjectChar]);

/// Register match mode actions
pub fn register(editor: &mut Editor, cx: &mut Context<Vim>) {
    Vim::action(editor, cx, helix_match_brackets);
    Vim::action(editor, cx, helix_surround_add);
    Vim::action(editor, cx, helix_surround_replace);
    Vim::action(editor, cx, helix_surround_delete);
    Vim::action(editor, cx, helix_select_text_object_around);
    Vim::action(editor, cx, helix_select_text_object_inside);
    Vim::action(editor, cx, helix_select_text_object_char);
    Vim::action(editor, cx, helix_cancel_text_object);
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
    _window: &mut Window,
    cx: &mut Context<Vim>,
) {
    println!("DEBUG: helix_surround_add called");
    
    // Clear any existing state first
    vim.match_mode_awaiting_text_object = None;
    vim.match_mode_skip_next_text_object_intercept = false;
    
    // Set the context for surround character input
    vim.match_mode_awaiting_surround_add = true;
    vim.match_mode_skip_next_text_object_intercept = true; // Skip the current keystroke
    vim.status_label = Some("Surround with: ".into());
    println!("DEBUG: Set match_mode_awaiting_surround_add to true");
    cx.notify();
}

/// Surround replace functionality - replace surrounding characters
fn helix_surround_replace(
    vim: &mut Vim,
    _: &SurroundReplace,
    _window: &mut Window,
    cx: &mut Context<Vim>,
) {
    println!("DEBUG: helix_surround_replace called");
    
    // Clear any existing state first
    vim.match_mode_awaiting_text_object = None;
    vim.match_mode_skip_next_text_object_intercept = false;
    
    // Set the context for surround replace character input
    vim.match_mode_awaiting_surround_replace_from = true;
    vim.match_mode_skip_next_text_object_intercept = true; // Skip the current keystroke
    vim.status_label = Some("Replace surround from: ".into());
    println!("DEBUG: Set match_mode_awaiting_surround_replace_from to true");
    cx.notify();
}

/// Surround delete functionality - delete surrounding characters
fn helix_surround_delete(
    vim: &mut Vim,
    _: &SurroundDelete,
    _window: &mut Window,
    cx: &mut Context<Vim>,
) {
    println!("DEBUG: helix_surround_delete called");
    
    // Clear any existing state first
    vim.match_mode_awaiting_text_object = None;
    vim.match_mode_skip_next_text_object_intercept = false;
    
    // Set the context for surround delete character input
    vim.match_mode_awaiting_surround_delete = true;
    vim.match_mode_skip_next_text_object_intercept = true; // Skip the current keystroke
    vim.status_label = Some("Delete surround: ".into());
    println!("DEBUG: Set match_mode_awaiting_surround_delete to true");
    cx.notify();
}

/// Select text object around functionality
fn helix_select_text_object_around(
    vim: &mut Vim,
    _: &SelectTextObjectAround,
    _window: &mut Window,
    cx: &mut Context<Vim>,
) {
    println!("DEBUG: helix_select_text_object_around called");
    
    // Clear any existing state first
    vim.match_mode_awaiting_text_object = None;
    vim.match_mode_skip_next_text_object_intercept = false;
    
    // Set the context for text object character input
    vim.match_mode_awaiting_text_object = Some(true);
    vim.match_mode_skip_next_text_object_intercept = true; // Skip the current keystroke
    vim.status_label = Some("Select around object: ".into());
    println!("DEBUG: Set match_mode_awaiting_text_object to Some(true)");
    cx.notify();
}

/// Select text object inside functionality
fn helix_select_text_object_inside(
    vim: &mut Vim,
    _: &SelectTextObjectInside,
    _window: &mut Window,
    cx: &mut Context<Vim>,
) {
    println!("DEBUG: helix_select_text_object_inside called");
    
    // Clear any existing state first
    vim.match_mode_awaiting_text_object = None;
    vim.match_mode_skip_next_text_object_intercept = false;
    
    // Set the context for text object character input
    vim.match_mode_awaiting_text_object = Some(false);
    vim.match_mode_skip_next_text_object_intercept = true; // Skip the current keystroke
    vim.status_label = Some("Select inside object: ".into());
    println!("DEBUG: Set match_mode_awaiting_text_object to Some(false)");
    cx.notify();
}

/// Handle character selection for text objects
fn helix_select_text_object_char(
    vim: &mut Vim,
    action: &SelectTextObjectChar,
    window: &mut Window,
    cx: &mut Context<Vim>,
) {
    println!("DEBUG: helix_select_text_object_char called with char='{}', around={}", action.char, action.around);
    
    // Clear the awaiting state
    vim.match_mode_awaiting_text_object = None;
    vim.status_label = None;
    
    // Convert character to object
    let object = match action.char {
        '(' | ')' => Object::Parentheses,
        '[' | ']' => Object::SquareBrackets,
        '{' | '}' => Object::CurlyBrackets,
        '<' | '>' => Object::AngleBrackets,
        '"' => Object::DoubleQuotes,
        '\'' => Object::Quotes,
        '`' => Object::BackQuotes,
        'w' => Object::Word { ignore_punctuation: false },
        'W' => Object::Word { ignore_punctuation: true },
        's' => Object::Sentence,
        'p' => Object::Paragraph,
        _ => {
            println!("DEBUG: Unsupported text object character: {}", action.char);
            return;
        }
    };
    
    println!("DEBUG: Selecting text object {:?} with around={}", object, action.around);
    
    // Directly select the text object
    vim.update_editor(window, cx, |_, editor, window, cx| {
        editor.change_selections(Some(Autoscroll::fit()), window, cx, |s| {
            s.move_with(|map, selection| {
                if let Some(range) = object.range(map, selection.clone(), action.around) {
                    selection.start = range.start;
                    selection.end = range.end;
                    selection.reversed = false;
                    selection.goal = SelectionGoal::None;
                }
            });
        });
    });
    
    cx.notify();
}

/// Handle character input for text object selection
pub fn handle_text_object_input(
    vim: &mut Vim,
    ch: char,
    around: bool,
    window: &mut Window,
    cx: &mut Context<Vim>,
) {
    println!("DEBUG: handle_text_object_input called with ch='{}', around={}", ch, around);
    
    // Clear the awaiting state immediately to prevent multiple calls
    vim.match_mode_awaiting_text_object = None;
    vim.match_mode_skip_next_text_object_intercept = false;
    vim.status_label = None;
    
    // Convert character to object
    let object = match ch {
        '(' | ')' => Object::Parentheses,
        '[' | ']' => Object::SquareBrackets,
        '{' | '}' => Object::CurlyBrackets,
        '<' | '>' => Object::AngleBrackets,
        '"' => Object::DoubleQuotes,
        '\'' => Object::Quotes,
        '`' => Object::BackQuotes,
        'w' => Object::Word { ignore_punctuation: false },
        'W' => Object::Word { ignore_punctuation: true },
        's' => Object::Sentence,
        'p' => Object::Paragraph,
        _ => {
            println!("DEBUG: Unsupported text object character: {}", ch);
            cx.notify();
            return;
        }
    };
    
    println!("DEBUG: Selecting text object {:?} with around={}", object, around);
    
    // Directly select the text object
    vim.update_editor(window, cx, |_, editor, window, cx| {
        editor.change_selections(Some(Autoscroll::fit()), window, cx, |s| {
            s.move_with(|map, selection| {
                if let Some(range) = object.range(map, selection.clone(), around) {
                    println!("DEBUG: Found text object range: {:?}", range);
                    selection.start = range.start;
                    selection.end = range.end;
                    selection.reversed = false;
                    selection.goal = SelectionGoal::None;
                } else {
                    println!("DEBUG: No text object range found for {:?}", object);
                }
            });
        });
    });
    
    cx.notify();
}

/// Cancel text object selection
fn helix_cancel_text_object(
    vim: &mut Vim,
    _: &CancelTextObject,
    _window: &mut Window,
    cx: &mut Context<Vim>,
) {
    println!("DEBUG: helix_cancel_text_object called");
    // Clear the awaiting state
    vim.match_mode_awaiting_text_object = None;
    vim.status_label = None;
    cx.notify();
}

/// Handle character input for surround add operation
pub fn handle_surround_add_input(
    vim: &mut Vim,
    ch: char,
    window: &mut Window,
    cx: &mut Context<Vim>,
) {
    println!("DEBUG: handle_surround_add_input called with ch='{}'", ch);
    
    // Clear the awaiting state immediately
    vim.match_mode_awaiting_surround_add = false;
    vim.match_mode_skip_next_text_object_intercept = false;
    vim.status_label = None;
    
    // Get the opening and closing characters
    let (open_char, close_char) = match ch {
        '(' | ')' => ('(', ')'),
        '[' | ']' => ('[', ']'),
        '{' | '}' => ('{', '}'),
        '<' | '>' => ('<', '>'),
        '"' => ('"', '"'),
        '\'' => ('\'', '\''),
        '`' => ('`', '`'),
        _ => {
            println!("DEBUG: Unsupported surround character: {}", ch);
            cx.notify();
            return;
        }
    };
    
    println!("DEBUG: Surrounding selections with '{}' and '{}'", open_char, close_char);
    
    // Surround all selections
    vim.update_editor(window, cx, |_, editor, _window, cx| {
        editor.transact(_window, cx, |editor, _window, cx| {
            let snapshot = editor.snapshot(_window, cx);
            let selections = editor.selections.all::<language::Point>(cx);
            let mut edits = Vec::new();
            
            // Process selections in reverse order to maintain correct positions
            for selection in selections.iter().rev() {
                let start_point = selection.start;
                let end_point = selection.end;
                
                // Create anchors for the start and end positions
                let start_anchor = snapshot.buffer_snapshot.anchor_before(start_point);
                let end_anchor = snapshot.buffer_snapshot.anchor_after(end_point);
                
                // Add opening character at start first
                edits.push((start_anchor..start_anchor, open_char.to_string()));
                // Add closing character at end
                edits.push((end_anchor..end_anchor, close_char.to_string()));
            }
            
            editor.edit(edits, cx);
            
            // Update selections to include the newly added surrounding characters
            editor.change_selections(None, _window, cx, |s| {
                s.move_with(|map, selection| {
                    // Convert display points to points for manipulation
                    let start_point = selection.start.to_point(map);
                    let end_point = selection.end.to_point(map);
                    
                    // Move start back by 1 to include opening character
                    let new_start = if start_point.column > 0 {
                        language::Point::new(start_point.row, start_point.column - 1)
                    } else {
                        start_point
                    };
                    
                    // Move end forward by 1 to include closing character
                    let new_end = language::Point::new(end_point.row, end_point.column + 1);
                    
                    // Convert back to display points and update selection
                    let new_start_display = map.point_to_display_point(new_start, language::Bias::Left);
                    let new_end_display = map.point_to_display_point(new_end, language::Bias::Right);
                    
                    selection.start = new_start_display;
                    selection.end = new_end_display;
                });
            });
        });
    });
    
    cx.notify();
}

/// Handle character input for surround delete operation
pub fn handle_surround_delete_input(
    vim: &mut Vim,
    ch: char,
    window: &mut Window,
    cx: &mut Context<Vim>,
) {
    println!("DEBUG: handle_surround_delete_input called with ch='{}'", ch);
    
    // Clear the awaiting state immediately
    vim.match_mode_awaiting_surround_delete = false;
    vim.match_mode_skip_next_text_object_intercept = false;
    vim.status_label = None;
    
    // Convert character to object for finding surrounding pairs
    let object = match ch {
        '(' | ')' => Object::Parentheses,
        '[' | ']' => Object::SquareBrackets,
        '{' | '}' => Object::CurlyBrackets,
        '<' | '>' => Object::AngleBrackets,
        '"' => Object::DoubleQuotes,
        '\'' => Object::Quotes,
        '`' => Object::BackQuotes,
        _ => {
            println!("DEBUG: Unsupported surround delete character: {}", ch);
            cx.notify();
            return;
        }
    };
    
    println!("DEBUG: Deleting surrounding {:?}", object);
    
    // Delete surrounding characters
    vim.update_editor(window, cx, |_, editor, _window, cx| {
        editor.transact(_window, cx, |editor, _window, cx| {
            let mut edits = Vec::new();
            let snapshot = editor.snapshot(_window, cx);
            
            // Store original cursor positions
            let original_selections = editor.selections.all::<language::Point>(cx);
            println!("DEBUG: Original selections: {:?}", original_selections);
            
            for selection in original_selections.iter() {
                // Convert to display point selection
                let display_selection = selection.map(|point| {
                    snapshot.display_snapshot.point_to_display_point(point, language::Bias::Left)
                });
                
                println!("DEBUG: Looking for {:?} around selection {:?}", object, display_selection);
                
                if let Some(range) = object.range(&snapshot.display_snapshot, display_selection, true) {
                    // The object.range() with around=true should return the range INCLUDING the brackets
                    // But we need to be careful about what exactly it returns
                    let start_point = range.start.to_point(&snapshot.display_snapshot);
                    let end_point = range.end.to_point(&snapshot.display_snapshot);
                    
                    println!("DEBUG: Found surrounding range: {:?} to {:?}", start_point, end_point);
                    
                    // Get the actual text to understand what we're dealing with
                    let start_offset = start_point.to_offset(&snapshot.buffer_snapshot);
                    let end_offset = end_point.to_offset(&snapshot.buffer_snapshot);
                    let text_slice = snapshot.buffer_snapshot.text_for_range(start_offset..end_offset).collect::<String>();
                    println!("DEBUG: Text in range: '{}'", text_slice);
                    
                    // For text objects with around=true, the range should include the delimiters
                    // So start_point should be at the opening bracket and end_point should be after the closing bracket
                    // We need to delete exactly the first and last characters of this range
                    
                    // Delete the last character (closing bracket)
                    let close_start_point = language::Point::new(end_point.row, end_point.column.saturating_sub(1));
                    let close_start_anchor = snapshot.buffer_snapshot.anchor_before(close_start_point);
                    let close_end_anchor = snapshot.buffer_snapshot.anchor_after(end_point);
                    edits.push((close_start_anchor..close_end_anchor, String::new()));
                    
                    // Delete the first character (opening bracket)
                    let open_end_point = language::Point::new(start_point.row, start_point.column + 1);
                    let open_start_anchor = snapshot.buffer_snapshot.anchor_before(start_point);
                    let open_end_anchor = snapshot.buffer_snapshot.anchor_after(open_end_point);
                    edits.push((open_start_anchor..open_end_anchor, String::new()));
                    
                    println!("DEBUG: Added {} edits for deletion", edits.len());
                } else {
                    println!("DEBUG: No surrounding range found for {:?}", object);
                }
            }
            
            println!("DEBUG: Total edits to apply: {}", edits.len());
            
            if !edits.is_empty() {
                editor.edit(edits, cx);
                println!("DEBUG: Applied edits successfully");
                
                // Restore cursor positions, adjusting for deleted characters
                editor.change_selections(None, _window, cx, |s| {
                    let new_selections: Vec<_> = original_selections.iter().map(|selection| {
                        // Adjust cursor position by subtracting 1 for the deleted opening character
                        let cursor_point = selection.head();
                        let adjusted_point = if cursor_point.column > 0 {
                            language::Point::new(cursor_point.row, cursor_point.column - 1)
                        } else {
                            cursor_point
                        };
                        let display_pos = s.display_map().point_to_display_point(adjusted_point, language::Bias::Left);
                        display_pos..display_pos
                    }).collect();
                    s.select_display_ranges(new_selections);
                });
                println!("DEBUG: Updated cursor positions");
            } else {
                println!("DEBUG: No edits to apply - surround delete failed");
            }
        });
    });
    
    cx.notify();
}

/// Handle character input for surround replace operation
pub fn handle_surround_replace_input(
    vim: &mut Vim,
    from_ch: char,
    to_ch: char,
    window: &mut Window,
    cx: &mut Context<Vim>,
) {
    println!("DEBUG: handle_surround_replace_input called with from_ch='{}', to_ch='{}'", from_ch, to_ch);
    
    // Clear the awaiting state immediately
    vim.match_mode_awaiting_surround_replace_to = false;
    vim.match_mode_surround_replace_from_char = None;
    vim.status_label = None;
    
    // Convert from character to object for finding surrounding pairs
    let from_object = match from_ch {
        '(' | ')' => Object::Parentheses,
        '[' | ']' => Object::SquareBrackets,
        '{' | '}' => Object::CurlyBrackets,
        '<' | '>' => Object::AngleBrackets,
        '"' => Object::DoubleQuotes,
        '\'' => Object::Quotes,
        '`' => Object::BackQuotes,
        _ => {
            println!("DEBUG: Unsupported surround replace from character: {}", from_ch);
            cx.notify();
            return;
        }
    };
    
    // Get the new opening and closing characters
    let (new_open_char, new_close_char) = match to_ch {
        '(' | ')' => ('(', ')'),
        '[' | ']' => ('[', ']'),
        '{' | '}' => ('{', '}'),
        '<' | '>' => ('<', '>'),
        '"' => ('"', '"'),
        '\'' => ('\'', '\''),
        '`' => ('`', '`'),
        _ => {
            println!("DEBUG: Unsupported surround replace to character: {}", to_ch);
            cx.notify();
            return;
        }
    };
    
    println!("DEBUG: Replacing surrounding {:?} with '{}' and '{}'", from_object, new_open_char, new_close_char);
    
    // Replace surrounding characters
    vim.update_editor(window, cx, |_, editor, _window, cx| {
        editor.transact(_window, cx, |editor, _window, cx| {
            let mut edits = Vec::new();
            let snapshot = editor.snapshot(_window, cx);
            
            for selection in editor.selections.all::<language::Point>(cx).iter() {
                // Convert to display point selection
                let display_selection = selection.map(|point| {
                    snapshot.display_snapshot.point_to_display_point(point, language::Bias::Left)
                });
                
                if let Some(range) = from_object.range(&snapshot.display_snapshot, display_selection, true) {
                    // Find the actual surrounding characters to replace
                    let start_point = range.start.to_point(&snapshot.display_snapshot);
                    let end_point = range.end.to_point(&snapshot.display_snapshot);
                    
                    // Replace closing character first (to maintain positions)
                    let close_range = snapshot.buffer_snapshot.anchor_before(end_point)
                        ..snapshot.buffer_snapshot.anchor_after(end_point);
                    edits.push((close_range, new_close_char.to_string()));
                    
                    // Replace opening character
                    let open_range = snapshot.buffer_snapshot.anchor_before(start_point)
                        ..snapshot.buffer_snapshot.anchor_after(start_point);
                    edits.push((open_range, new_open_char.to_string()));
                }
            }
            
            if !edits.is_empty() {
                editor.edit(edits, cx);
            }
        });
    });
    
    cx.notify();
} 