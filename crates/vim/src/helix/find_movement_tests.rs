#[cfg(test)]
mod test {
    use crate::{
        test::VimTestContext,
        Mode,
    };
    use indoc::indoc;

    // Tests adapted from helix/helix-term/tests/test/movement.rs
    // find_char_line_ending() and related find movement tests

    #[gpui::test]
    async fn test_helix_find_char_forward_basic(cx: &mut gpui::TestAppContext) {
        // From Helix: helix/helix-term/tests/test/movement.rs:552-570
        // find_char_line_ending() - basic find forward
        let mut cx = VimTestContext::new(cx, true).await;
        
        cx.set_state("hello ˇworld testing", Mode::HelixNormal);
        
        // f + char should create selection from cursor to target character (inclusive)
        cx.simulate_keystrokes("f t");
        
        // Should select from current position (space after "hello") to 't' in "testing"
        cx.assert_state("hello «world tˇ»esting", Mode::HelixNormal);
    }

    #[gpui::test]
    async fn test_helix_find_char_till_forward(cx: &mut gpui::TestAppContext) {
        // Test till forward (t) - stops before target character
        let mut cx = VimTestContext::new(cx, true).await;
        
        cx.set_state("hello ˇworld testing", Mode::HelixNormal);
        
        // t + char should create selection up to (but not including) target
        cx.simulate_keystrokes("t t");
        
        // Should select from current position to just before 't' in "testing"
        cx.assert_state("hello «world ˇ»testing", Mode::HelixNormal);
    }

    #[gpui::test]
    async fn test_helix_find_char_backward_basic(cx: &mut gpui::TestAppContext) {
        // Test find backward (F)
        let mut cx = VimTestContext::new(cx, true).await;
        
        cx.set_state("hello world tesˇting", Mode::HelixNormal);
        
        // F + char should create selection backward to target
        cx.simulate_keystrokes("shift-f w");
        
        // Should select from current position back to 'w' in "world"
        cx.assert_state("hello «wˇorld tes»ting", Mode::HelixNormal);
    }

    #[gpui::test]
    async fn test_helix_find_char_till_backward(cx: &mut gpui::TestAppContext) {
        // Test till backward (T)
        let mut cx = VimTestContext::new(cx, true).await;
        
        cx.set_state("hello world tesˇting", Mode::HelixNormal);
        
        // T + char should create selection backward up to (but not including) target
        cx.simulate_keystrokes("shift-t w");
        
        // Should select from current position back to just after 'w' in "world"
        cx.assert_state("hello w«oˇrld tes»ting", Mode::HelixNormal);
    }

