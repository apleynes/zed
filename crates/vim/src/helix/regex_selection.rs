use editor::{Editor, scroll::Autoscroll};
use gpui::{Window, Context, actions, Entity, App, WeakEntity, EventEmitter, FocusHandle, Focusable, Render, IntoElement, ParentElement, Styled};
use crate::{Vim, helix::selections};
use regex::Regex;
use language::Point;
use ui::{prelude::*, Button, ButtonStyle, Label, v_flex, h_flex};
use workspace::ModalView;

actions!(
    regex_selection,
    [
        SelectRegex,
        SplitSelectionOnRegex,
        KeepSelections,
        RemoveSelections,
        ConfirmRegexSelection,
        CancelRegexSelection,
    ]
);

pub fn register(editor: &mut Editor, cx: &mut Context<Vim>) {
    Vim::action(editor, cx, helix_select_regex);
    Vim::action(editor, cx, helix_split_selection_on_regex);
    Vim::action(editor, cx, helix_keep_selections);
    Vim::action(editor, cx, helix_remove_selections);
}

fn helix_select_regex(
    vim: &mut Vim,
    _: &SelectRegex,
    window: &mut Window,
    cx: &mut Context<Vim>,
) {
    let Some(workspace) = vim.workspace(window) else {
        return;
    };
    
    let editor = vim.editor.clone();
    
    // Create interactive regex prompt with real-time preview
    workspace.update(cx, |workspace, cx| {
        workspace.toggle_modal(window, cx, |window, cx| {
            InteractiveRegexPrompt::new(
                editor,
                RegexOperation::Select,
                "Select regex matches:".to_string(),
                window,
                cx,
            )
        });
    });
}

fn helix_split_selection_on_regex(
    vim: &mut Vim,
    _: &SplitSelectionOnRegex,
    window: &mut Window,
    cx: &mut Context<Vim>,
) {
    let Some(workspace) = vim.workspace(window) else {
        return;
    };
    
    let editor = vim.editor.clone();
    
    workspace.update(cx, |workspace, cx| {
        workspace.toggle_modal(window, cx, |window, cx| {
            InteractiveRegexPrompt::new(
                editor,
                RegexOperation::Split,
                "Split selections on regex:".to_string(),
                window,
                cx,
            )
        });
    });
}

fn helix_keep_selections(
    vim: &mut Vim,
    _: &KeepSelections,
    window: &mut Window,
    cx: &mut Context<Vim>,
) {
    let Some(workspace) = vim.workspace(window) else {
        return;
    };
    
    let editor = vim.editor.clone();
    
    workspace.update(cx, |workspace, cx| {
        workspace.toggle_modal(window, cx, |window, cx| {
            InteractiveRegexPrompt::new(
                editor,
                RegexOperation::Keep,
                "Keep selections matching regex:".to_string(),
                window,
                cx,
            )
        });
    });
}

fn helix_remove_selections(
    vim: &mut Vim,
    _: &RemoveSelections,
    window: &mut Window,
    cx: &mut Context<Vim>,
) {
    let Some(workspace) = vim.workspace(window) else {
        return;
    };
    
    let editor = vim.editor.clone();
    
    workspace.update(cx, |workspace, cx| {
        workspace.toggle_modal(window, cx, |window, cx| {
            InteractiveRegexPrompt::new(
                editor,
                RegexOperation::Remove,
                "Remove selections matching regex:".to_string(),
                window,
                cx,
            )
        });
    });
}

#[derive(Clone, Copy)]
enum RegexOperation {
    Select,
    Split,
    Keep,
    Remove,
}

