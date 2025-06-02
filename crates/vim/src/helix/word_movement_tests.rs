#[cfg(test)]
mod test {
    use crate::{
        test::VimTestContext,
        Mode,
    };
    use indoc::indoc;

    // Tests adapted from helix/helix-core/src/movement.rs
    // test_behaviour_when_moving_to_start_of_next_words() and test_behaviour_when_moving_to_start_of_next_long_words()

    #[gpui::test]
    async fn test_helix_word_vs_word_punctuation(cx: &mut gpui::TestAppContext) {
        // From Helix: helix/helix-core/src/movement.rs:1005-1010
        // "alphanumeric.!,and.?=punctuation are considered 'words' for the purposes of word motion"
        let mut cx = VimTestContext::new(cx, true).await;
        
        cx.set_state("ˇalphanumeric.!,and.?=punctuation", Mode::HelixNormal);
        
        // w should select "alphanumeric" only
        cx.simulate_keystrokes("w");
        cx.assert_state("«alphanumericˇ».!,and.?=punctuation", Mode::HelixNormal);
        
        // Another w should select ".!,"
        cx.simulate_keystrokes("w");
        cx.assert_state("alphanumeric«.!,ˇ»and.?=punctuation", Mode::HelixNormal);
        
        // Another w should select "and"
        cx.simulate_keystrokes("w");
        cx.assert_state("alphanumeric.!,«andˇ».?=punctuation", Mode::HelixNormal);
    }

    #[gpui::test]
    async fn test_helix_word_vs_long_word_punctuation(cx: &mut gpui::TestAppContext) {
        // From Helix: helix/helix-core/src/movement.rs:1265-1267
        // "alphanumeric.!,and.?=punctuation are not treated any differently than alphanumerics"
        let mut cx = VimTestContext::new(cx, true).await;
        
        cx.set_state("ˇalphanumeric.!,and.?=punctuation", Mode::HelixNormal);
        
        // W should select the entire thing as one WORD
        cx.simulate_keystrokes("shift-w");
        cx.assert_state("«alphanumeric.!,and.?=punctuationˇ»", Mode::HelixNormal);
    }

    #[gpui::test]
    async fn test_helix_word_with_underscores(cx: &mut gpui::TestAppContext) {
        // From Helix: helix/helix-core/src/movement.rs:1007
        // "Identifiers_with_underscores are considered a single word"
        let mut cx = VimTestContext::new(cx, true).await;
        
        cx.set_state("ˇIdentifiers_with_underscores", Mode::HelixNormal);
        
        // w should select entire identifier as one word
        cx.simulate_keystrokes("w");
        cx.assert_state("«Identifiers_with_underscoresˇ»", Mode::HelixNormal);
    }

    #[gpui::test]
    async fn test_helix_word_whitespace_behavior(cx: &mut gpui::TestAppContext) {
        // From Helix: helix/helix-core/src/movement.rs:995-996
        // "Basic forward motion stops at the first space"
        // Range::new(0, 0) to Range::new(0, 6) - selects "Basic "
        let mut cx = VimTestContext::new(cx, true).await;
        
        cx.set_state("ˇBasic forward", Mode::HelixNormal);
        
        // w should select "Basic " (including trailing space)
        cx.simulate_keystrokes("w");
        cx.assert_state("«Basic ˇ»forward", Mode::HelixNormal);
    }

    #[gpui::test]
    async fn test_helix_word_from_whitespace(cx: &mut gpui::TestAppContext) {
        // From Helix: helix/helix-core/src/movement.rs:1000
        // "Starting from whitespace moves to last space in sequence"
        // Range::new(0, 0) to Range::new(0, 4) - selects "    "
        let mut cx = VimTestContext::new(cx, true).await;
        
        cx.set_state("ˇ    Starting from", Mode::HelixNormal);
        
        // w should select the whitespace sequence "    "
        cx.simulate_keystrokes("w");
        cx.assert_state("«    ˇ»Starting from", Mode::HelixNormal);
    }

    #[gpui::test]
    async fn test_helix_word_long_whitespace(cx: &mut gpui::TestAppContext) {
        // From Helix: helix/helix-core/src/movement.rs:997
        // "Long       whitespace gap is bridged by the head"
        // Range::new(0, 0) to Range::new(0, 11) - selects "Long       "
        let mut cx = VimTestContext::new(cx, true).await;
        
        cx.set_state("ˇLong       whitespace", Mode::HelixNormal);
        
        // w should select "Long       " (including trailing whitespace up to next word)
        cx.simulate_keystrokes("w");
        cx.assert_state("«Long       ˇ»whitespace", Mode::HelixNormal);
    }

