use crate::{motion::Motion, Vim};
use editor::{Editor, scroll::Autoscroll, DisplayPoint, Bias, display_map::DisplayRow};
use gpui::{actions, Context, Window};


actions!(
    helix_movement,
    [
        // Basic cursor movements (cursor-only in normal mode, extend in select mode)
        MoveCharLeft,
        MoveCharRight,
        MoveLineUp,
        MoveLineDown,
        
        // Word/find movements (create selections in normal mode, extend in select mode)
        MoveNextWordStart,
        MovePrevWordStart,
        MoveNextWordEnd,
        MovePrevWordEnd,
        MoveStartOfLine,
        MoveEndOfLine,
        MoveFirstNonWhitespace,
        MoveStartOfDocument,
        MoveEndOfDocument,
        
        // Find movements
        FindForward,
        FindBackward,
        FindForwardTill,
        FindBackwardTill,
        
        // WORD movements (ignore punctuation)
        MoveNextWordStartIgnorePunctuation,
        MovePrevWordStartIgnorePunctuation,
        MoveNextWordEndIgnorePunctuation,
        MovePrevWordEndIgnorePunctuation,
        
        // Other movements
        MatchBrackets,
    ]
);

pub fn register(editor: &mut Editor, cx: &mut Context<Vim>) {
    // Basic movements: cursor-only in normal mode, extend in select mode
    Vim::action(editor, cx, |vim, _: &MoveCharLeft, window, cx| {
        let extend = vim.is_helix_select_mode();
        vim.helix_move_cursor(Motion::Left, extend, window, cx);
    });
    
    Vim::action(editor, cx, |vim, _: &MoveCharRight, window, cx| {
        let extend = vim.is_helix_select_mode();
        vim.helix_move_cursor(Motion::Right, extend, window, cx);
    });
    
    Vim::action(editor, cx, |vim, _: &MoveLineUp, window, cx| {
        let extend = vim.is_helix_select_mode();
        vim.helix_move_cursor(Motion::Up { display_lines: true }, extend, window, cx);
    });
    
    Vim::action(editor, cx, |vim, _: &MoveLineDown, window, cx| {
        let extend = vim.is_helix_select_mode();
        vim.helix_move_cursor(Motion::Down { display_lines: true }, extend, window, cx);
    });
    
    // Word/find movements: create selections in normal mode, extend in select mode
    Vim::action(editor, cx, |vim, _: &MoveNextWordStart, window, cx| {
        vim.helix_word_move_cursor(Motion::NextWordStart { ignore_punctuation: false }, window, cx);
    });
    
    Vim::action(editor, cx, |vim, _: &MovePrevWordStart, window, cx| {
        vim.helix_word_move_cursor(Motion::PreviousWordStart { ignore_punctuation: false }, window, cx);
    });
    
    Vim::action(editor, cx, |vim, _: &MoveNextWordEnd, window, cx| {
        vim.helix_word_move_cursor(Motion::NextWordEnd { ignore_punctuation: false }, window, cx);
    });
    
    Vim::action(editor, cx, |vim, _: &MovePrevWordEnd, window, cx| {
        vim.helix_word_move_cursor(Motion::PreviousWordEnd { ignore_punctuation: false }, window, cx);
    });
    
    Vim::action(editor, cx, |vim, _: &MoveStartOfLine, window, cx| {
        vim.helix_word_move_cursor(Motion::StartOfLine { display_lines: true }, window, cx);
    });
    
    Vim::action(editor, cx, |vim, _: &MoveEndOfLine, window, cx| {
        vim.helix_word_move_cursor(Motion::EndOfLine { display_lines: true }, window, cx);
    });
    
    Vim::action(editor, cx, |vim, _: &MoveFirstNonWhitespace, window, cx| {
        vim.helix_word_move_cursor(Motion::FirstNonWhitespace { display_lines: true }, window, cx);
    });
    
    Vim::action(editor, cx, |vim, _: &MoveStartOfDocument, window, cx| {
        vim.helix_word_move_cursor(Motion::StartOfDocument, window, cx);
    });
    
    Vim::action(editor, cx, |vim, _: &MoveEndOfDocument, window, cx| {
        vim.helix_word_move_cursor(Motion::EndOfDocument, window, cx);
    });
    
    // Find movements: create selections to target character
    Vim::action(editor, cx, |vim, _: &FindForward, window, cx| {
        vim.push_operator(crate::Operator::HelixFindForward { before: false }, window, cx);
    });
    
    Vim::action(editor, cx, |vim, _: &FindBackward, window, cx| {
        vim.push_operator(crate::Operator::HelixFindBackward { after: false }, window, cx);
    });
    
    Vim::action(editor, cx, |vim, _: &FindForwardTill, window, cx| {
        vim.push_operator(crate::Operator::HelixFindForward { before: true }, window, cx);
    });
    
    Vim::action(editor, cx, |vim, _: &FindBackwardTill, window, cx| {
        vim.push_operator(crate::Operator::HelixFindBackward { after: true }, window, cx);
    });
    
    // WORD movements: create selections in normal mode, extend in select mode
    Vim::action(editor, cx, |vim, _: &MoveNextWordStartIgnorePunctuation, window, cx| {
        vim.helix_word_move_cursor(Motion::NextWordStart { ignore_punctuation: true }, window, cx);
    });
    
    Vim::action(editor, cx, |vim, _: &MovePrevWordStartIgnorePunctuation, window, cx| {
        vim.helix_word_move_cursor(Motion::PreviousWordStart { ignore_punctuation: true }, window, cx);
    });
    
    Vim::action(editor, cx, |vim, _: &MoveNextWordEndIgnorePunctuation, window, cx| {
        vim.helix_word_move_cursor(Motion::NextWordEnd { ignore_punctuation: true }, window, cx);
    });
    
    Vim::action(editor, cx, |vim, _: &MovePrevWordEndIgnorePunctuation, window, cx| {
        vim.helix_word_move_cursor(Motion::PreviousWordEnd { ignore_punctuation: true }, window, cx);
    });
}

