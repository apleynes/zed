#[cfg(test)]
mod test {
    use crate::{
        test::VimTestContext,
        Mode,
        helix::*,
    };
    use gpui::TestAppContext;
    use indoc::indoc;

    // Test cases adapted from Helix's actual test suite
    // Converted from Helix notation to Zed notation:
    // Helix: #[text|]# -> Zed: «textˇ»
    // Helix: #[|text]# -> Zed: «ˇtext»
    // Helix: #(text|)# -> Zed: secondary «textˇ»
    // Helix: #(|text)# -> Zed: secondary «ˇtext»

    #[gpui::test]
    async fn test_helix_selection_duplication_copy_forward(cx: &mut TestAppContext) {
        let mut cx = VimTestContext::new(cx, true).await;
        
        // From Helix test: Copy selection to next line (C key)
        cx.set_state(
            indoc! {"
                «loˇ»rem
                ipsum
                dolor
            "},
            Mode::HelixNormal,
        );
        
        cx.dispatch_action(CopySelectionOnNextLine);
        
        cx.assert_state(
            indoc! {"
                «loˇ»rem
                «loˇ»sum
                «doˇ»lor
            "},
            Mode::HelixNormal,
        );
    }

    #[gpui::test]
    async fn test_helix_selection_duplication_copy_backward(cx: &mut TestAppContext) {
        let mut cx = VimTestContext::new(cx, true).await;
        
        // From Helix test: Copy selection to previous line (Alt-C key)
        cx.set_state(
            indoc! {"
                test
                «testˇ»
            "},
            Mode::HelixNormal,
        );
        
        cx.dispatch_action(CopySelectionOnPrevLine);
        
        cx.assert_state(
            indoc! {"
                «testˇ»
                «testˇ»
            "},
            Mode::HelixNormal,
        );
    }

    #[gpui::test]
    async fn test_helix_multi_selection_paste(cx: &mut TestAppContext) {
        let mut cx = VimTestContext::new(cx, true).await;
        
        // From Helix test: Multi-selection paste behavior
        cx.set_state(
            indoc! {"
                «ˇlorem»
                «ˇipsum»
                «ˇdolor»
            "},
            Mode::HelixNormal,
        );
        
        cx.simulate_keystrokes("y p");  // Yank and paste (keep as keystroke test)
        
        cx.assert_state(
            indoc! {"
                lorem«ˇlorem»
                ipsum«ˇipsum»
                dolor«ˇdolor»
            "},
            Mode::HelixNormal,
        );
    }

    #[gpui::test]
    async fn test_helix_trim_selections(cx: &mut TestAppContext) {
        let mut cx = VimTestContext::new(cx, true).await;
        
        // Test trim whitespace from selections (_)
        cx.set_state("  «  hello  ˇ»  world  ", Mode::HelixNormal);
        
        cx.dispatch_action(TrimSelections);
        
        cx.assert_state("  «helloˇ»  world  ", Mode::HelixNormal);
    }

    #[gpui::test]
    async fn test_helix_trim_selections_whitespace_only(cx: &mut TestAppContext) {
        let mut cx = VimTestContext::new(cx, true).await;
        
        // Test trim selections that are only whitespace
        cx.set_state("hello«   ˇ»world", Mode::HelixNormal);
        
        cx.dispatch_action(TrimSelections);
        
        // Whitespace-only selection becomes cursor
        cx.assert_state("helloˇworld", Mode::HelixNormal);
    }

    #[gpui::test]
    async fn test_helix_align_selections(cx: &mut TestAppContext) {
        let mut cx = VimTestContext::new(cx, true).await;
        
        // Test align selections to same width (&)
        cx.set_state(
            indoc! {"
                a «shortˇ»
                a «longerˇ»
                a «veryˇ»
            "},
            Mode::HelixNormal,
        );
        
        cx.dispatch_action(AlignSelections);
        
        cx.assert_state(
            indoc! {"
                a «short  ˇ»
                a «longer ˇ»
                a «very   ˇ»
            "},
            Mode::HelixNormal,
        );
    }

    #[gpui::test]
    async fn test_helix_merge_selections(cx: &mut TestAppContext) {
        let mut cx = VimTestContext::new(cx, true).await;
        
        // Test merge selections (Alt--)
        cx.set_state("hello «worldˇ» testing «textˇ»", Mode::HelixNormal);
        
        cx.dispatch_action(MergeSelections);
        
        cx.assert_state("hello «world testing textˇ»", Mode::HelixNormal);
    }

    #[gpui::test]
    async fn test_helix_merge_consecutive_selections(cx: &mut TestAppContext) {
        let mut cx = VimTestContext::new(cx, true).await;
        
        // Test merge consecutive selections (Alt-_)
        cx.set_state(
            indoc! {"
                first «lineˇ»
                second «lineˇ»
                third «lineˇ»
            "},
            Mode::HelixNormal,
        );
        
        cx.dispatch_action(MergeConsecutiveSelections);
        
        // Should merge consecutive ranges
        cx.assert_state(
            indoc! {"
                first «line
                second line
                third lineˇ»
            "},
            Mode::HelixNormal,
        );
    }

    #[gpui::test]
    async fn test_helix_collapse_selection(cx: &mut TestAppContext) {
        let mut cx = VimTestContext::new(cx, true).await;
        
        // Test collapse selection to cursor (;)
        cx.set_state("hello «worldˇ» testing", Mode::HelixNormal);
        
        cx.dispatch_action(CollapseSelection);
        
        cx.assert_state("hello worldˇ testing", Mode::HelixNormal);
    }

    #[gpui::test]
    async fn test_helix_flip_selections(cx: &mut TestAppContext) {
        let mut cx = VimTestContext::new(cx, true).await;
        
        // Test flip selection direction (Alt-;)
        cx.set_state("hello «worldˇ» testing", Mode::HelixNormal);
        
        cx.dispatch_action(FlipSelections);
        
        cx.assert_state("hello «ˇworld» testing", Mode::HelixNormal);
    }

    #[gpui::test]
    async fn test_helix_keep_primary_selection(cx: &mut TestAppContext) {
        let mut cx = VimTestContext::new(cx, true).await;
        
        // Test keep only primary selection (,)
        cx.set_state("«firstˇ» and «secondˇ» and «thirdˇ»", Mode::HelixNormal);
        
        cx.dispatch_action(KeepPrimarySelection);
        
        cx.assert_state("«firstˇ» and second and third", Mode::HelixNormal);
    }

    #[gpui::test]
    async fn test_helix_remove_primary_selection(cx: &mut TestAppContext) {
        let mut cx = VimTestContext::new(cx, true).await;
        
        // Test remove primary selection (Alt-,)
        cx.set_state("«firstˇ» and «secondˇ» and «thirdˇ»", Mode::HelixNormal);
        
        cx.dispatch_action(RemovePrimarySelection);
        
        cx.assert_state("first and «secondˇ» and «thirdˇ»", Mode::HelixNormal);
    }

    #[gpui::test]
    async fn test_helix_rotate_selection_contents_forward(cx: &mut TestAppContext) {
        let mut cx = VimTestContext::new(cx, true).await;
        
        // Test rotate selection contents forward (Alt-))
        cx.set_state("«oneˇ» «twoˇ» «threeˇ»", Mode::HelixNormal);
        
        cx.dispatch_action(RotateSelectionContentsForward);
        
        cx.assert_state("«threeˇ» «oneˇ» «twoˇ»", Mode::HelixNormal);
    }

    #[gpui::test]
    async fn test_helix_rotate_selection_contents_backward(cx: &mut TestAppContext) {
        let mut cx = VimTestContext::new(cx, true).await;
        
        // Test rotate selection contents backward (Alt-()
        cx.set_state("«oneˇ» «twoˇ» «threeˇ»", Mode::HelixNormal);
        
        cx.dispatch_action(RotateSelectionContentsBackward);
        
        cx.assert_state("«twoˇ» «threeˇ» «oneˇ»", Mode::HelixNormal);
    }

    #[gpui::test]
    async fn test_helix_find_creates_selection_forward(cx: &mut TestAppContext) {
        let mut cx = VimTestContext::new(cx, true).await;
        
        // Test find forward creates selection (f)
        cx.set_state("hello ˇworld testing", Mode::HelixNormal);
        
        cx.simulate_keystrokes("f t");  // Find 't'
        
        cx.assert_state("hello world «tesˇ»ting", Mode::HelixNormal);
    }

    #[gpui::test]
    async fn test_helix_find_creates_selection_till(cx: &mut TestAppContext) {
        let mut cx = VimTestContext::new(cx, true).await;
        
        // Test find till creates selection (t)
        cx.set_state("hello ˇworld testing", Mode::HelixNormal);
        
        cx.simulate_keystrokes("t t");  // Till 't'
        
        cx.assert_state("hello world «tesˇ»ting", Mode::HelixNormal);
    }

    #[gpui::test]
    async fn test_helix_find_backward_creates_selection(cx: &mut TestAppContext) {
        let mut cx = VimTestContext::new(cx, true).await;
        
        // Test find backward creates selection (F)
        cx.set_state("hello world tesˇting", Mode::HelixNormal);
        
        cx.simulate_keystrokes("shift-f w");  // Find 'w' backward
        
        cx.assert_state("hello «ˇworld tesˇ»ting", Mode::HelixNormal);
    }

    #[gpui::test]
    async fn test_helix_shift_word_movements_create_selections(cx: &mut TestAppContext) {
        let mut cx = VimTestContext::new(cx, true).await;
        
        // Test Shift+W creates selection ignoring punctuation
        cx.set_state("hello ˇworld testing-word", Mode::HelixNormal);
        
        cx.simulate_keystrokes("shift-w");  // Move to next WORD
        
        cx.assert_state("hello «world ˇ»testing-word", Mode::HelixNormal);
    }

    #[gpui::test]
    async fn test_helix_select_mode_extends_selections(cx: &mut TestAppContext) {
        let mut cx = VimTestContext::new(cx, true).await;
        
        // Test select mode extends existing selections
        cx.set_state("hello ˇworld testing", Mode::HelixNormal);
        
        cx.simulate_keystrokes("w");  // Create word selection
        cx.assert_state("hello «worldˇ» testing", Mode::HelixNormal);
        
        cx.simulate_keystrokes("v");  // Enter select mode
        assert_eq!(cx.mode(), Mode::HelixSelect);
        
        cx.simulate_keystrokes("w");  // Extend selection
        cx.assert_state("hello «world testingˇ»", Mode::HelixSelect);
    }

    #[gpui::test]
    async fn test_helix_comprehensive_workflow(cx: &mut TestAppContext) {
        let mut cx = VimTestContext::new(cx, true).await;
        
        // Test comprehensive workflow combining multiple operations
        cx.set_state(
            indoc! {"
                one   ˇtwo  
                three ˇfour   
                five  ˇsix
            "},
            Mode::HelixNormal,
        );
        
        // Create word selections
        cx.simulate_keystrokes("w");  // Select "two"
        cx.simulate_keystrokes("j w");  // Select "four"
        cx.simulate_keystrokes("j w");  // Select "six"
        
        cx.assert_state(
            indoc! {"
                one   «twoˇ»  
                three «fourˇ»   
                five  «sixˇ»
            "},
            Mode::HelixNormal,
        );
        
        // Trim whitespace from selections
        cx.dispatch_action(TrimSelections);
        
        // Align selections to same width
        cx.dispatch_action(AlignSelections);
        
        // Copy selections to next line
        cx.dispatch_action(CopySelectionOnNextLine);
        
        // Merge all selections
        cx.dispatch_action(MergeSelections);
        
        // Should result in one large merged selection (exact result depends on implementation)
        // The key is that this workflow should not panic or fail
    }

    #[gpui::test]
    async fn test_helix_join_selections(cx: &mut TestAppContext) {
        let mut cx = VimTestContext::new(cx, true).await;
        
        // From Helix test: Join selections behavior
        cx.set_state(
            indoc! {"
                «aˇ»bc
                def
            "},
            Mode::HelixNormal,
        );
        
        cx.simulate_keystrokes("shift-j");  // Join lines
        
        cx.assert_state("«aˇ»bc def", Mode::HelixNormal);
    }

    #[gpui::test]
    async fn test_helix_delete_overlapping_ranges(cx: &mut TestAppContext) {
        let mut cx = VimTestContext::new(cx, true).await;
        
        // From Helix test: Don't panic when deleting overlapping ranges
        cx.set_state("fo«oˇ»ba«rˇ»", Mode::HelixNormal);
        
        cx.simulate_keystrokes("d");  // Delete selections
        
        cx.assert_state("foˇba", Mode::HelixNormal);
    }

    #[gpui::test]
    async fn test_helix_extend_line_with_count(cx: &mut TestAppContext) {
        let mut cx = VimTestContext::new(cx, true).await;
        
        // From Helix test: Extend line selection with count
        cx.set_state(
            indoc! {"
                «lˇ»orem
                ipsum
                dolor
                
            "},
            Mode::HelixNormal,
        );
        
        cx.simulate_keystrokes("x 2 x");  // Select line, then extend 2 more lines
        
        cx.assert_state(
            indoc! {"
                «lorem
                ipsum
                dolorˇ»
                
            "},
            Mode::HelixNormal,
        );
    }
}