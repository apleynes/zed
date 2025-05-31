use editor::{Editor, scroll::Autoscroll, ToPoint};
use gpui::{Context, Window, actions};
use language::{Point, SelectionGoal};
use regex::Regex;

use crate::Vim;

actions!(vim, [
    SelectRegex,
    SplitSelectionOnRegex,
    AlignSelections,
    MergeSelections,
    MergeConsecutiveSelections,
    TrimSelections,
    CollapseSelection,
    FlipSelections,
    KeepPrimarySelection,
    RemovePrimarySelection,
    CopySelectionOnNextLine,
    CopySelectionOnPrevLine,
    RotateSelectionsBackward,
    RotateSelectionsForward,
    RotateSelectionContentsBackward,
    RotateSelectionContentsForward,
    KeepSelections,
    RemoveSelections,
]);

pub(crate) fn register(editor: &mut Editor, cx: &mut Context<Vim>) {
    Vim::action(editor, cx, |vim, _: &SelectRegex, window, cx| {
        vim.select_regex(window, cx);
    });
    Vim::action(editor, cx, |vim, _: &SplitSelectionOnRegex, window, cx| {
        vim.split_selection_on_regex(window, cx);
    });
    Vim::action(editor, cx, |vim, _: &AlignSelections, window, cx| {
        vim.align_selections(window, cx);
    });
    Vim::action(editor, cx, |vim, _: &MergeSelections, window, cx| {
        vim.merge_selections(window, cx);
    });
    Vim::action(editor, cx, |vim, _: &MergeConsecutiveSelections, window, cx| {
        vim.merge_consecutive_selections(window, cx);
    });
    Vim::action(editor, cx, |vim, _: &TrimSelections, window, cx| {
        vim.trim_selections(window, cx);
    });
    Vim::action(editor, cx, |vim, _: &CollapseSelection, window, cx| {
        vim.collapse_selection(window, cx);
    });
    Vim::action(editor, cx, |vim, _: &FlipSelections, window, cx| {
        vim.flip_selections(window, cx);
    });
    Vim::action(editor, cx, |vim, _: &KeepPrimarySelection, window, cx| {
        vim.keep_primary_selection(window, cx);
    });
    Vim::action(editor, cx, |vim, _: &RemovePrimarySelection, window, cx| {
        vim.remove_primary_selection(window, cx);
    });
    Vim::action(editor, cx, |vim, _: &CopySelectionOnNextLine, window, cx| {
        vim.copy_selection_on_next_line(window, cx);
    });
    Vim::action(editor, cx, |vim, _: &CopySelectionOnPrevLine, window, cx| {
        vim.copy_selection_on_prev_line(window, cx);
    });
    Vim::action(editor, cx, |vim, _: &RotateSelectionsBackward, window, cx| {
        vim.rotate_selections_backward(window, cx);
    });
    Vim::action(editor, cx, |vim, _: &RotateSelectionsForward, window, cx| {
        vim.rotate_selections_forward(window, cx);
    });
    Vim::action(editor, cx, |vim, _: &RotateSelectionContentsBackward, window, cx| {
        vim.rotate_selection_contents_backward(window, cx);
    });
    Vim::action(editor, cx, |vim, _: &RotateSelectionContentsForward, window, cx| {
        vim.rotate_selection_contents_forward(window, cx);
    });
    Vim::action(editor, cx, |vim, _: &KeepSelections, window, cx| {
        vim.keep_selections(window, cx);
    });
    Vim::action(editor, cx, |vim, _: &RemoveSelections, window, cx| {
        vim.remove_selections(window, cx);
    });
}

