use crate::{
    helix::core::{self},
    Vim,
};
use editor::{scroll::Autoscroll, Editor};
use gpui::{actions, Window, Context};

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
        let buffer = editor.buffer().read(cx);
        let snapshot = buffer.snapshot(cx);
        let rope_text = rope::Rope::from(snapshot.text());
        
        editor.change_selections(Some(Autoscroll::fit()), window, cx, |s| {
            s.move_with(|map, selection| {
                // Get current cursor position
                let cursor_pos = selection.head();
                let cursor_byte_offset = cursor_pos.to_offset(map, editor::Bias::Left);
                let cursor_char_offset = core::byte_offset_to_char_index(&rope_text, cursor_byte_offset);
                
                // Find matching bracket
                if let Some(match_char_offset) = core::find_matching_bracket(&rope_text, cursor_char_offset) {
                    // Convert back to byte offset and then to Point
                    let match_byte_offset = core::char_index_to_byte_offset(&rope_text, match_char_offset);
                    let match_point = snapshot.offset_to_point(match_byte_offset);
                    let match_display_point = map.point_to_display_point(match_point, editor::Bias::Left);
                    
                    // Move cursor to matching bracket
                    selection.collapse_to(match_display_point, selection.goal);
                }
                // If no matching bracket found, selection remains unchanged
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{test::VimTestContext, state::Mode};
    use indoc::indoc;

    #[gpui::test]
    async fn test_match_brackets_parentheses(cx: &mut gpui::TestAppContext) {
        let mut cx = VimTestContext::new(cx, true).await;

        // Test bracket matching with parentheses - cursor on opening bracket
        cx.set_state(
            indoc! {"
            functionˇ(arg1, arg2) {
                return arg1 + arg2;
            }"},
            Mode::HelixNormal,
        );

        // Dispatch the match brackets action directly
        cx.dispatch_action(MatchBrackets);

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
    async fn test_match_brackets_parentheses_reverse(cx: &mut gpui::TestAppContext) {
        let mut cx = VimTestContext::new(cx, true).await;

        // Test bracket matching with parentheses - cursor on closing bracket
        cx.set_state(
            indoc! {"
            function(arg1, arg2ˇ) {
                return arg1 + arg2;
            }"},
            Mode::HelixNormal,
        );

        // Dispatch the match brackets action directly
        cx.dispatch_action(MatchBrackets);

        // Should move to the matching opening parenthesis
        cx.assert_state(
            indoc! {"
            functionˇ(arg1, arg2) {
                return arg1 + arg2;
            }"},
            Mode::HelixNormal,
        );
    }

    #[gpui::test]
    async fn test_match_brackets_square_brackets(cx: &mut gpui::TestAppContext) {
        let mut cx = VimTestContext::new(cx, true).await;

        // Test bracket matching with square brackets
        cx.set_state("arrayˇ[index] = value;", Mode::HelixNormal);

        cx.dispatch_action(MatchBrackets);

        // Should move to the matching closing bracket
        cx.assert_state("array[indexˇ] = value;", Mode::HelixNormal);
    }

    #[gpui::test]
    async fn test_match_brackets_curly_braces(cx: &mut gpui::TestAppContext) {
        let mut cx = VimTestContext::new(cx, true).await;

        // Test bracket matching with curly braces
        cx.set_state(
            indoc! {"
            if (condition) ˇ{
                doSomething();
            }"},
            Mode::HelixNormal,
        );

        cx.dispatch_action(MatchBrackets);

        // Should move to the matching closing brace
        cx.assert_state(
            indoc! {"
            if (condition) {
                doSomething();
            ˇ}"},
            Mode::HelixNormal,
        );
    }

    #[gpui::test]
    async fn test_match_brackets_nested(cx: &mut gpui::TestAppContext) {
        let mut cx = VimTestContext::new(cx, true).await;

        // Test bracket matching with nested brackets
        cx.set_state("outer(innerˇ(deep))", Mode::HelixNormal);

        cx.dispatch_action(MatchBrackets);

        // Should move to the matching closing parenthesis of the inner pair
        cx.assert_state("outer(inner(deepˇ))", Mode::HelixNormal);
    }

    #[gpui::test]
    async fn test_match_brackets_no_match(cx: &mut gpui::TestAppContext) {
        let mut cx = VimTestContext::new(cx, true).await;

        // Test with cursor not on a bracket
        cx.set_state("hello ˇworld", Mode::HelixNormal);

        cx.dispatch_action(MatchBrackets);

        // Should not move cursor
        cx.assert_state("hello ˇworld", Mode::HelixNormal);
    }

    #[gpui::test]
    async fn test_match_brackets_tutor_example_1(cx: &mut gpui::TestAppContext) {
        let mut cx = VimTestContext::new(cx, true).await;

        // From Helix tutor: "you can (jump between matching parentheses)"
        cx.set_state("you can ˇ(jump between matching parentheses)", Mode::HelixNormal);

        cx.dispatch_action(MatchBrackets);

        cx.assert_state("you can (jump between matching parenthesesˇ)", Mode::HelixNormal);
    }

    #[gpui::test]
    async fn test_match_brackets_tutor_example_2(cx: &mut gpui::TestAppContext) {
        let mut cx = VimTestContext::new(cx, true).await;

        // From Helix tutor: "or between matching [ square brackets ]"
        cx.set_state("or between matching ˇ[ square brackets ]", Mode::HelixNormal);

        cx.dispatch_action(MatchBrackets);

        cx.assert_state("or between matching [ square brackets ˇ]", Mode::HelixNormal);
    }

    #[gpui::test]
    async fn test_match_brackets_tutor_example_3(cx: &mut gpui::TestAppContext) {
        let mut cx = VimTestContext::new(cx, true).await;

        // From Helix tutor: "now { you know the drill: this works with brackets too }"
        cx.set_state("now ˇ{ you know the drill: this works with brackets too }", Mode::HelixNormal);

        cx.dispatch_action(MatchBrackets);

        cx.assert_state("now { you know the drill: this works with brackets too ˇ}", Mode::HelixNormal);
    }

    #[gpui::test]
    async fn test_match_brackets_mode_preservation(cx: &mut gpui::TestAppContext) {
        let mut cx = VimTestContext::new(cx, true).await;

        // Test that mode is preserved after bracket matching
        cx.set_state("functionˇ(arg)", Mode::HelixNormal);

        cx.dispatch_action(MatchBrackets);

        // Should move to matching bracket and stay in HelixNormal mode
        cx.assert_state("function(argˇ)", Mode::HelixNormal);
    }
} 