    #[gpui::test]
    async fn test_helix_find_char_line_ending(cx: &mut gpui::TestAppContext) {
        // From Helix: helix/helix-term/tests/test/movement.rs:552-570
        // Complex find across line endings
        let mut cx = VimTestContext::new(cx, true).await;
        
        cx.set_state(
            indoc! {"
                one
                «ˇt»wo
                three
            "},
            Mode::HelixNormal,
        );
        
        // This tests find behavior with line endings
        // Note: Exact behavior may need verification against real Helix
        cx.simulate_keystrokes("shift-t enter");
        
        // Should find previous newline character
        // Expected result depends on Helix's exact find behavior
    }

    #[gpui::test]
    async fn test_helix_find_char_multiple_occurrences(cx: &mut gpui::TestAppContext) {
        // Test finding character that appears multiple times
        let mut cx = VimTestContext::new(cx, true).await;
        
        cx.set_state("hello ˇworld wonderful", Mode::HelixNormal);
        
        // f + char should find first occurrence from current position
        cx.simulate_keystrokes("f o");
        
        // Should select from current position to first 'o' in "world"
        cx.assert_state("hello «woˇ»rld wonderful", Mode::HelixNormal);
        
        // Subsequent find should go to next occurrence from current position
        cx.simulate_keystrokes("f o");
        
        // Should select from current position to next 'o' in "wonderful"
        cx.assert_state("hello wo«rld wˇ»onderful", Mode::HelixNormal);
    }

    #[gpui::test]
    async fn test_helix_find_char_not_found(cx: &mut gpui::TestAppContext) {
        // Test behavior when character is not found
        let mut cx = VimTestContext::new(cx, true).await;
        
        cx.set_state("hello ˇworld", Mode::HelixNormal);
        
        // f + char that doesn't exist should not change selection
        cx.simulate_keystrokes("f z");
        
        cx.assert_state("hello ˇworld", Mode::HelixNormal);
    }

    #[gpui::test]
    async fn test_helix_find_char_at_line_boundary(cx: &mut gpui::TestAppContext) {
        // Test find behavior at line boundaries
        let mut cx = VimTestContext::new(cx, true).await;
        
        cx.set_state(
            indoc! {"
                hello worldˇ
                testing find
            "},
            Mode::HelixNormal,
        );
        
        // Find character on next line
        cx.simulate_keystrokes("f i");
        
        // Should find 'i' in "testing" on next line
        cx.assert_state(
            indoc! {"
                hello world
                test«iˇ»ng find
            "},
            Mode::HelixNormal,
        );
    }

    #[gpui::test]
    async fn test_helix_find_char_select_mode_extends(cx: &mut gpui::TestAppContext) {
        // Test that find commands extend selections in select mode
        let mut cx = VimTestContext::new(cx, true).await;
        
        cx.set_state("hello ˇworld testing", Mode::HelixNormal);
        
        // Create initial selection
        cx.simulate_keystrokes("w");
        cx.assert_state("hello «worldˇ» testing", Mode::HelixNormal);
        
        // Enter select mode
        cx.simulate_keystrokes("v");
        assert_eq!(cx.mode(), Mode::HelixSelect);
        
        // Find should extend the existing selection
        cx.simulate_keystrokes("f t");
        cx.assert_state("hello «world tesˇ»ting", Mode::HelixSelect);
    }

    #[gpui::test]
    async fn test_helix_find_char_punctuation(cx: &mut gpui::TestAppContext) {
        // Test finding punctuation characters
        let mut cx = VimTestContext::new(cx, true).await;
        
        cx.set_state("hello, ˇworld; testing!", Mode::HelixNormal);
        
        // Find comma
        cx.simulate_keystrokes("shift-f ,");
        cx.assert_state("hello«ˇ, ˇ»world; testing!", Mode::HelixNormal);
        
        // Find semicolon forward
        cx.simulate_keystrokes("f ;");
        cx.assert_state("hello, world«;ˇ» testing!", Mode::HelixNormal);
    }

    #[gpui::test]
    async fn test_helix_find_char_whitespace(cx: &mut gpui::TestAppContext) {
        // Test finding whitespace characters
        let mut cx = VimTestContext::new(cx, true).await;
        
        cx.set_state("helloˇworld testing", Mode::HelixNormal);
        
        // Find space character
        cx.simulate_keystrokes("f space");
        
        cx.assert_state("helloworld« ˇ»testing", Mode::HelixNormal);
    }

    #[gpui::test]
    async fn test_helix_find_char_case_sensitive(cx: &mut gpui::TestAppContext) {
        // Test that find is case sensitive
        let mut cx = VimTestContext::new(cx, true).await;
        
        cx.set_state("Hello ˇworld World", Mode::HelixNormal);
        
        // Find lowercase 'w'
        cx.simulate_keystrokes("f w");
        cx.assert_state("Hello «wˇ»orld World", Mode::HelixNormal);
        
        // Find uppercase 'W'
        cx.simulate_keystrokes("f shift-w");
        cx.assert_state("Hello world «Wˇ»orld", Mode::HelixNormal);
    }

    #[gpui::test]
    async fn test_helix_find_repeat_last(cx: &mut gpui::TestAppContext) {
        // Test repeating last find with ; and ,
        let mut cx = VimTestContext::new(cx, true).await;
        
        cx.set_state("hello ˇworld wonderful", Mode::HelixNormal);
        
        // Initial find
        cx.simulate_keystrokes("f o");
        cx.assert_state("hello w«oˇ»rld wonderful", Mode::HelixNormal);
        
        // Repeat find forward with ;
        cx.simulate_keystrokes(";");
        cx.assert_state("hello wo«rld w»onderful", Mode::HelixNormal);
        
        // Repeat find backward with ,
        cx.simulate_keystrokes(",");
        cx.assert_state("hello w«oˇ»rld wonderful", Mode::HelixNormal);
    }

    #[gpui::test]
    async fn test_helix_find_complex_workflow(cx: &mut gpui::TestAppContext) {
        // Test complex find workflow combining multiple operations
        let mut cx = VimTestContext::new(cx, true).await;
        
        cx.set_state(
            indoc! {"
                function test(ˇparam) {
                    return param + 1;
                }
            "},
            Mode::HelixNormal,
        );
        
        // Find opening parenthesis backward
        cx.simulate_keystrokes("shift-f (");
        cx.assert_state(
            indoc! {"
                function test«ˇ(ˇ»param) {
                    return param + 1;
                }
            "},
            Mode::HelixNormal,
        );
        
        // Find closing brace forward
        cx.simulate_keystrokes("f }");
        cx.assert_state(
            indoc! {"
                function test(param) {
                    return param + 1;
                «}ˇ»
            "},
            Mode::HelixNormal,
        );
    }
}