impl Vim {
    /// Helix-style cursor movement for basic movements (h,j,k,l)
    /// 
    /// If extend is false: moves cursor only
    /// If extend is true: extends current selection to destination (for select mode)
    pub fn helix_move_cursor(
        &mut self,
        motion: Motion,
        extend: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if extend {
            // In select mode, create or extend selections
            self.update_editor(window, cx, |_, editor, window, cx| {
                let text_layout_details = editor.text_layout_details(window);
                editor.change_selections(Some(Autoscroll::fit()), window, cx, |s| {
                    s.move_with(|map, selection| {
                        let current_head = selection.head();
                        
                        // Calculate destination position
                        let Some((new_head, goal)) = motion.move_point(
                            map,
                            current_head,
                            selection.goal,
                            Some(1),
                            &text_layout_details,
                        ) else {
                            return;
                        };

                        if selection.is_empty() {
                            // No existing selection - create one from current cursor to destination
                            selection.set_tail(current_head, selection.goal);
                            selection.set_head(new_head, goal);
                        } else {
                            // Existing selection - extend it to destination
                            selection.set_head(new_head, goal);
                        }
                    });
                });
            });
        } else {
            // In normal mode, move cursor only (no selection creation)
            self.normal_motion(motion, None, Some(1), false, window, cx);
        }
    }