impl Vim {
    fn select_regex(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // For now, use a simple implementation that could be enhanced with proper UI later
        // This demonstrates the core functionality using word boundaries as an example
        let pattern = r"\b\w+\b"; // Example: select all words
        
        self.update_editor(window, cx, |_, editor, window, cx| {
            let buffer = editor.buffer().read(cx).snapshot(cx);
            let selections = editor.selections.all_adjusted(cx);
            
            if let Ok(regex) = Regex::new(pattern) {
                let mut new_ranges = Vec::new();
                
                for selection in &selections {
                    let text = buffer.text_for_range(selection.start..selection.end).collect::<String>();
                    let selection_start_offset = buffer.point_to_offset(selection.start);
                    
                    for match_result in regex.find_iter(&text) {
                        let start_offset = selection_start_offset + match_result.start();
                        let end_offset = selection_start_offset + match_result.end();
                        let start_point = buffer.offset_to_point(start_offset);
                        let end_point = buffer.offset_to_point(end_offset);
                        new_ranges.push(start_point..end_point);
                    }
                }
                
                if !new_ranges.is_empty() {
                    editor.change_selections(Some(Autoscroll::fit()), window, cx, |s| {
                        s.select_ranges(new_ranges);
                    });
                }
            }
        });
    }

    fn split_selection_on_regex(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // For now, use a simple implementation that splits on whitespace as an example
        let pattern = r"\s+"; // Example: split on whitespace
        
        self.update_editor(window, cx, |_, editor, window, cx| {
            let buffer = editor.buffer().read(cx).snapshot(cx);
            let selections = editor.selections.all_adjusted(cx);
            
            if let Ok(regex) = Regex::new(pattern) {
                let mut new_ranges = Vec::new();
                
                for selection in &selections {
                    let text = buffer.text_for_range(selection.start..selection.end).collect::<String>();
                    let selection_start_offset = buffer.point_to_offset(selection.start);
                    
                    // Find split positions within this selection
                    let mut last_end = 0;
                    for match_result in regex.find_iter(&text) {
                        // Add text before the match as a selection
                        if match_result.start() > last_end {
                            let start_offset = selection_start_offset + last_end;
                            let end_offset = selection_start_offset + match_result.start();
                            let start_point = buffer.offset_to_point(start_offset);
                            let end_point = buffer.offset_to_point(end_offset);
                            if start_point < end_point {
                                new_ranges.push(start_point..end_point);
                            }
                        }
                        last_end = match_result.end();
                    }
                    
                    // Add remaining text after last match
                    if last_end < text.len() {
                        let start_offset = selection_start_offset + last_end;
                        let end_offset = selection_start_offset + text.len();
                        let start_point = buffer.offset_to_point(start_offset);
                        let end_point = buffer.offset_to_point(end_offset);
                        if start_point < end_point {
                            new_ranges.push(start_point..end_point);
                        }
                    }
                }
                
                if !new_ranges.is_empty() {
                    editor.change_selections(Some(Autoscroll::fit()), window, cx, |s| {
                        s.select_ranges(new_ranges);
                    });
                }
            }
        });
    }

    fn align_selections(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.update_editor(window, cx, |_, editor, _window, cx| {
            let buffer = editor.buffer().read(cx).snapshot(cx);
            let selections = editor.selections.all_adjusted(cx);
            
            if selections.len() <= 1 {
                return;
            }
            
            // Find the maximum column position across all selections
            let mut max_column = 0;
            let mut selection_points = Vec::new();
            
            for selection in &selections {
                let start_point = selection.start.to_point(&buffer);
                selection_points.push((start_point, selection.end.to_point(&buffer)));
                max_column = max_column.max(start_point.column);
            }
            
            // Calculate new positions and required indentation
            let mut edits = Vec::new();
            
            for (i, (start_point, _end_point)) in selection_points.iter().enumerate() {
                if start_point.column < max_column {
                    let spaces_needed = max_column - start_point.column;
                    let insert_position = selections[i].start;
                    let spaces = " ".repeat(spaces_needed as usize);
                    
                    edits.push((insert_position..insert_position, spaces));
                }
            }
            
            // Apply the edits to align selections
            if !edits.is_empty() {
                editor.edit(edits, cx);
            }
        });
    }

    fn merge_selections(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.update_editor(window, cx, |_, editor, window, cx| {
            let selections = editor.selections.all_adjusted(cx);
            if selections.is_empty() {
                return;
            }

            // Find the overall range of all selections
            let mut min_start = selections[0].start;
            let mut max_end = selections[0].end;

            for selection in &selections {
                if selection.start < min_start {
                    min_start = selection.start;
                }
                if selection.end > max_end {
                    max_end = selection.end;
                }
            }

            editor.change_selections(Some(Autoscroll::fit()), window, cx, |s| {
                s.select_ranges(vec![min_start..max_end]);
            });
        });
    }

