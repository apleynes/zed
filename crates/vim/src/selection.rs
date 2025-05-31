use editor::{Editor, ToPoint, scroll::Autoscroll};
use gpui::{Context, Window, actions};
use language::{Point, SelectionGoal};
use regex::Regex;

use crate::{Vim, regex_prompt::RegexPrompt};
use anyhow;

actions!(
    vim,
    [
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
    ]
);

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
    Vim::action(
        editor,
        cx,
        |vim, _: &MergeConsecutiveSelections, window, cx| {
            vim.merge_consecutive_selections(window, cx);
        },
    );
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
    Vim::action(
        editor,
        cx,
        |vim, _: &CopySelectionOnNextLine, window, cx| {
            vim.copy_selection_on_next_line(window, cx);
        },
    );
    Vim::action(
        editor,
        cx,
        |vim, _: &CopySelectionOnPrevLine, window, cx| {
            vim.copy_selection_on_prev_line(window, cx);
        },
    );
    Vim::action(
        editor,
        cx,
        |vim, _: &RotateSelectionsBackward, window, cx| {
            vim.rotate_selections_backward(window, cx);
        },
    );
    Vim::action(
        editor,
        cx,
        |vim, _: &RotateSelectionsForward, window, cx| {
            vim.rotate_selections_forward(window, cx);
        },
    );
    Vim::action(
        editor,
        cx,
        |vim, _: &RotateSelectionContentsBackward, window, cx| {
            vim.rotate_selection_contents_backward(window, cx);
        },
    );
    Vim::action(
        editor,
        cx,
        |vim, _: &RotateSelectionContentsForward, window, cx| {
            vim.rotate_selection_contents_forward(window, cx);
        },
    );
    Vim::action(editor, cx, |vim, _: &KeepSelections, window, cx| {
        vim.keep_selections(window, cx);
    });
    Vim::action(editor, cx, |vim, _: &RemoveSelections, window, cx| {
        vim.remove_selections(window, cx);
    });
}

impl Vim {
    fn select_regex(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(workspace) = self.workspace(window) else {
            return;
        };

        let vim = cx.weak_entity();
        workspace.update(cx, |workspace, cx| {
            let rx = RegexPrompt::prompt_for_regex(
                workspace,
                "Select all regex matches".to_string(),
                window,
                cx,
            );

            cx.spawn_in(window, async move |_, cx| {
                if let Ok(Some(pattern)) = rx.await {
                    vim.update_in(cx, |vim, window, cx| {
                        vim.apply_regex_selection(&pattern, window, cx);
                    })?;
                }
                anyhow::Ok(())
            })
            .detach_and_log_err(cx);
        });
    }

