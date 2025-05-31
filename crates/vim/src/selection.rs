use editor::{Editor, scroll::Autoscroll, ToPoint};
use gpui::{Context, Window, actions};
use language::{Point, SelectionGoal};

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
    fn select_regex(&mut self, _window: &mut Window, _cx: &mut Context<Self>) {
        // TODO: Implement regex selection - requires prompt for regex pattern
        // This would select all regex matches within current selections
    }

    fn split_selection_on_regex(&mut self, _window: &mut Window, _cx: &mut Context<Self>) {
        // TODO: Implement regex splitting - requires prompt for regex pattern
        // This would split current selections on regex matches
    }

    fn align_selections(&mut self, _window: &mut Window, _cx: &mut Context<Self>) {
        // TODO: Implement selection alignment - requires more complex buffer manipulation
        // This would align all selections to the same column position
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

    fn trim_selections(&mut self, _window: &mut Window, _cx: &mut Context<Self>) {
        // TODO: Implement selection trimming - requires complex text manipulation
        // This would trim whitespace from the start and end of each selection
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

    fn rotate_selection_contents_backward(&mut self, _window: &mut Window, _cx: &mut Context<Self>) {
        // TODO: Implement selection content rotation - requires complex text manipulation
        // This would rotate the text content between selections backward
    }

    fn rotate_selection_contents_forward(&mut self, _window: &mut Window, _cx: &mut Context<Self>) {
        // TODO: Implement selection content rotation - requires complex text manipulation
        // This would rotate the text content between selections forward
    }

    fn keep_selections(&mut self, _window: &mut Window, _cx: &mut Context<Self>) {
        // TODO: Implement keep selections matching regex - requires prompt for regex pattern
        // This would keep only selections that match a regex pattern
    }

    fn remove_selections(&mut self, _window: &mut Window, _cx: &mut Context<Self>) {
        // TODO: Implement remove selections matching regex - requires prompt for regex pattern
        // This would remove selections that match a regex pattern
    }
}

#[cfg(test)]
mod test {
    use indoc::indoc;

    use crate::{state::Mode, test::VimTestContext};

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
    async fn test_flip_selections(cx: &mut gpui::TestAppContext) {
        let mut cx = VimTestContext::new(cx, true).await;
        
        cx.set_state(
            indoc! {"
            The qu«ick ˇ»brown
            fox jumps over
            the lazy dog."},
            Mode::HelixNormal,
        );

        cx.simulate_keystrokes("alt-;");

        cx.assert_state(
            indoc! {"
            The qu«ˇick »brown
            fox jumps over
            the lazy dog."},
            Mode::HelixNormal,
        );
    }

    #[gpui::test]
    async fn test_copy_selection_on_next_line(cx: &mut gpui::TestAppContext) {
        let mut cx = VimTestContext::new(cx, true).await;
        
        cx.set_state(
            indoc! {"
            The qu«ick ˇ»brown
            fox jumps over
            the lazy dog."},
            Mode::HelixNormal,
        );

        cx.simulate_keystrokes("shift-c");

        cx.assert_state(
            indoc! {"
            The qu«ick ˇ»brown
            fox ju«mps ˇ»over
            the lazy dog."},
            Mode::HelixNormal,
        );
    }
}