    fn merge_consecutive_selections(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.update_editor(window, cx, |_, editor, window, cx| {
            let mut selections = editor.selections.all_adjusted(cx);
            if selections.len() <= 1 {
                return;
            }

            // Sort selections by start position
            selections.sort_by_key(|sel| sel.start);

            let mut merged = Vec::new();
            let mut current_start = selections[0].start;
            let mut current_end = selections[0].end;

            for selection in selections.into_iter().skip(1) {
                // Check if selections are consecutive (touching or overlapping)
                if current_end >= selection.start {
                    // Merge by extending the end
                    current_end = current_end.max(selection.end);
                } else {
                    // Not consecutive, push current and start new
                    merged.push(current_start..current_end);
                    current_start = selection.start;
                    current_end = selection.end;
                }
            }
            merged.push(current_start..current_end);

            editor.change_selections(Some(Autoscroll::fit()), window, cx, |s| {
                s.select_ranges(merged);
            });
        });
    }

    fn trim_selections(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.update_editor(window, cx, |_, editor, window, cx| {
            let buffer = editor.buffer().read(cx).snapshot(cx);
            let selections = editor.selections.all_adjusted(cx);
            
            let mut new_ranges = Vec::new();
            
            for selection in &selections {
                let start_point = selection.start.to_point(&buffer);
                let end_point = selection.end.to_point(&buffer);
                let text = buffer.text_for_range(start_point..end_point).collect::<String>();
                let trimmed_text = text.trim();
                
                if !trimmed_text.is_empty() {
                    // Calculate how much whitespace to trim from start and end
                    let leading_whitespace = text.len() - text.trim_start().len();
                    let trailing_whitespace = text.len() - text.trim_end().len();
                    
                    let selection_start_offset = buffer.point_to_offset(start_point);
                    let new_start_offset = selection_start_offset + leading_whitespace;
                    let new_end_offset = selection_start_offset + text.len() - trailing_whitespace;
                    
                    let new_start = buffer.offset_to_point(new_start_offset);
                    let new_end = buffer.offset_to_point(new_end_offset);
                    
                    if new_start < new_end {
                        new_ranges.push(new_start..new_end);
                    } else {
                        // If somehow the trimmed selection is empty, collapse to cursor
                        new_ranges.push(start_point..start_point);
                    }
                } else {
                    // If trimmed text is empty, collapse selection to cursor
                    new_ranges.push(start_point..start_point);
                }
            }
            
            if !new_ranges.is_empty() {
                editor.change_selections(Some(Autoscroll::fit()), window, cx, |s| {
                    s.select_ranges(new_ranges);
                });
            }
        });
    }

    fn collapse_selection(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.update_editor(window, cx, |_, editor, window, cx| {
            editor.change_selections(Some(Autoscroll::fit()), window, cx, |s| {
                s.move_with(|_, selection| {
                    selection.collapse_to(selection.start, SelectionGoal::None);
                });
            });
        });
    }

    fn flip_selections(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.update_editor(window, cx, |_, editor, window, cx| {
            editor.change_selections(Some(Autoscroll::fit()), window, cx, |s| {
                s.move_with(|_, selection| {
                    selection.swap_head_tail();
                });
            });
        });
    }

    fn keep_primary_selection(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.update_editor(window, cx, |_, editor, window, cx| {
            let primary = editor.selections.newest_adjusted(cx);
            editor.change_selections(Some(Autoscroll::fit()), window, cx, |s| {
                s.select_ranges(vec![primary.start..primary.end]);
            });
        });
    }

    fn remove_primary_selection(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.update_editor(window, cx, |_, editor, window, cx| {
            let selections = editor.selections.all_adjusted(cx);
            if selections.len() <= 1 {
                return; // Can't remove the only selection
            }

            let primary_id = editor.selections.newest_anchor().id;
            let filtered: Vec<_> = selections.into_iter()
                .filter(|sel| sel.id != primary_id)
                .map(|sel| sel.start..sel.end)
                .collect();
            
            editor.change_selections(Some(Autoscroll::fit()), window, cx, |s| {
                s.select_ranges(filtered);
            });
        });
    }

