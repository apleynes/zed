use crate::{motion::Motion, Vim};
use editor::Editor;
use gpui::{actions, Context, Window};

actions!(
    helix_movement,
    [
        // Basic cursor movements (normal mode)
        MoveCharLeft,
        MoveCharRight,
        MoveLineUp,
        MoveLineDown,
        MoveNextWordStart,
        MovePrevWordStart,
        MoveNextWordEnd,
        MoveStartOfLine,
        MoveEndOfLine,
        MoveFirstNonWhitespace,
        MoveStartOfDocument,
        MoveEndOfDocument,
        
        // Selection extending movements (select mode)
        ExtendCharLeft,
        ExtendCharRight,
        ExtendLineUp,
        ExtendLineDown,
        ExtendNextWordStart,
        ExtendPrevWordStart,
        ExtendNextWordEnd,
        ExtendStartOfLine,
        ExtendEndOfLine,
        ExtendFirstNonWhitespace,
        ExtendStartOfDocument,
        ExtendEndOfDocument,
    ]
);

pub fn register(editor: &mut Editor, cx: &mut Context<Vim>) {
    // Register normal cursor movements
    Vim::action(editor, cx, |vim, _: &MoveCharLeft, window, cx| {
        vim.helix_move_cursor(Motion::Left, false, window, cx);
    });
    
    Vim::action(editor, cx, |vim, _: &MoveCharRight, window, cx| {
        vim.helix_move_cursor(Motion::Right, false, window, cx);
    });
    
    Vim::action(editor, cx, |vim, _: &MoveLineUp, window, cx| {
        vim.helix_move_cursor(Motion::Up { display_lines: true }, false, window, cx);
    });
    
    Vim::action(editor, cx, |vim, _: &MoveLineDown, window, cx| {
        vim.helix_move_cursor(Motion::Down { display_lines: true }, false, window, cx);
    });
    
    Vim::action(editor, cx, |vim, _: &MoveNextWordStart, window, cx| {
        vim.helix_move_cursor(Motion::NextWordStart { ignore_punctuation: false }, false, window, cx);
    });
    
    Vim::action(editor, cx, |vim, _: &MovePrevWordStart, window, cx| {
        vim.helix_move_cursor(Motion::PreviousWordStart { ignore_punctuation: false }, false, window, cx);
    });
    
    Vim::action(editor, cx, |vim, _: &MoveNextWordEnd, window, cx| {
        vim.helix_move_cursor(Motion::NextWordEnd { ignore_punctuation: false }, false, window, cx);
    });
    
    Vim::action(editor, cx, |vim, _: &MoveStartOfLine, window, cx| {
        vim.helix_move_cursor(Motion::StartOfLine { display_lines: true }, false, window, cx);
    });
    
    Vim::action(editor, cx, |vim, _: &MoveEndOfLine, window, cx| {
        vim.helix_move_cursor(Motion::EndOfLine { display_lines: true }, false, window, cx);
    });
    
    Vim::action(editor, cx, |vim, _: &MoveFirstNonWhitespace, window, cx| {
        vim.helix_move_cursor(Motion::FirstNonWhitespace { display_lines: true }, false, window, cx);
    });
    
    Vim::action(editor, cx, |vim, _: &MoveStartOfDocument, window, cx| {
        vim.helix_move_cursor(Motion::StartOfDocument, false, window, cx);
    });
    
    Vim::action(editor, cx, |vim, _: &MoveEndOfDocument, window, cx| {
        vim.helix_move_cursor(Motion::EndOfDocument, false, window, cx);
    });
    
    // Register selection extending movements  
    Vim::action(editor, cx, |vim, _: &ExtendCharLeft, window, cx| {
        vim.helix_move_cursor(Motion::Left, true, window, cx);
    });
    
    Vim::action(editor, cx, |vim, _: &ExtendCharRight, window, cx| {
        vim.helix_move_cursor(Motion::Right, true, window, cx);
    });
    
    Vim::action(editor, cx, |vim, _: &ExtendLineUp, window, cx| {
        vim.helix_move_cursor(Motion::Up { display_lines: true }, true, window, cx);
    });
    
    Vim::action(editor, cx, |vim, _: &ExtendLineDown, window, cx| {
        vim.helix_move_cursor(Motion::Down { display_lines: true }, true, window, cx);
    });
    
    Vim::action(editor, cx, |vim, _: &ExtendNextWordStart, window, cx| {
        vim.helix_move_cursor(Motion::NextWordStart { ignore_punctuation: false }, true, window, cx);
    });
    
    Vim::action(editor, cx, |vim, _: &ExtendPrevWordStart, window, cx| {
        vim.helix_move_cursor(Motion::PreviousWordStart { ignore_punctuation: false }, true, window, cx);
    });
    
    Vim::action(editor, cx, |vim, _: &ExtendNextWordEnd, window, cx| {
        vim.helix_move_cursor(Motion::NextWordEnd { ignore_punctuation: false }, true, window, cx);
    });
    
    Vim::action(editor, cx, |vim, _: &ExtendStartOfLine, window, cx| {
        vim.helix_move_cursor(Motion::StartOfLine { display_lines: true }, true, window, cx);
    });
    
    Vim::action(editor, cx, |vim, _: &ExtendEndOfLine, window, cx| {
        vim.helix_move_cursor(Motion::EndOfLine { display_lines: true }, true, window, cx);
    });
    
    Vim::action(editor, cx, |vim, _: &ExtendFirstNonWhitespace, window, cx| {
        vim.helix_move_cursor(Motion::FirstNonWhitespace { display_lines: true }, true, window, cx);
    });
    
    Vim::action(editor, cx, |vim, _: &ExtendStartOfDocument, window, cx| {
        vim.helix_move_cursor(Motion::StartOfDocument, true, window, cx);
    });
    
    Vim::action(editor, cx, |vim, _: &ExtendEndOfDocument, window, cx| {
        vim.helix_move_cursor(Motion::EndOfDocument, true, window, cx);
    });
}

impl Vim {
    /// Helix-style cursor movement
    /// 
    /// If extend is false: moves cursor (creates new 1-char selection at destination)
    /// If extend is true: extends current selection to destination (for select mode)
    pub fn helix_move_cursor(
        &mut self,
        motion: Motion,
        extend: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if extend {
            // In select mode, use vim's visual motion system to extend selections
            let old_mode = self.mode;
            self.mode = crate::state::Mode::Visual; // Temporarily switch to visual for extension
            self.normal_motion(motion, None, Some(1), false, window, cx);
            self.mode = old_mode; // Restore helix select mode
        } else {
            // In normal mode, use vim's normal motion but ensure we maintain cursor semantics
            self.normal_motion(motion, None, Some(1), false, window, cx);
        }
    }
}