use crate::{state::Mode, Vim};
use editor::Editor;
use gpui::{actions, Context, Window};

actions!(
    helix_mode,
    [
        EnterSelectMode,
        ExitSelectMode,
        ToggleSelectMode,
    ]
);

pub fn register(editor: &mut Editor, cx: &mut Context<Vim>) {
    Vim::action(editor, cx, |vim, _: &EnterSelectMode, window, cx| {
        vim.enter_helix_select_mode(window, cx);
    });
    
    Vim::action(editor, cx, |vim, _: &ExitSelectMode, window, cx| {
        vim.exit_helix_select_mode(window, cx);
    });
    
    Vim::action(editor, cx, |vim, _: &ToggleSelectMode, window, cx| {
        vim.toggle_helix_select_mode(window, cx);
    });
}

impl Vim {
    pub fn enter_helix_select_mode(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.mode == Mode::HelixNormal {
            self.switch_mode(Mode::HelixSelect, false, window, cx);
        }
    }
    
    pub fn exit_helix_select_mode(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.mode == Mode::HelixSelect {
            self.switch_mode(Mode::HelixNormal, false, window, cx);
            
            // Collapse selections to cursor position (left edge)
            self.update_editor(window, cx, |_, editor, window, cx| {
                editor.change_selections(None, window, cx, |s| {
                    s.move_with(|_map, selection| {
                        // In helix, cursor is positioned at left edge of selection
                        let cursor_pos = if selection.head() > selection.tail() {
                            // Forward selection: cursor at tail (left edge)
                            selection.tail()
                        } else {
                            // Backward selection: cursor at head (left edge)  
                            selection.head()
                        };
                        selection.collapse_to(cursor_pos, selection.goal);
                    });
                });
            });
        }
    }
    
    pub fn toggle_helix_select_mode(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        match self.mode {
            Mode::HelixNormal => self.enter_helix_select_mode(window, cx),
            Mode::HelixSelect => self.exit_helix_select_mode(window, cx),
            _ => {
                // If we're in a different mode, do nothing or switch to helix normal first
            }
        }
    }
    
    pub fn is_helix_mode(&self) -> bool {
        matches!(self.mode, Mode::HelixNormal | Mode::HelixSelect)
    }
    
    pub fn is_helix_select_mode(&self) -> bool {
        self.mode == Mode::HelixSelect
    }
}