    #[gpui::test]
    async fn test_helix_word_from_mid_word(cx: &mut gpui::TestAppContext) {
        // From Helix: helix/helix-core/src/movement.rs:1001
        // "Starting from mid-word leaves anchor at start position and moves head"
        // Range::new(3, 3) to Range::new(3, 9) - selects "rting "
        let mut cx = VimTestContext::new(cx, true).await;
        
        cx.set_state("Staˇrting from", Mode::HelixNormal);
        
        // w should select from current position to next word start
        cx.simulate_keystrokes("w");
        cx.assert_state("Sta«rting ˇ»from", Mode::HelixNormal);
    }

    #[gpui::test]
    async fn test_helix_word_end_basic(cx: &mut gpui::TestAppContext) {
        // From Helix: helix/helix-core/src/movement.rs:1603-1604
        // "Basic forward motion from the start of a word to the end of it"
        let mut cx = VimTestContext::new(cx, true).await;
        
        cx.set_state("ˇBasic forward", Mode::HelixNormal);
        
        // e should select to end of current word
        cx.simulate_keystrokes("e");
        cx.assert_state("«Basicˇ» forward", Mode::HelixNormal);
    }

    #[gpui::test]
    async fn test_helix_word_end_punctuation(cx: &mut gpui::TestAppContext) {
        // From Helix: helix/helix-core/src/movement.rs:1619-1626
        // Word end behavior with punctuation
        let mut cx = VimTestContext::new(cx, true).await;
        
        cx.set_state("ˇalphanumeric.!,and.?=punctuation", Mode::HelixNormal);
        
        // e should select to end of "alphanumeric"
        cx.simulate_keystrokes("e");
        cx.assert_state("«alphanumericˇ».!,and.?=punctuation", Mode::HelixNormal);
        
        // Another e should select ".!,"
        cx.simulate_keystrokes("e");
        cx.assert_state("alphanumeric«.!,ˇ»and.?=punctuation", Mode::HelixNormal);
    }

    #[gpui::test]
    async fn test_helix_word_back_basic(cx: &mut gpui::TestAppContext) {
        // From Helix: helix/helix-core/src/movement.rs:1335-1336
        // "Basic backward motion from the middle of a word"
        // Range::new(3, 3) to Range::new(4, 0) - selects "Basi" (backward)
        let mut cx = VimTestContext::new(cx, true).await;
        
        cx.set_state("Basˇic backward motion from the middle of a word", Mode::HelixNormal);
        
        // b should select from current position to start of word (backward selection)
        cx.simulate_keystrokes("b");
        cx.assert_state("«ˇBasi»c backward motion from the middle of a word", Mode::HelixNormal);
    }

    #[gpui::test]
    async fn test_helix_word_back_whitespace(cx: &mut gpui::TestAppContext) {
        // From Helix: helix/helix-core/src/movement.rs:1349
        // "    Starting from whitespace moves to first space in sequence"
        // When cursor is at position 4 ('S'), backward movement should select "    S"
        let mut cx = VimTestContext::new(cx, true).await;
        
        cx.set_state("    ˇStarting from whitespace moves to first space in sequence", Mode::HelixNormal);
        
        // b should select back to start of whitespace (backward selection including current char)
        cx.simulate_keystrokes("b");
        cx.assert_state("«ˇ    S»tarting from whitespace moves to first space in sequence", Mode::HelixNormal);
    }