    fn split_selection_on_regex(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(workspace) = self.workspace(window) else {
            return;
        };

        let vim = cx.weak_entity();
        workspace.update(cx, |workspace, cx| {
            let rx = RegexPrompt::prompt_for_regex(
                workspace,
                "Split selections on regex matches".to_string(),
                window,
                cx,
            );

            cx.spawn_in(window, async move |_, cx| {
                if let Ok(Some(pattern)) = rx.await {
                    vim.update_in(cx, |vim, window, cx| {
                        vim.apply_split_selection(&pattern, window, cx);
                    })?;
                }
                anyhow::Ok(())
            })
            .detach_and_log_err(cx);
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
                let text = buffer
                    .text_for_range(start_point..end_point)
                    .collect::<String>();
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
                    selection.collapse_to(selection.head(), SelectionGoal::None);
                });
            });
        });
    }

    fn flip_selections(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.update_editor(window, cx, |_, editor, window, cx| {
            editor.change_selections(Some(Autoscroll::fit()), window, cx, |s| {
                s.move_with(|_, selection| {
                    // Manually flip head and tail to avoid rope offset issues
                    let old_head = selection.head();
                    let old_tail = selection.tail();
                    let goal = selection.goal;

                    // Set new head to old tail and new tail to old head
                    selection.set_head(old_tail, goal);
                    selection.set_tail(old_head, goal);
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
            let filtered: Vec<_> = selections
                .into_iter()
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
                    language::Bias::Left,
                );
                let next_end = buffer.clip_point(
                    Point::new(next_row, end_point.column),
                    language::Bias::Right,
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
                    language::Bias::Left,
                );
                let prev_end = buffer.clip_point(
                    Point::new(prev_row, end_point.column),
                    language::Bias::Right,
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
            let ranges: Vec<_> = selections
                .into_iter()
                .map(|sel| sel.start..sel.end)
                .collect();

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
            let ranges: Vec<_> = selections
                .into_iter()
                .map(|sel| sel.start..sel.end)
                .collect();

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
                    buffer
                        .text_for_range(selection.start..selection.end)
                        .collect::<String>()
                })
                .collect();

            // Rotate backward: move first element to end
            if !contents.is_empty() {
                let first_content = contents.remove(0);
                contents.push(first_content);
            }

            // Apply the rotated content to the original selection positions
            let edits: Vec<_> = selections
                .iter()
                .zip(contents.iter())
                .map(|(selection, content)| (selection.start..selection.end, content.clone()))
                .collect();

            if !edits.is_empty() {
                editor.edit(edits, cx);

                // Update selections to cover the new content at original positions
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
                    buffer
                        .text_for_range(selection.start..selection.end)
                        .collect::<String>()
                })
                .collect();

            // Rotate forward: move last element to front
            if !contents.is_empty() {
                let last_content = contents.pop().unwrap();
                contents.insert(0, last_content);
            }

            // Apply the rotated content to the original selection positions
            let edits: Vec<_> = selections
                .iter()
                .zip(contents.iter())
                .map(|(selection, content)| (selection.start..selection.end, content.clone()))
                .collect();

            if !edits.is_empty() {
                editor.edit(edits, cx);

                // Update selections to cover the new content at original positions
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
        let Some(workspace) = self.workspace(window) else {
            return;
        };

        let vim = cx.weak_entity();
        workspace.update(cx, |workspace, cx| {
            let rx = RegexPrompt::prompt_for_regex(
                workspace,
                "Keep selections matching regex".to_string(),
                window,
                cx,
            );

            cx.spawn_in(window, async move |_, cx| {
                if let Ok(Some(pattern)) = rx.await {
                    vim.update_in(cx, |vim, window, cx| {
                        vim.apply_keep_selections(&pattern, window, cx);
                    })?;
                }
                anyhow::Ok(())
            })
            .detach_and_log_err(cx);
        });
    }

    fn remove_selections(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(workspace) = self.workspace(window) else {
            return;
        };

        let vim = cx.weak_entity();
        workspace.update(cx, |workspace, cx| {
            let rx = RegexPrompt::prompt_for_regex(
                workspace,
                "Remove selections matching regex".to_string(),
                window,
                cx,
            );

            cx.spawn_in(window, async move |_, cx| {
                if let Ok(Some(pattern)) = rx.await {
                    vim.update_in(cx, |vim, window, cx| {
                        vim.apply_remove_selections(&pattern, window, cx);
                    })?;
                }
                anyhow::Ok(())
            })
            .detach_and_log_err(cx);
        });
    }

    fn apply_regex_selection(
        &mut self,
        pattern: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Ok(regex) = Regex::new(pattern) else {
            return;
        };

        self.update_editor(window, cx, |_, editor, window, cx| {
            let buffer = editor.buffer().read(cx).snapshot(cx);
            let selections = editor.selections.all_adjusted(cx);
            let mut new_ranges = Vec::new();

            for selection in selections {
                let selection_text = buffer.text_for_range(selection.range()).collect::<String>();
                let selection_start_offset = editor::ToOffset::to_offset(&selection.start, &buffer);

                for mat in regex.find_iter(&selection_text) {
                    let start_offset = selection_start_offset + mat.start();
                    let end_offset = selection_start_offset + mat.end();

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

    fn apply_split_selection(
        &mut self,
        pattern: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Ok(regex) = Regex::new(pattern) else {
            return;
        };

        self.update_editor(window, cx, |_, editor, window, cx| {
            let buffer = editor.buffer().read(cx).snapshot(cx);
            let selections = editor.selections.all_adjusted(cx);
            let mut new_ranges = Vec::new();

            for selection in selections {
                let selection_text = buffer.text_for_range(selection.range()).collect::<String>();
                let selection_start_offset = editor::ToOffset::to_offset(&selection.start, &buffer);
                let mut last_end = 0;

                for mat in regex.find_iter(&selection_text) {
                    // Add text before the match
                    if mat.start() > last_end {
                        let start_offset = selection_start_offset + last_end;
                        let end_offset = selection_start_offset + mat.start();

                        new_ranges.push(start_offset..end_offset);
                    }
                    last_end = mat.end();
                }

                // Add remaining text after last match
                if last_end < selection_text.len() {
                    let start_offset = selection_start_offset + last_end;
                    let end_offset = editor::ToOffset::to_offset(&selection.end, &buffer);

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

    fn apply_keep_selections(
        &mut self,
        pattern: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Ok(regex) = Regex::new(pattern) else {
            return;
        };

        self.update_editor(window, cx, |_, editor, window, cx| {
            let buffer = editor.buffer().read(cx).snapshot(cx);
            let selections = editor.selections.all_adjusted(cx);
            let mut new_ranges = Vec::new();

            for selection in selections {
                let selection_text = buffer.text_for_range(selection.range()).collect::<String>();
                if regex.is_match(&selection_text) {
                    new_ranges.push(selection.range());
                }
            }

            if !new_ranges.is_empty() {
                editor.change_selections(Some(Autoscroll::fit()), window, cx, |s| {
                    s.select_ranges(new_ranges);
                });
            }
        });
    }

    fn apply_remove_selections(
        &mut self,
        pattern: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Ok(regex) = Regex::new(pattern) else {
            return;
        };

        self.update_editor(window, cx, |_, editor, window, cx| {
            let buffer = editor.buffer().read(cx).snapshot(cx);
            let selections = editor.selections.all_adjusted(cx);
            let mut new_ranges = Vec::new();

            for selection in selections {
                let selection_text = buffer.text_for_range(selection.range()).collect::<String>();
                if !regex.is_match(&selection_text) {
                    new_ranges.push(selection.range());
                }
            }

            if !new_ranges.is_empty() {
                editor.change_selections(Some(Autoscroll::fit()), window, cx, |s| {
                    s.select_ranges(new_ranges);
                });
            }
        });
    }
}

#[cfg(test)]
mod test {
    use indoc::indoc;

    use crate::{state::Mode, test::VimTestContext, VimAddon};

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

    // #[gpui::test]
    // async fn test_rotate_selection_contents_comprehensive(cx: &mut gpui::TestAppContext) {
    //     let mut cx = VimTestContext::new(cx, true).await;
    //
    //     // Test with three selections with different lengths to verify selection positioning
    //     cx.set_state(
    //         indoc! {"
    //         start «oneˇ» middle «twoˇ» end «threeˇ» finish"},
    //         Mode::HelixNormal,
    //     );

    //     // Rotate forward: one->two->three->one
    //     // Content "three" should move to first position
    //     // Content "one" should move to second position
    //     // Content "two" should move to third position
    //     cx.simulate_keystrokes("alt-)");

    //     cx.assert_state(
    //         indoc! {"
    //         start «threeˇ» middle «oneˇ» end «twoˇ» finish"},
    //         Mode::HelixNormal,
    //     );

    //     // Rotate backward to verify it works in reverse
    //     cx.simulate_keystrokes("alt-(");

    //     cx.assert_state(
    //         indoc! {"
    //         start «oneˇ» middle «twoˇ» end «threeˇ» finish"},
    //         Mode::HelixNormal,
    //     );
    // }

    #[gpui::test]
    async fn test_select_regex(cx: &mut gpui::TestAppContext) {
        let mut cx = VimTestContext::new(cx, true).await;

        // Test selecting all words within a selection using word pattern
        cx.set_state(
            indoc! {"
            «hello world testˇ»
            other line"},
            Mode::HelixNormal,
        );

        // Directly call apply_regex_selection with word pattern
        let vim = cx.update_editor(|editor, _window, _cx| editor.addon::<VimAddon>().cloned().unwrap());
        cx.update(|window, cx| {
            vim.entity.update(cx, |vim, cx| {
                vim.apply_regex_selection(r"\b\w+\b", window, cx);
            });
        });

        // Should select each word within the original selection
        cx.assert_state(
            indoc! {"
            «helloˇ» «worldˇ» «testˇ»
            other line"},
            Mode::HelixNormal,
        );
    }

    #[gpui::test]
    async fn test_split_selection_on_regex(cx: &mut gpui::TestAppContext) {
        let mut cx = VimTestContext::new(cx, true).await;

        // Test splitting selection on whitespace
        cx.set_state(
            indoc! {"
            «hello world testˇ»
            other line"},
            Mode::HelixNormal,
        );

        // Directly call apply_split_selection with whitespace pattern
        let vim = cx.update_editor(|editor, _window, _cx| editor.addon::<VimAddon>().cloned().unwrap());
        cx.update(|window, cx| {
            vim.entity.update(cx, |vim, cx| {
                vim.apply_split_selection(r"\s+", window, cx);
            });
        });

        // Should split on whitespace, creating selections for the non-whitespace parts
        cx.assert_state(
            indoc! {"
            «helloˇ» «worldˇ» «testˇ»
            other line"},
            Mode::HelixNormal,
        );
    }

    #[gpui::test]
    async fn test_keep_selections(cx: &mut gpui::TestAppContext) {
        let mut cx = VimTestContext::new(cx, true).await;

        // Create multiple selections, some matching pattern, some not
        cx.set_state(
            indoc! {"
            «helloˇ» «123ˇ» «worldˇ» «456ˇ»
            other line"},
            Mode::HelixNormal,
        );

        // Directly call apply_keep_selections with word pattern (alphabetic only)
        let vim = cx.update_editor(|editor, _window, _cx| editor.addon::<VimAddon>().cloned().unwrap());
        cx.update(|window, cx| {
            vim.entity.update(cx, |vim, cx| {
                vim.apply_keep_selections(r"^[a-zA-Z]+$", window, cx);
            });
        });

        // Should keep only selections that match the pattern (alphabetic words)
        cx.assert_state(
            indoc! {"
            «helloˇ» 123 «worldˇ» 456
            other line"},
            Mode::HelixNormal,
        );
    }

    #[gpui::test]
    async fn test_remove_selections(cx: &mut gpui::TestAppContext) {
        let mut cx = VimTestContext::new(cx, true).await;

        // Create multiple selections, some matching pattern, some not
        cx.set_state(
            indoc! {"
            «helloˇ» «123ˇ» «worldˇ» «456ˇ»
            other line"},
            Mode::HelixNormal,
        );

        // Directly call apply_remove_selections with numeric pattern
        let vim = cx.update_editor(|editor, _window, _cx| editor.addon::<VimAddon>().cloned().unwrap());
        cx.update(|window, cx| {
            vim.entity.update(cx, |vim, cx| {
                vim.apply_remove_selections(r"^\d+$", window, cx);
            });
        });

        // Should remove selections that match the pattern (numeric only)
        cx.assert_state(
            indoc! {"
            «helloˇ» 123 «worldˇ» 456
            other line"},
            Mode::HelixNormal,
        );
    }

    #[gpui::test]
    async fn test_select_literal_space(cx: &mut gpui::TestAppContext) {
        let mut cx = VimTestContext::new(cx, true).await;

        // Test selecting literal spaces within a selection
        cx.set_state(
            indoc! {"
            «hello world test hereˇ»
            other line"},
            Mode::HelixNormal,
        );

        // Directly call apply_regex_selection with literal space pattern
        let vim = cx.update_editor(|editor, _window, _cx| editor.addon::<VimAddon>().cloned().unwrap());
        cx.update(|window, cx| {
            vim.entity.update(cx, |vim, cx| {
                vim.apply_regex_selection(r" ", window, cx);
            });
        });

        // Should select each space character within the original selection
        cx.assert_state(
            indoc! {"
            hello« ˇ»world« ˇ»test« ˇ»here
            other line"},
            Mode::HelixNormal,
        );
    }

    #[gpui::test]
    async fn test_select_spaces_comprehensive(cx: &mut gpui::TestAppContext) {
        let mut cx = VimTestContext::new(cx, true).await;

        // Test with text that has various space scenarios
        cx.set_state(
            indoc! {"
            «  hello   world  ˇ»
            other line"},
            Mode::HelixNormal,
        );

        // Test literal space pattern
        let vim = cx.update_editor(|editor, _window, _cx| editor.addon::<VimAddon>().cloned().unwrap());
        cx.update(|window, cx| {
            vim.entity.update(cx, |vim, cx| {
                vim.apply_regex_selection(r" ", window, cx);
            });
        });

        // Should select space groups (adjacent spaces get merged by editor)
        cx.assert_state(
            indoc! {"
            «  ˇ»hello«   ˇ»world«  ˇ»
            other line"},
            Mode::HelixNormal,
        );
    }

    #[gpui::test]
    async fn test_select_single_space_only(cx: &mut gpui::TestAppContext) {
        let mut cx = VimTestContext::new(cx, true).await;

        // Test selecting just a single space character
        cx.set_state(
            indoc! {"
            hello« ˇ»world
            other line"},
            Mode::HelixNormal,
        );

        // Test literal space pattern on single space selection
        let vim = cx.update_editor(|editor, _window, _cx| editor.addon::<VimAddon>().cloned().unwrap());
        cx.update(|window, cx| {
            vim.entity.update(cx, |vim, cx| {
                vim.apply_regex_selection(r" ", window, cx);
            });
        });

        // Should still select the space (should not be filtered out)
        cx.assert_state(
            indoc! {"
            hello« ˇ»world
            other line"},
            Mode::HelixNormal,
        );
    }

    #[gpui::test]
    async fn test_select_ranges_simple(cx: &mut gpui::TestAppContext) {
        let mut cx = VimTestContext::new(cx, true).await;

        // Test with non-adjacent matches to verify select_ranges works
        cx.set_state(
            indoc! {"
            «hello test hello testˇ»
            other line"},
            Mode::HelixNormal,
        );

        // Test selecting "hello" pattern
        let vim = cx.update_editor(|editor, _window, _cx| editor.addon::<VimAddon>().cloned().unwrap());
        cx.update(|window, cx| {
            vim.entity.update(cx, |vim, cx| {
                vim.apply_regex_selection(r"hello", window, cx);
            });
        });

        // Should select both instances of "hello"
        cx.assert_state(
            indoc! {"
            «helloˇ» test «helloˇ» test
            other line"},
            Mode::HelixNormal,
        );
    }



    #[gpui::test]
    async fn test_flip_selections(cx: &mut gpui::TestAppContext) {
        let mut cx = VimTestContext::new(cx, true).await;

        // Test flipping selection direction
        cx.set_state("The qu«ick ˇ»brown", Mode::HelixNormal);

        // Flip selection: cursor and anchor should swap
        cx.simulate_keystrokes("alt-;");

        // After flip, the selection direction should be reversed
        cx.assert_state("The qu«ˇick »brown", Mode::HelixNormal);
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
            The quick ˇbrown
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