    fn copy_selection_on_next_line(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.update_editor(window, cx, |_, editor, window, cx| {
            let selections = editor.selections.all_adjusted(cx);
            let buffer = editor.buffer().read(cx).snapshot(cx);
            
            let mut new_ranges = Vec::new();
            for selection in &selections {
                let start_point = selection.start.to_point(&buffer);
                let end_point = selection.end.to_point(&buffer);
                new_ranges.push(start_point..end_point);
            }
            
            let primary = editor.selections.newest_adjusted(cx);
            let start_point = primary.start.to_point(&buffer);
            let end_point = primary.end.to_point(&buffer);
            
            // Calculate position on next line
            let next_row = start_point.row + 1;
            let max_row = buffer.max_point().row;
            if next_row <= max_row {
                let next_start = buffer.clip_point(
                    Point::new(next_row, start_point.column),
                    language::Bias::Left
                );
                let next_end = buffer.clip_point(
                    Point::new(next_row, end_point.column),
                    language::Bias::Right
                );
                
                new_ranges.push(next_start..next_end);
                
                editor.change_selections(Some(Autoscroll::fit()), window, cx, |s| {
                    s.select_ranges(new_ranges);
                });
            }
        });
    }

    fn copy_selection_on_prev_line(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.update_editor(window, cx, |_, editor, window, cx| {
            let selections = editor.selections.all_adjusted(cx);
            let buffer = editor.buffer().read(cx).snapshot(cx);
            
            let mut new_ranges = Vec::new();
            for selection in &selections {
                let start_point = selection.start.to_point(&buffer);
                let end_point = selection.end.to_point(&buffer);
                new_ranges.push(start_point..end_point);
            }
            
            let primary = editor.selections.newest_adjusted(cx);
            let start_point = primary.start.to_point(&buffer);
            let end_point = primary.end.to_point(&buffer);
            
            // Calculate position on previous line
            if start_point.row > 0 {
                let prev_row = start_point.row - 1;
                let prev_start = buffer.clip_point(
                    Point::new(prev_row, start_point.column),
                    language::Bias::Left
                );
                let prev_end = buffer.clip_point(
                    Point::new(prev_row, end_point.column),
                    language::Bias::Right
                );
                
                new_ranges.push(prev_start..prev_end);
                
                editor.change_selections(Some(Autoscroll::fit()), window, cx, |s| {
                    s.select_ranges(new_ranges);
                });
            }
        });
    }

    fn rotate_selections_backward(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.update_editor(window, cx, |_, editor, window, cx| {
            let mut selections = editor.selections.all_adjusted(cx);
            if selections.len() <= 1 {
                return;
            }

            // Move the last selection to the front
            let last = selections.pop().unwrap();
            selections.insert(0, last);
            let ranges: Vec<_> = selections.into_iter().map(|sel| sel.start..sel.end).collect();
            
            editor.change_selections(Some(Autoscroll::fit()), window, cx, |s| {
                s.select_ranges(ranges);
            });
        });
    }

    fn rotate_selections_forward(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.update_editor(window, cx, |_, editor, window, cx| {
            let mut selections = editor.selections.all_adjusted(cx);
            if selections.len() <= 1 {
                return;
            }

            // Move the first selection to the end
            let first = selections.remove(0);
            selections.push(first);
            let ranges: Vec<_> = selections.into_iter().map(|sel| sel.start..sel.end).collect();
            
            editor.change_selections(Some(Autoscroll::fit()), window, cx, |s| {
                s.select_ranges(ranges);
            });
        });
    }