    /// Helix-style word movement (w,b,e,etc.) 
    /// 
    /// In normal mode: creates selections (helix default behavior)
    /// In select mode: extends existing selections
    pub fn helix_word_move_cursor(
        &mut self,
        motion: Motion,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.is_helix_select_mode() {
            // In select mode, extend existing selections by moving only the head
            self.update_editor(window, cx, |_, editor, window, cx| {
                let text_layout_details = editor.text_layout_details(window);
                editor.change_selections(Some(Autoscroll::fit()), window, cx, |s| {
                    s.move_with(|map, selection| {
                        let start_pos = selection.head();
                        
                        eprintln!("DEBUG: motion = {:?}", motion);
                        
                        // Calculate destination position using motion system
                        if let Some((mut end_pos, goal)) = motion.move_point(
                            map,
                            start_pos,
                            selection.goal,
                            Some(1),
                            &text_layout_details,
                        ) {
                            // For word end motions, vim returns exclusive position but Helix expects inclusive
                            if matches!(motion, Motion::NextWordEnd { .. } | Motion::PreviousWordEnd { .. }) {
                                end_pos = editor::movement::right(map, end_pos);
                            }
                            
                            // In select mode, extend existing selection - only move the head
                            selection.set_head(end_pos, goal);
                        }
                    });
                });
            });
        } else {
            // In normal mode, create selection from current cursor to destination
            self.update_editor(window, cx, |_, editor, window, cx| {
                let text_layout_details = editor.text_layout_details(window);
                
                // Extract buffer data outside the selection closure
                let buffer = editor.buffer().read(cx);
                let snapshot = buffer.snapshot(cx);
                let rope = rope::Rope::from(snapshot.text());
                
                editor.change_selections(Some(Autoscroll::fit()), window, cx, |s| {
                    s.move_with(|map, selection| {
                        // Get current cursor position
                        let start_pos = selection.head();
                        
                        eprintln!("DEBUG: motion = {:?}", motion);
                        
                        // Use Helix core movement functions for word movements
                        if matches!(motion, Motion::NextWordStart { .. } | Motion::PreviousWordStart { .. } | 
                                          Motion::NextWordEnd { .. } | Motion::PreviousWordEnd { .. }) {
                            eprintln!("DEBUG: Taking Helix core branch");
                            // Convert to Helix coordinate system
                            let start_offset = start_pos.to_offset(map, editor::Bias::Left);
                            let helix_range = super::core::Range::new(start_offset, start_offset);
                            
                            // Call our Helix core functions
                            let new_range = match motion {
                                Motion::NextWordStart { ignore_punctuation: false } => {
                                    super::core::move_next_word_start(&rope, helix_range, 1)
                                },
                                Motion::PreviousWordStart { ignore_punctuation: false } => {
                                    super::core::move_prev_word_start(&rope, helix_range, 1)
                                },
                                Motion::NextWordEnd { ignore_punctuation: false } => {
                                    super::core::move_next_word_end(&rope, helix_range, 1)
                                },
                                Motion::PreviousWordEnd { ignore_punctuation: false } => {
                                    super::core::move_prev_word_end(&rope, helix_range, 1)
                                },
                                Motion::NextWordStart { ignore_punctuation: true } => {
                                    super::core::move_next_long_word_start(&rope, helix_range, 1)
                                },
                                Motion::PreviousWordStart { ignore_punctuation: true } => {
                                    super::core::move_prev_long_word_start(&rope, helix_range, 1)
                                },
                                Motion::NextWordEnd { ignore_punctuation: true } => {
                                    super::core::move_next_long_word_end(&rope, helix_range, 1)
                                },
                                Motion::PreviousWordEnd { ignore_punctuation: true } => {
                                    super::core::move_prev_long_word_end(&rope, helix_range, 1)
                                },
                                _ => helix_range,
                            };
                            
                            // Convert back to Zed coordinate system
                            let anchor_point = snapshot.offset_to_point(new_range.anchor);
                            // Helix ranges are inclusive, but Zed selections work with the actual positions
                            // Don't subtract 1 - use the actual head position from Helix
                            let head_point = snapshot.offset_to_point(new_range.head);
                            
                            let anchor_display = DisplayPoint::new(DisplayRow(anchor_point.row), anchor_point.column);
                            let head_display = DisplayPoint::new(DisplayRow(head_point.row), head_point.column);
                            
                            eprintln!("DEBUG coordinate conversion:");
                            eprintln!("  new_range: {:?}", new_range);
                            eprintln!("  anchor_point: {:?}, head_point: {:?}", anchor_point, head_point);
                            eprintln!("  anchor_display: {:?}, head_display: {:?}", anchor_display, head_display);
                            eprintln!("  original start_pos: {:?}", start_pos);
                            
                            // Debug: Show what text is being selected
                            let text = snapshot.text();
                            let (start_offset, end_offset) = if new_range.head >= new_range.anchor {
                                (new_range.anchor, new_range.head)
                            } else {
                                (new_range.head, new_range.anchor)
                            };
                            let selected_text = text.chars().skip(start_offset).take(end_offset - start_offset).collect::<String>();
                            eprintln!("  selected_text: '{}'", selected_text);
                            let context_start = start_offset.saturating_sub(2);
                            let context_len = (end_offset - start_offset) + 4;
                            eprintln!("  text around selection: '{}'", text.chars().skip(context_start).take(context_len).collect::<String>());
                            
                            // Create selection in Helix style (anchor to head)
                            selection.set_tail(anchor_display, selection.goal);
                            selection.set_head(head_display, selection.goal);
                            
                            eprintln!("DEBUG: After setting selection:");
                            eprintln!("  selection.tail(): {:?}", selection.tail());
                            eprintln!("  selection.head(): {:?}", selection.head());
                            eprintln!("  selection.is_empty(): {}", selection.is_empty());
                        } else {
                            eprintln!("DEBUG: Taking vim motion system branch");
                            // Use existing vim motion system for non-word movements
                            if let Some((mut end_pos, goal)) = motion.move_point(
                                map,
                                start_pos,
                                selection.goal,
                                Some(1),
                                &text_layout_details,
                            ) {
                                // For document motions, Helix expects absolute positions
                                if matches!(motion, Motion::StartOfDocument) {
                                    // Go to absolute beginning of document (row 0, column 0)
                                    end_pos = map.clip_point(DisplayPoint::new(DisplayRow(0), 0), Bias::Left);
                                } else if matches!(motion, Motion::EndOfDocument) {
                                    // Go to absolute end of document (last character of content)
                                    let max_pos = map.max_point();
                                    end_pos = editor::movement::left(map, max_pos);
                                }
                                
                                // Create selection from start to end position
                                // In helix, cursor is at the end of the selection
                                selection.set_tail(start_pos, selection.goal);
                                selection.set_head(end_pos, goal);
                            }
                        }
                    });
                });
            });
        }
    }




}