fn apply_regex_selection(
    editor: WeakEntity<Editor>,
    pattern: &str,
    operation: RegexOperation,
    window: &mut Window,
    cx: &mut App,
) {
    let Ok(regex) = Regex::new(pattern) else {
        return;
    };

    let _ = editor.update(cx, |editor, cx| {
        let buffer = editor.buffer().read(cx).snapshot(cx);
        let selections = editor.selections.all_adjusted(cx);
        let mut new_ranges = Vec::new();

        match operation {
            RegexOperation::Select => {
                // Select all regex matches within current selections (like Helix select_on_matches)
                for selection in selections {
                    let selection_text = buffer.text_for_range(selection.range()).collect::<String>();
                    let selection_start_offset = editor::ToOffset::to_offset(&selection.start, &buffer);

                    for mat in regex.find_iter(&selection_text) {
                        let start_offset = selection_start_offset + mat.start();
                        let end_offset = selection_start_offset + mat.end();
                        
                        // Convert back to points for range creation
                        let start_point = buffer.offset_to_point(start_offset);
                        let end_point = buffer.offset_to_point(end_offset);
                        new_ranges.push(start_point..end_point);
                    }
                }
            }
            
            RegexOperation::Split => {
                // Split selections on regex matches (like Helix split_on_matches)
                for selection in selections {
                    let selection_text = buffer.text_for_range(selection.range()).collect::<String>();
                    let selection_start_offset = editor::ToOffset::to_offset(&selection.start, &buffer);
                    
                    // Handle zero-width selections
                    if selection.start == selection.end {
                        new_ranges.push(selection.range());
                        continue;
                    }
                    
                    let mut last_end = 0;
                    
                    for mat in regex.find_iter(&selection_text) {
                        // Add text before the match (including empty strings for leading matches)
                        let start_offset = selection_start_offset + last_end;
                        let end_offset = selection_start_offset + mat.start();
                        let start_point = buffer.offset_to_point(start_offset);
                        let end_point = buffer.offset_to_point(end_offset);
                        new_ranges.push(start_point..end_point);
                        
                        last_end = mat.end();
                    }
                    
                    // Add remaining text after last match
                    if last_end <= selection_text.len() {
                        let start_offset = selection_start_offset + last_end;
                        let start_point = buffer.offset_to_point(start_offset);
                        new_ranges.push(start_point..selection.end);
                    }
                }
            }
            
            RegexOperation::Keep => {
                // Keep only selections that match the regex (like Helix keep_or_remove_matches with remove=false)
                for selection in selections {
                    let selection_text = buffer.text_for_range(selection.range()).collect::<String>();
                    if regex.is_match(&selection_text) {
                        new_ranges.push(selection.range());
                    }
                }
            }
            
            RegexOperation::Remove => {
                // Remove selections that match the regex (like Helix keep_or_remove_matches with remove=true)
                for selection in selections {
                    let selection_text = buffer.text_for_range(selection.range()).collect::<String>();
                    if !regex.is_match(&selection_text) {
                        new_ranges.push(selection.range());
                    }
                }
            }
        }

        if !new_ranges.is_empty() {
            // Reset primary index since we're creating new selections (like Helix behavior)
            selections::reset_primary_selection_index();
            
            editor.change_selections(Some(Autoscroll::fit()), window, cx, |s| {
                s.select_ranges(new_ranges);
            });
        }
    });
}

// Interactive regex prompt with real-time preview using Zed's search infrastructure pattern
pub struct InteractiveRegexPrompt {
    editor: WeakEntity<Editor>,
    operation: RegexOperation,
    prompt_message: String,
    original_selections: Vec<std::ops::Range<Point>>,
    regex_editor: Entity<Editor>,
    focus_handle: FocusHandle,
}

impl InteractiveRegexPrompt {
    pub fn new(
        editor: WeakEntity<Editor>,
        operation: RegexOperation,
        prompt_message: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        // Store original selections for restoration on cancel
        let original_selections = editor.update(cx, |editor, cx| {
            editor.selections.all_adjusted(cx).iter().map(|s| s.range()).collect()
        }).unwrap_or_default();
        
        let regex_editor = cx.new(|cx| {
            let mut editor = Editor::single_line(window, cx);
            editor.set_placeholder_text("Enter regex pattern (e.g., \\s+ for whitespace, \\w+ for words)...", cx);
            editor
        });
        
        let focus_handle = cx.focus_handle();
        
        // Subscribe to regex editor changes for real-time preview
        cx.subscribe_in(&regex_editor, window, Self::on_regex_editor_event).detach();
        
        // Focus the regex editor when the prompt is created
        regex_editor.update(cx, |editor, cx| {
            editor.focus_handle(cx).focus(window);
        });
        
        Self {
            editor,
            operation,
            prompt_message,
            original_selections,
            regex_editor,
            focus_handle,
        }
    }
    
    fn on_regex_editor_event(
        &mut self,
        _: &Entity<Editor>,
        event: &editor::EditorEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event {
            editor::EditorEvent::Edited { .. } => {
                // Real-time preview update as user types (like BufferSearchBar)
                let pattern = self.regex_editor.read(cx).text(cx);
                self.update_preview(&pattern, window, cx);
            }
            _ => {}
        }
    }
    