    fn rotate_selection_contents_backward(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.update_editor(window, cx, |_, editor, window, cx| {
            let buffer = editor.buffer().read(cx).snapshot(cx);
            let selections = editor.selections.all_adjusted(cx);
            
            if selections.len() <= 1 {
                return;
            }
            
            // Collect the text content from each selection
            let mut contents: Vec<String> = selections
                .iter()
                .map(|selection| {
                    buffer.text_for_range(selection.start..selection.end).collect::<String>()
                })
                .collect();
            
            // Rotate backward: move first element to end
            if !contents.is_empty() {
                let first = contents.remove(0);
                contents.push(first);
            }
            
            // Apply the rotated content to each selection
            let edits: Vec<_> = selections
                .iter()
                .zip(contents.iter())
                .map(|(selection, content)| (selection.start..selection.end, content.clone()))
                .collect();
            
            if !edits.is_empty() {
                editor.edit(edits, cx);
                
                // Update selections to cover the new content
                let new_buffer = editor.buffer().read(cx).snapshot(cx);
                let mut new_ranges = Vec::new();
                
                for (selection, content) in selections.iter().zip(contents.iter()) {
                    let start_point = selection.start;
                    let end_offset = new_buffer.point_to_offset(start_point) + content.len();
                    let end_point = new_buffer.offset_to_point(end_offset);
                    new_ranges.push(start_point..end_point);
                }
                
                editor.change_selections(Some(Autoscroll::fit()), window, cx, |s| {
                    s.select_ranges(new_ranges);
                });
            }
        });
    }

    fn rotate_selection_contents_forward(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.update_editor(window, cx, |_, editor, window, cx| {
            let buffer = editor.buffer().read(cx).snapshot(cx);
            let selections = editor.selections.all_adjusted(cx);
            
            if selections.len() <= 1 {
                return;
            }
            
            // Collect the text content from each selection
            let mut contents: Vec<String> = selections
                .iter()
                .map(|selection| {
                    buffer.text_for_range(selection.start..selection.end).collect::<String>()
                })
                .collect();
            
            // Rotate forward: move last element to front
            if !contents.is_empty() {
                let last = contents.pop().unwrap();
                contents.insert(0, last);
            }
            
            // Apply the rotated content to each selection
            let edits: Vec<_> = selections
                .iter()
                .zip(contents.iter())
                .map(|(selection, content)| (selection.start..selection.end, content.clone()))
                .collect();
            
            if !edits.is_empty() {
                editor.edit(edits, cx);
                
                // Update selections to cover the new content
                let new_buffer = editor.buffer().read(cx).snapshot(cx);
                let mut new_ranges = Vec::new();
                
                for (selection, content) in selections.iter().zip(contents.iter()) {
                    let start_point = selection.start;
                    let end_offset = new_buffer.point_to_offset(start_point) + content.len();
                    let end_point = new_buffer.offset_to_point(end_offset);
                    new_ranges.push(start_point..end_point);
                }
                
                editor.change_selections(Some(Autoscroll::fit()), window, cx, |s| {
                    s.select_ranges(new_ranges);
                });
            }
        });
    }

    fn keep_selections(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // For now, use a simple implementation that keeps selections containing letters
        let pattern = r"[a-zA-Z]"; // Example: keep selections that contain letters
        
        self.update_editor(window, cx, |_, editor, window, cx| {
            let buffer = editor.buffer().read(cx).snapshot(cx);
            let selections = editor.selections.all_adjusted(cx);
            
            if let Ok(regex) = Regex::new(pattern) {
                let mut new_ranges = Vec::new();
                
                for selection in &selections {
                    let text = buffer.text_for_range(selection.start..selection.end).collect::<String>();
                    
                    if regex.is_match(&text) {
                        new_ranges.push(selection.start..selection.end);
                    }
                }
                
                if !new_ranges.is_empty() {
                    editor.change_selections(Some(Autoscroll::fit()), window, cx, |s| {
                        s.select_ranges(new_ranges);
                    });
                } else {
                    // If no selections match, keep a single cursor at the first selection
                    if !selections.is_empty() {
                        let first = &selections[0];
                        editor.change_selections(Some(Autoscroll::fit()), window, cx, |s| {
                            s.select_ranges(vec![first.start..first.start]);
                        });
                    }
                }
            }
        });
    }

