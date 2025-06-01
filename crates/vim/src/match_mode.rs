use editor::{Editor, scroll::Autoscroll};
use gpui::{Window, Context, actions};
use std::time::Instant;
use crate::{
    Vim,
    motion::Motion,
    object::Object,
    state::Mode,
    surrounds::SurroundsType,
};

actions!(
    vim,
    [
        EnterMatchMode,
        ExitMatchMode,
        MatchModeBracket,
        MatchModeSurround,
        MatchModeReplaceSurround,
        MatchModeDeleteSurround,
        MatchModeTextObjectAround,
        MatchModeTextObjectInside,
    ]
);

pub fn register(editor: &mut Editor, cx: &mut Context<Vim>) {
    Vim::action(editor, cx, Vim::enter_match_mode);
    Vim::action(editor, cx, Vim::exit_match_mode_action);
    Vim::action(editor, cx, Vim::match_mode_bracket);
    Vim::action(editor, cx, Vim::match_mode_surround);
    Vim::action(editor, cx, Vim::match_mode_replace_surround);
    Vim::action(editor, cx, Vim::match_mode_delete_surround);
    Vim::action(editor, cx, Vim::match_mode_text_object_around);
    Vim::action(editor, cx, Vim::match_mode_text_object_inside);
}

impl Vim {
    pub fn enter_match_mode(
        &mut self,
        _: &EnterMatchMode,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.mode != Mode::HelixNormal {
            return;
        }
        
        // Set match mode state
        self.match_mode_active = true;
        self.match_mode_timeout = Some(Instant::now());
        
        // Update UI to show match mode indicator
        cx.notify();
    }
    
    pub fn exit_match_mode_action(
        &mut self,
        _: &ExitMatchMode,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.match_mode_active {
            self.exit_match_mode(window, cx);
        }
    }
    
    pub fn match_mode_bracket(
        &mut self,
        _: &MatchModeBracket,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.match_mode_active {
            return;
        }
        
        self.exit_match_mode(window, cx);
        
        // Go to matching bracket using existing functionality
        self.motion(Motion::Matching, window, cx);
    }
    
    pub fn match_mode_surround(
        &mut self,
        _: &MatchModeSurround,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.match_mode_active {
            return;
        }
        
        // Start surround input mode - wait for character
        self.match_mode_awaiting_surround_char = true;
        cx.notify();
    }
    
    pub fn match_mode_replace_surround(
        &mut self,
        _: &MatchModeReplaceSurround,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.match_mode_active {
            return;
        }
        
        // Start replace surround input mode - wait for two characters
        self.match_mode_awaiting_replace_from = true;
        cx.notify();
    }
    
    pub fn match_mode_delete_surround(
        &mut self,
        _: &MatchModeDeleteSurround,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.match_mode_active {
            return;
        }
        
        // Start delete surround input mode - wait for character
        self.match_mode_awaiting_delete_char = true;
        cx.notify();
    }
    
    pub fn match_mode_text_object_around(
        &mut self,
        _: &MatchModeTextObjectAround,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.match_mode_active {
            return;
        }
        
        // Start text object around input mode - wait for object specifier
        self.match_mode_awaiting_text_object = Some(true); // true = around
        cx.notify();
    }
    
    pub fn match_mode_text_object_inside(
        &mut self,
        _: &MatchModeTextObjectInside,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.match_mode_active {
            return;
        }
        
        // Start text object inside input mode - wait for object specifier
        self.match_mode_awaiting_text_object = Some(false); // false = inside
        cx.notify();
    }
    
    pub fn handle_match_mode_input(
        &mut self,
        input: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if !self.match_mode_active {
            return false;
        }
        
        if let Some(ch) = input.chars().next() {
            if self.match_mode_awaiting_surround_char {
                self.handle_surround_input(ch, window, cx);
                return true;
            }
            
            if self.match_mode_awaiting_delete_char {
                self.handle_delete_surround_input(ch, window, cx);
                return true;
            }
            
            if self.match_mode_awaiting_replace_from {
                self.match_mode_replace_from_char = Some(ch);
                self.match_mode_awaiting_replace_from = false;
                self.match_mode_awaiting_replace_to = true;
                return true;
            }
            
            if self.match_mode_awaiting_replace_to {
                if let Some(from_char) = self.match_mode_replace_from_char {
                    self.handle_replace_surround_input(from_char, ch, window, cx);
                }
                return true;
            }
            
            if let Some(around) = self.match_mode_awaiting_text_object {
                if let Some(object) = self.char_to_object(ch) {
                    self.handle_text_object_input(object, around, window, cx);
                }
                return true;
            }
        }
        
        false
    }
    