    fn update_preview(&mut self, pattern: &str, window: &mut Window, cx: &mut Context<Self>) {
        if pattern.is_empty() {
            // Restore original selections if pattern is empty
            let _ = self.editor.update(cx, |editor, cx| {
                editor.change_selections(Some(Autoscroll::fit()), window, cx, |s| {
                    s.select_ranges(self.original_selections.clone());
                });
            });
            return;
        }
        
        let Ok(regex) = Regex::new(pattern) else {
            return; // Invalid regex, keep current state
        };
        
        let _ = self.editor.update(cx, |editor, cx| {
            let buffer = editor.buffer().read(cx).snapshot(cx);
            let mut new_ranges = Vec::new();
            
            // Apply the operation to original selections for preview
            for selection_range in &self.original_selections {
                let selection_text = buffer.text_for_range(selection_range.clone()).collect::<String>();
                let selection_start_offset = buffer.point_to_offset(selection_range.start);
                
                match self.operation {
                    RegexOperation::Select => {
                        for mat in regex.find_iter(&selection_text) {
                            let start_offset = selection_start_offset + mat.start();
                            let end_offset = selection_start_offset + mat.end();
                            let start_point = buffer.offset_to_point(start_offset);
                            let end_point = buffer.offset_to_point(end_offset);
                            new_ranges.push(start_point..end_point);
                        }
                    }
                    
                    RegexOperation::Split => {
                        if selection_range.start == selection_range.end {
                            new_ranges.push(selection_range.clone());
                            continue;
                        }
                        
                        let mut last_end = 0;
                        for mat in regex.find_iter(&selection_text) {
                            // Add text before the match (including empty strings for leading matches)
                            let start_offset = selection_start_offset + last_end;
                            let end_offset = selection_start_offset + mat.start();
                            let start_point = buffer.offset_to_point(start_offset);
                            let end_point = buffer.offset_to_point(end_offset);
                            new_ranges.push(start_point..end_point);
                            
                            last_end = mat.end();
                        }
                        
                        if last_end <= selection_text.len() {
                            let start_offset = selection_start_offset + last_end;
                            let start_point = buffer.offset_to_point(start_offset);
                            new_ranges.push(start_point..selection_range.end);
                        }
                    }
                    
                    RegexOperation::Keep => {
                        if regex.is_match(&selection_text) {
                            new_ranges.push(selection_range.clone());
                        }
                    }
                    
                    RegexOperation::Remove => {
                        if !regex.is_match(&selection_text) {
                            new_ranges.push(selection_range.clone());
                        }
                    }
                }
            }
            
            if !new_ranges.is_empty() {
                editor.change_selections(Some(Autoscroll::fit()), window, cx, |s| {
                    s.select_ranges(new_ranges);
                });
            }
        });
    }
    
    fn confirm(&mut self, _: &ConfirmRegexSelection, window: &mut Window, cx: &mut Context<Self>) {
        let pattern = self.regex_editor.read(cx).text(cx);
        if !pattern.trim().is_empty() {
            apply_regex_selection(self.editor.clone(), &pattern, self.operation, window, cx);
        }
        cx.emit(gpui::DismissEvent);
    }
    
    fn cancel(&mut self, _: &CancelRegexSelection, window: &mut Window, cx: &mut Context<Self>) {
        // Restore original selections
        let _ = self.editor.update(cx, |editor, cx| {
            editor.change_selections(Some(Autoscroll::fit()), window, cx, |s| {
                s.select_ranges(self.original_selections.clone());
            });
        });
        cx.emit(gpui::DismissEvent);
    }
}

impl EventEmitter<gpui::DismissEvent> for InteractiveRegexPrompt {}

impl Focusable for InteractiveRegexPrompt {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl ModalView for InteractiveRegexPrompt {}

impl Render for InteractiveRegexPrompt {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let focus_handle = self.focus_handle.clone();
        
        // Focus the regex editor when rendering
        self.regex_editor.update(cx, |editor, cx| {
            editor.focus_handle(cx).focus(window);
        });

        div()
            .absolute()
            .bottom_4()
            .right_4()
            .child(
                v_flex()
                    .key_context("InteractiveRegexPrompt")
                    .on_action(cx.listener(Self::confirm))
                    .on_action(cx.listener(Self::cancel))
                    .track_focus(&focus_handle)
                    .elevation_3(cx)
                    .w(rems(34.))
                    .child(
                        v_flex()
                            .gap_2()
                            .p_4()
                            .child(
                                Label::new(self.prompt_message.clone())
                                    .size(LabelSize::Default)
                            )
                            .child(
                                Label::new("Tip: Use \\s for whitespace, \\w for word chars, \\d for digits")
                                    .size(LabelSize::Small)
                                    .color(Color::Muted)
                            )
                            .child(
                                self.regex_editor.clone()
                            )
                            .child(
                                h_flex()
                                    .gap_2()
                                    .justify_end()
                                    .child(
                                        Button::new("cancel", "Cancel")
                                            .style(ButtonStyle::Filled)
                                            .on_click(cx.listener(|this, _, window, cx| {
                                                this.cancel(&CancelRegexSelection, window, cx);
                                            }))
                                    )
                                    .child(
                                        Button::new("confirm", "Confirm")
                                            .style(ButtonStyle::Filled)
                                            .on_click(cx.listener(|this, _, window, cx| {
                                                this.confirm(&ConfirmRegexSelection, window, cx);
                                            }))
                                    )
                            )
                    )
            )
    }
} 