    #[gpui::test]
    async fn test_helix_word_newlines(cx: &mut gpui::TestAppContext) {
        // From Helix: helix/helix-core/src/movement.rs:1008-1009
        // "Jumping\n    into starting whitespace selects the spaces before 'into'"
        // Range::new(0, 7) to Range::new(8, 12) - selects "    " (spaces only, not newline)
        let mut cx = VimTestContext::new(cx, true).await;
        
        cx.set_state(indoc! {"
            Jumpingˇ
                into starting
        "}, Mode::HelixNormal);
        
        // w should select only the whitespace before 'into', not the newline
        cx.simulate_keystrokes("w");
        cx.assert_state(indoc! {"
            Jumping
            «    ˇ»into starting
        "}, Mode::HelixNormal);
    }

    #[gpui::test]
    async fn test_helix_word_punctuation_joins(cx: &mut gpui::TestAppContext) {
        // From Helix: helix/helix-core/src/movement.rs:1011-1012
        // ".._.._ punctuation is not joined by underscores into a single block"
        // Range::new(0, 0) to Range::new(0, 2) - selects ".."
        let mut cx = VimTestContext::new(cx, true).await;
        
        cx.set_state("ˇ.._.._ punctuation", Mode::HelixNormal);
        
        // w should select ".." as first punctuation word
        cx.simulate_keystrokes("w");
        cx.assert_state("«..ˇ»_.._ punctuation", Mode::HelixNormal);
    }

    #[gpui::test]
    async fn test_helix_tutor_example(cx: &mut gpui::TestAppContext) {
        // From Helix tutor 3.3: "one-of-a-kind" and "modal" word vs WORD test
        let mut cx = VimTestContext::new(cx, true).await;
        
        cx.set_state("ˇHelix is a one-of-a-kind \"modal\" text editor", Mode::HelixNormal);
        
        // Test w movements - Helix behavior based on punctuation/word boundaries
        cx.simulate_keystrokes("w");  // Select "Helix "
        cx.assert_state("«Helix ˇ»is a one-of-a-kind \"modal\" text editor", Mode::HelixNormal);
        
        cx.simulate_keystrokes("w");  // Select "is "
        cx.assert_state("Helix «is ˇ»a one-of-a-kind \"modal\" text editor", Mode::HelixNormal);
        
        cx.simulate_keystrokes("w");  // Select "a "
        cx.assert_state("Helix is «a ˇ»one-of-a-kind \"modal\" text editor", Mode::HelixNormal);
        
        cx.simulate_keystrokes("w");  // Select "one"
        cx.assert_state("Helix is a «oneˇ»-of-a-kind \"modal\" text editor", Mode::HelixNormal);
    }

    #[gpui::test]
    async fn test_helix_tutor_word_example(cx: &mut gpui::TestAppContext) {
        // From Helix tutor 3.3: WORD movements should select "one-of-a-kind" in one keystroke
        let mut cx = VimTestContext::new(cx, true).await;
        
        cx.set_state("ˇHelix is a one-of-a-kind \"modal\" text editor", Mode::HelixNormal);
        
        // Test W movements - should select entire WORDS (ignore punctuation)
        cx.simulate_keystrokes("shift-w");  // Select "Helix "
        cx.assert_state("«Helix ˇ»is a one-of-a-kind \"modal\" text editor", Mode::HelixNormal);
        
        cx.simulate_keystrokes("shift-w");  // Select "is "
        cx.assert_state("Helix «is ˇ»a one-of-a-kind \"modal\" text editor", Mode::HelixNormal);
        
        cx.simulate_keystrokes("shift-w");  // Select "a "
        cx.assert_state("Helix is «a ˇ»one-of-a-kind \"modal\" text editor", Mode::HelixNormal);
        
        cx.simulate_keystrokes("shift-w");  // Select "one-of-a-kind "
        cx.assert_state("Helix is a «one-of-a-kind ˇ»\"modal\" text editor", Mode::HelixNormal);
    }

    #[gpui::test]
    async fn test_helix_word_boundary_behavior(cx: &mut gpui::TestAppContext) {
        // From Helix: helix/helix-core/src/movement.rs:996
        // "Starting from a boundary advances the anchor"
        // Range::new(0, 0) to Range::new(1, 10) - selects "Starting "
        let mut cx = VimTestContext::new(cx, true).await;
        
        cx.set_state("ˇ Starting from", Mode::HelixNormal);
        
        // w from space boundary should select "Starting " (word + trailing space)
        cx.simulate_keystrokes("w");
        cx.assert_state(" «Starting ˇ»from", Mode::HelixNormal);
    }

    #[gpui::test]
    async fn test_helix_word_select_mode_extends(cx: &mut gpui::TestAppContext) {
        // Test that word movements extend selections in select mode
        let mut cx = VimTestContext::new(cx, true).await;
        
        cx.set_state("hello ˇworld testing", Mode::HelixNormal);
        
        // Create initial selection
        cx.simulate_keystrokes("w");
        cx.assert_state("hello «world ˇ»testing", Mode::HelixNormal);
        
        // Enter select mode
        cx.simulate_keystrokes("v");
        assert_eq!(cx.mode(), Mode::HelixSelect);
        
        // Word movement should extend selection
        cx.simulate_keystrokes("w");
        cx.assert_state("hello «world testingˇ»", Mode::HelixSelect);
    }
}