    fn handle_surround_input(
        &mut self,
        ch: char,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.exit_match_mode(window, cx);
        
        let text = ch.to_string().into();
        self.add_surrounds(text, SurroundsType::Selection, window, cx);
    }
    
    fn handle_delete_surround_input(
        &mut self,
        ch: char,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.exit_match_mode(window, cx);
        
        let text = ch.to_string().into();
        self.delete_surrounds(text, window, cx);
    }
    
    fn handle_replace_surround_input(
        &mut self,
        from_char: char,
        to_char: char,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.exit_match_mode(window, cx);
        
        if let Some(from_object) = self.char_to_surround_object(from_char) {
            let to_text = to_char.to_string().into();
            self.change_surrounds(to_text, from_object, window, cx);
        }
    }
    
    fn handle_text_object_input(
        &mut self,
        object: Object,
        around: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.exit_match_mode(window, cx);
        
        // Select the text object
        self.update_editor(window, cx, |_, editor, window, cx| {
            editor.change_selections(Some(Autoscroll::fit()), window, cx, |s| {
                s.move_with(|map, selection| {
                    object.expand_selection(map, selection, around);
                });
            });
        });
    }
    
    fn char_to_object(&self, ch: char) -> Option<Object> {
        match ch {
            'w' => Some(Object::Word { ignore_punctuation: false }),
            'W' => Some(Object::Word { ignore_punctuation: true }),
            's' => Some(Object::Sentence),
            'p' => Some(Object::Paragraph),
            'a' => Some(Object::Argument),
            'b' => Some(Object::Parentheses),
            'B' => Some(Object::CurlyBrackets),
            'r' => Some(Object::SquareBrackets),
            't' => Some(Object::Tag),
            'f' => Some(Object::Method),
            'c' => Some(Object::Comment),
            'C' => Some(Object::Class),
            '"' => Some(Object::DoubleQuotes),
            '\'' => Some(Object::Quotes),
            '`' => Some(Object::BackQuotes),
            _ => None,
        }
    }
    
    fn char_to_surround_object(&self, ch: char) -> Option<Object> {
        match ch {
            '(' | ')' => Some(Object::Parentheses),
            '{' | '}' => Some(Object::CurlyBrackets),
            '[' | ']' => Some(Object::SquareBrackets),
            '<' | '>' => Some(Object::AngleBrackets),
            '"' => Some(Object::DoubleQuotes),
            '\'' => Some(Object::Quotes),
            '`' => Some(Object::BackQuotes),
            _ => None,
        }
    }
    
    pub fn exit_match_mode(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.match_mode_active = false;
        self.match_mode_timeout = None;
        self.match_mode_awaiting_surround_char = false;
        self.match_mode_awaiting_delete_char = false;
        self.match_mode_awaiting_replace_from = false;
        self.match_mode_awaiting_replace_to = false;
        self.match_mode_replace_from_char = None;
        self.match_mode_awaiting_text_object = None;
        cx.notify();
    }
    
    pub fn check_match_mode_timeout(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(timeout) = self.match_mode_timeout {
            if timeout.elapsed() > std::time::Duration::from_millis(5000) {
                self.exit_match_mode(window, cx);
            }
        }
    }
}

// Module functions called from vim.rs
pub fn check_match_mode_timeout(vim: &mut Vim, window: &mut Window, cx: &mut Context<Vim>) {
    vim.check_match_mode_timeout(window, cx);
}

pub fn exit_match_mode(vim: &mut Vim, window: &mut Window, cx: &mut Context<Vim>) {
    vim.exit_match_mode(window, cx);
}

pub fn handle_match_mode_input(vim: &mut Vim, text: &str, window: &mut Window, cx: &mut Context<Vim>) -> bool {
    if !vim.match_mode_active {
        return false;
    }
    
    // Handle surround character input
    if vim.match_mode_awaiting_surround_char {
        vim.match_mode_awaiting_surround_char = false;
        vim.exit_match_mode(window, cx);
        // TODO: Implement actual surround logic
        return true;
    }
    
    // Handle delete surround character input
    if vim.match_mode_awaiting_delete_char {
        vim.match_mode_awaiting_delete_char = false;
        vim.exit_match_mode(window, cx);
        // TODO: Implement actual delete surround logic
        return true;
    }
    
    // Handle replace surround character input
    if vim.match_mode_awaiting_replace_from {
        vim.match_mode_replace_from_char = text.chars().next();
        vim.match_mode_awaiting_replace_from = false;
        vim.match_mode_awaiting_replace_to = true;
        return true;
    }
    
    if vim.match_mode_awaiting_replace_to {
        vim.match_mode_awaiting_replace_to = false;
        vim.exit_match_mode(window, cx);
        // TODO: Implement actual replace surround logic
        return true;
    }
    
    false
}