    fn remove_selections(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // For now, use a simple implementation that removes selections containing only digits
        let pattern = r"^\d+$"; // Example: remove selections that are only digits
        
        self.update_editor(window, cx, |_, editor, window, cx| {
            let buffer = editor.buffer().read(cx).snapshot(cx);
            let selections = editor.selections.all_adjusted(cx);
            
            if let Ok(regex) = Regex::new(pattern) {
                let mut new_ranges = Vec::new();
                
                for selection in &selections {
                    let text = buffer.text_for_range(selection.start..selection.end).collect::<String>();
                    
                    if !regex.is_match(&text) {
                        new_ranges.push(selection.start..selection.end);
                    }
                }
                
                if !new_ranges.is_empty() {
                    editor.change_selections(Some(Autoscroll::fit()), window, cx, |s| {
                        s.select_ranges(new_ranges);
                    });
                } else {
                    // If all selections would be removed, keep a single cursor at the first selection
                    if !selections.is_empty() {
                        let first = &selections[0];
                        editor.change_selections(Some(Autoscroll::fit()), window, cx, |s| {
                            s.select_ranges(vec![first.start..first.start]);
                        });
                    }
                }
            }
        });
    }
}

#[cfg(test)]
mod test {
    use indoc::indoc;

    use crate::{state::Mode, test::VimTestContext};

    #[gpui::test]
    async fn test_trim_selections(cx: &mut gpui::TestAppContext) {
        let mut cx = VimTestContext::new(cx, true).await;
        
        cx.set_state(
            indoc! {"
            The qu«  ick  ˇ»brown
            fox jumps over
            the lazy dog."},
            Mode::HelixNormal,
        );

        cx.simulate_keystrokes("_");

        cx.assert_state(
            indoc! {"
            The qu  «ickˇ»  brown
            fox jumps over
            the lazy dog."},
            Mode::HelixNormal,
        );
    }

    #[gpui::test]
    async fn test_trim_selections_simple(cx: &mut gpui::TestAppContext) {
        let mut cx = VimTestContext::new(cx, true).await;
        
        // Simple test with just whitespace around "abc"
        cx.set_state("«  abc  ˇ»", Mode::HelixNormal);
        cx.simulate_keystrokes("_");
        // After trim, selection should exclude leading/trailing whitespace
        cx.assert_state("  «abcˇ»  ", Mode::HelixNormal);
    }

    #[gpui::test]
    async fn test_align_selections_simple(cx: &mut gpui::TestAppContext) {
        let mut cx = VimTestContext::new(cx, true).await;
        
        // Simple case: two selections at different column positions
        cx.set_state(
            indoc! {"
            a«bcˇ»
            hello«worldˇ»"},
            Mode::HelixNormal,
        );

        cx.simulate_keystrokes("&");

        // After alignment, both selections should start at the same column
        cx.assert_state(
            indoc! {"
            a    «bcˇ»
            hello«worldˇ»"},
            Mode::HelixNormal,
        );
    }

    #[gpui::test]
    async fn test_rotate_selection_contents_simple(cx: &mut gpui::TestAppContext) {
        let mut cx = VimTestContext::new(cx, true).await;
        
        // Simple test with just two selections
        cx.set_state("«aˇ» «bˇ»", Mode::HelixNormal);

        // Rotate forward: a->b, b->a
        cx.simulate_keystrokes("alt-)");

        cx.assert_state("«bˇ» «aˇ»", Mode::HelixNormal);
    }



    #[gpui::test]
    async fn test_collapse_selection(cx: &mut gpui::TestAppContext) {
        let mut cx = VimTestContext::new(cx, true).await;
        
        cx.set_state(
            indoc! {"
            The qu«ick ˇ»brown
            fox jumps over
            the lazy dog."},
            Mode::HelixNormal,
        );

        cx.simulate_keystrokes(";");

        cx.assert_state(
            indoc! {"
            The quˇick brown
            fox jumps over
            the lazy dog."},
            Mode::HelixNormal,
        );
    }

    #[gpui::test]
    async fn test_merge_selections(cx: &mut gpui::TestAppContext) {
        let mut cx = VimTestContext::new(cx, true).await;
        
        cx.set_state(
            indoc! {"
            The qu«ickˇ» br«ownˇ»
            fox jumps over
            the lazy dog."},
            Mode::HelixNormal,
        );

        cx.simulate_keystrokes("alt-minus");

        cx.assert_state(
            indoc! {"
            The qu«ick brownˇ»
            fox jumps over
            the lazy dog."},
            Mode::HelixNormal,
        );
    }
}