impl Vim {
    pub fn is_match_mode_active(&self) -> bool {
        self.match_mode_active
    }
    
    pub fn match_mode_status(&self) -> Option<String> {
        if !self.match_mode_active {
            return None;
        }
        
        if self.match_mode_awaiting_surround_char {
            Some("match(surround): ".to_string())
        } else if self.match_mode_awaiting_delete_char {
            Some("match(delete): ".to_string())
        } else if self.match_mode_awaiting_replace_from {
            Some("match(replace from): ".to_string())
        } else if self.match_mode_awaiting_replace_to {
            Some("match(replace to): ".to_string())
        } else if self.match_mode_awaiting_text_object.is_some() {
            let around_or_inside = if self.match_mode_awaiting_text_object == Some(true) {
                "around"
            } else {
                "inside"
            };
            Some(format!("match({} object): ", around_or_inside))
        } else {
            Some("match: ".to_string())
        }
    }
}

#[cfg(test)]
mod test {
    use indoc::indoc;
    use crate::{state::Mode, test::VimTestContext};

    #[gpui::test]
    async fn test_match_mode_bracket_matching(cx: &mut gpui::TestAppContext) {
        let mut cx = VimTestContext::new(cx, true).await;

        // Test bracket matching with parentheses
        cx.set_state(
            indoc! {"
            function(ˇarg1, arg2) {
                return arg1 + arg2;
            }"},
            Mode::HelixNormal,
        );

        // Enter match mode and press 'm' for bracket matching
        cx.simulate_keystrokes("m m");

        // Should move to the matching closing parenthesis
        cx.assert_state(
            indoc! {"
            function(arg1, arg2ˇ) {
                return arg1 + arg2;
            }"},
            Mode::HelixNormal,
        );
    }

    #[gpui::test]
    async fn test_match_mode_text_object_around_word(cx: &mut gpui::TestAppContext) {
        let mut cx = VimTestContext::new(cx, true).await;

        cx.set_state("hello woˇrld test", Mode::HelixNormal);

        // Enter match mode, press 'a' for around, then 'w' for word
        cx.simulate_keystrokes("m a w");

        // Should select around the word "world"
        cx.assert_state("hello «worldˇ» test", Mode::HelixNormal);
    }

    #[gpui::test]
    async fn test_match_mode_text_object_inside_parentheses(cx: &mut gpui::TestAppContext) {
        let mut cx = VimTestContext::new(cx, true).await;

        cx.set_state("function(ˇarg1, arg2)", Mode::HelixNormal);

        // Enter match mode, press 'i' for inside, then 'b' for parentheses
        cx.simulate_keystrokes("m i b");

        // Should select inside the parentheses
        cx.assert_state("function(«arg1, arg2ˇ»)", Mode::HelixNormal);
    }

    #[gpui::test]
    async fn test_match_mode_surround_with_quotes(cx: &mut gpui::TestAppContext) {
        let mut cx = VimTestContext::new(cx, true).await;

        cx.set_state("hello «worldˇ»", Mode::HelixNormal);

        // Enter match mode, press 's' for surround, then '"' for quotes
        cx.simulate_keystrokes("m s \"");

        // Should surround the selection with quotes
        cx.assert_state("hello \"worldˇ\"", Mode::HelixNormal);
    }

    #[gpui::test]
    async fn test_match_mode_enters_and_exits_correctly(cx: &mut gpui::TestAppContext) {
        let mut cx = VimTestContext::new(cx, true).await;

        cx.set_state("test ˇtext", Mode::HelixNormal);

        // Should start in normal helix mode
        assert_eq!(cx.mode(), Mode::HelixNormal);
        
        // Press 'm' to enter match mode (this should not change the mode but activate match mode)
        cx.simulate_keystrokes("m");

        // Mode should still be HelixNormal
        assert_eq!(cx.mode(), Mode::HelixNormal);

        // Press escape to exit match mode
        cx.simulate_keystrokes("escape");

        // Should still be in HelixNormal mode
        assert_eq!(cx.mode(), Mode::HelixNormal);
    }
}