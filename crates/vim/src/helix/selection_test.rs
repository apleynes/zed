use crate::{
    helix::{
        CollapseSelection, FlipSelections, MergeSelections, MergeConsecutiveSelections,
        KeepPrimarySelection, RemovePrimarySelection, TrimSelections, AlignSelections,
        CopySelectionOnNextLine, CopySelectionOnPrevLine, RotateSelectionsForward,
        RotateSelectionsBackward, RotateSelectionContentsForward, RotateSelectionContentsBackward,
        SplitSelectionOnRegex,
    },
    test::VimTestContext,
    Mode,
};
use indoc::indoc;

#[gpui::test]
async fn test_collapse_selection_single(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    cx.set_state("hello «worldˇ»", Mode::HelixNormal);
    cx.dispatch_action(CollapseSelection);
    cx.assert_state("hello worldˇ", Mode::HelixNormal);
}

#[gpui::test]
async fn test_collapse_selection_multiple(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    cx.set_state("«oneˇ» and «twoˇ» and three", Mode::HelixNormal);
    cx.dispatch_action(CollapseSelection);
    cx.assert_state("oneˇ and twoˇ and three", Mode::HelixNormal);
}

#[gpui::test]
async fn test_flip_selections_single(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    cx.set_state("hello «worldˇ»", Mode::HelixNormal);
    cx.dispatch_action(FlipSelections);
    cx.assert_state("hello «ˇworld»", Mode::HelixNormal);
}

#[gpui::test]
async fn test_flip_selections_multiple(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    cx.set_state("«oneˇ» and «twoˇ»", Mode::HelixNormal);
    cx.dispatch_action(FlipSelections);
    cx.assert_state("«ˇone» and «ˇtwo»", Mode::HelixNormal);
}

#[gpui::test]
async fn test_merge_selections_adjacent(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    cx.set_state("«oneˇ» «twoˇ»", Mode::HelixNormal);
    cx.dispatch_action(MergeSelections);
    cx.assert_state("«one twoˇ»", Mode::HelixNormal);
}

#[gpui::test]
async fn test_merge_selections_overlapping(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    cx.set_state("«helloˇ» wo«rldˇ»", Mode::HelixNormal);
    cx.dispatch_action(MergeSelections);
    cx.assert_state("«hello worldˇ»", Mode::HelixNormal);
}

#[gpui::test]
async fn test_merge_consecutive_selections(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    // Test with truly consecutive selections (adjacent with no gap)
    cx.set_state("«oneˇ»«twoˇ» and «threeˇ»«fourˇ»", Mode::HelixNormal);
    
    cx.dispatch_action(MergeConsecutiveSelections);
    
    // Should merge the consecutive pairs: one+two and three+four
    cx.assert_state("«onetwoˇ» and «threefourˇ»", Mode::HelixNormal);
}

#[gpui::test]
async fn test_keep_primary_selection(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    cx.set_state("«oneˇ» and «twoˇ» and «threeˇ»", Mode::HelixNormal);
    cx.dispatch_action(KeepPrimarySelection);
    cx.assert_state("«oneˇ» and two and three", Mode::HelixNormal);
}

#[gpui::test]
async fn test_remove_primary_selection(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    cx.set_state("«oneˇ» and «twoˇ» and «threeˇ»", Mode::HelixNormal);
    cx.dispatch_action(RemovePrimarySelection);
    cx.assert_state("one and «twoˇ» and «threeˇ»", Mode::HelixNormal);
}

#[gpui::test]
async fn test_trim_selections_whitespace(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    cx.set_state("« hello ˇ» world", Mode::HelixNormal);
    cx.dispatch_action(TrimSelections);
    // Should trim leading and trailing whitespace from selection
    cx.assert_state(" «helloˇ»  world", Mode::HelixNormal);
}

#[gpui::test]
async fn test_trim_selections_multiple(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    cx.set_state("« oneˇ» and «  two  ˇ»", Mode::HelixNormal);
    cx.dispatch_action(TrimSelections);
    // Should trim whitespace from both selections
    cx.assert_state(" «oneˇ» and   «twoˇ»  ", Mode::HelixNormal);
}

#[gpui::test]
async fn test_align_selections_basic(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    cx.set_state(indoc! {"
        «shortˇ»
        «longerwordˇ»
        «midˇ»"}, Mode::HelixNormal);
    
    cx.dispatch_action(AlignSelections);
    
    // Should align all selections to the width of the longest one by adding spaces
    cx.assert_state(indoc! {"
        «short     ˇ»
        «longerwordˇ»
        «mid       ˇ»"}, Mode::HelixNormal);
}


#[gpui::test]
async fn test_copy_selection_on_next_line(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    cx.set_state(indoc! {"
        hello «worldˇ»
        foo bar"}, Mode::HelixNormal);
    
    cx.dispatch_action(CopySelectionOnNextLine);
    
    // Should copy selection to same column position on next line
    cx.assert_state(indoc! {"
        hello «worldˇ»
        foo ba«rˇ»"}, Mode::HelixNormal);
}

#[gpui::test]
async fn test_copy_selection_on_prev_line(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    cx.set_state(indoc! {"
        foo bar
        hello «worldˇ»"}, Mode::HelixNormal);
    
    cx.dispatch_action(CopySelectionOnPrevLine);
    
    // Should copy selection to same column position on previous line
    cx.assert_state(indoc! {"
        foo ba«rˇ»
        hello «worldˇ»"}, Mode::HelixNormal);
}

#[gpui::test]
async fn test_copy_selection_line_boundary(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    // Test copying beyond line length - should clamp to line end
    cx.set_state(indoc! {"
        short
        much longer «lineˇ»"}, Mode::HelixNormal);
    
    cx.dispatch_action(CopySelectionOnPrevLine);
    
    // Should clamp the copied selection to the shorter line length
    cx.assert_state(indoc! {"
        shortˇ
        much longer «lineˇ»"}, Mode::HelixNormal);
}

#[gpui::test]
async fn test_rotate_selections_forward(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    cx.set_state("«oneˇ» «twoˇ» «threeˇ»", Mode::HelixNormal);
    cx.dispatch_action(RotateSelectionsForward);
    
    // Should rotate which selection is primary (main), not move positions
    // The primary selection changes but positions stay the same
    cx.assert_state("«oneˇ» «twoˇ» «threeˇ»", Mode::HelixNormal);
}

#[gpui::test]
async fn test_rotate_selections_backward(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    cx.set_state("«oneˇ» «twoˇ» «threeˇ»", Mode::HelixNormal);
    cx.dispatch_action(RotateSelectionsBackward);
    
    // Should rotate which selection is primary (main), not move positions
    // The primary selection changes but positions stay the same
    cx.assert_state("«oneˇ» «twoˇ» «threeˇ»", Mode::HelixNormal);
}

#[gpui::test]
async fn test_rotate_selection_contents_forward(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    cx.set_state("«oneˇ» «twoˇ» «threeˇ»", Mode::HelixNormal);
    cx.dispatch_action(RotateSelectionContentsForward);
    
    // Content should rotate: last content goes to first position
    // Selections stay in same positions but their contents rotate
    cx.assert_state("«threeˇ» «oneˇ» «twoˇ»", Mode::HelixNormal);
}

#[gpui::test]
async fn test_rotate_selection_contents_backward(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    cx.set_state("«oneˇ» «twoˇ» «threeˇ»", Mode::HelixNormal);
    cx.dispatch_action(RotateSelectionContentsBackward);
    
    // Content should rotate: first content goes to last position
    // Selections stay in same positions but their contents rotate
    cx.assert_state("«twoˇ» «threeˇ» «oneˇ»", Mode::HelixNormal);
}

#[gpui::test]
async fn test_selection_operations_empty_selections(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    // Test operations with empty selections (cursors)
    cx.set_state("hello ˇworld ˇtest", Mode::HelixNormal);
    
    // These operations should handle empty selections gracefully
    cx.dispatch_action(CollapseSelection);
    cx.assert_state("hello ˇworld ˇtest", Mode::HelixNormal);
    
    cx.dispatch_action(FlipSelections);
    cx.assert_state("hello ˇworld ˇtest", Mode::HelixNormal);
}

#[gpui::test]
async fn test_selection_operations_single_selection(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    cx.set_state("hello «worldˇ»", Mode::HelixNormal);
    
    // Operations that require multiple selections should handle single selection gracefully
    cx.dispatch_action(MergeSelections);
    cx.assert_state("hello «worldˇ»", Mode::HelixNormal);
    
    cx.dispatch_action(RotateSelectionsForward);
    cx.assert_state("hello «worldˇ»", Mode::HelixNormal);
}

#[gpui::test]
async fn test_selection_workflow_comprehensive(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    // Start with a realistic text scenario
    cx.set_state(indoc! {"
        function «nameˇ»(param) {
            return «nameˇ».length;
        }
        
        const «nameˇ» = 'test';"}, Mode::HelixNormal);
    
    // First, flip all selections to see the anchor points
    cx.dispatch_action(FlipSelections);
    cx.assert_state(indoc! {"
        function «ˇname»(param) {
            return «ˇname».length;
        }
        
        const «ˇname» = 'test';"}, Mode::HelixNormal);
    
    // Then flip back
    cx.dispatch_action(FlipSelections);
    cx.assert_state(indoc! {"
        function «nameˇ»(param) {
            return «nameˇ».length;
        }
        
        const «nameˇ» = 'test';"}, Mode::HelixNormal);
    
    // Keep only the primary selection
    cx.dispatch_action(KeepPrimarySelection);
    cx.assert_state(indoc! {"
        function «nameˇ»(param) {
            return name.length;
        }
        
        const name = 'test';"}, Mode::HelixNormal);
}

#[gpui::test]
async fn test_rotate_selections_integration_comprehensive(cx: &mut gpui::TestAppContext) {
    // Manually developer-verified test case. Coding assistant: DO NOT MODIFY.
    let mut cx = VimTestContext::new(cx, true).await;

    // Reset primary index to match Helix behavior when new selections are created
    crate::helix::selections::reset_primary_selection_index();

    // Test the specific workflow: 5 words selected, rotate twice, then drop 3rd selection
    cx.set_state("«oneˇ» «twoˇ» «threeˇ» «fourˇ» «fiveˇ»", Mode::HelixNormal);
    
    // Initial state: [one, two, three, four, five] - one is primary (first)
    
    // First rotation forward - cyclic rotation: move first to end
    cx.dispatch_action(RotateSelectionsForward);
    // Expected: two is now primary (first)
    cx.assert_state("«oneˇ» «twoˇ» «threeˇ» «fourˇ» «fiveˇ»", Mode::HelixNormal);
    
    // Second rotation forward - move first to end again
    cx.dispatch_action(RotateSelectionsForward);
    // Expected: three is now primary (first)
    cx.assert_state("«oneˇ» «twoˇ» «threeˇ» «fourˇ» «fiveˇ»", Mode::HelixNormal);
    
    // Now remove primary selection (should remove "three" selection, which is now first)
    cx.dispatch_action(RemovePrimarySelection);
    
    // Expected: Should remove "three" selection (the current primary) and leave the rest
    cx.assert_state("«oneˇ» «twoˇ» three «fourˇ» «fiveˇ»", Mode::HelixNormal);
}

#[gpui::test]
async fn test_rotate_selections_primary_tracking(cx: &mut gpui::TestAppContext) {
    // Manually developer-verified test case. Coding assistant: DO NOT MODIFY.
    let mut cx = VimTestContext::new(cx, true).await;

    // Test that primary selection tracking works correctly
    cx.set_state("«aˇ» «bˇ» «cˇ»", Mode::HelixNormal);
    
    // Rotate forward: selection (not visible with cursor and selection markings, only after removing primary will the effect be visible)
    cx.dispatch_action(RotateSelectionsForward);
    cx.assert_state("«aˇ» «bˇ» «cˇ»", Mode::HelixNormal);
    
    // Remove primary (should remove "b" selection)
    cx.dispatch_action(RemovePrimarySelection);
    cx.assert_state("«aˇ» b «cˇ»", Mode::HelixNormal);
}

#[gpui::test]
async fn test_merge_selections_comprehensive(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    // Test merge selections with multiple non-adjacent selections
    cx.set_state("«oneˇ» middle «twoˇ» gap «threeˇ»", Mode::HelixNormal);
    
    cx.dispatch_action(MergeSelections);
    
    // Should create one selection spanning from start of first to end of last
    // This should include all the text in between, not just the selected parts
    cx.assert_state("«one middle two gap threeˇ»", Mode::HelixNormal);
}

#[gpui::test]
async fn test_merge_selections_with_gaps(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    // Test with larger gaps between selections
    cx.set_state(indoc! {"
        «firstˇ»
        
        some text here
        
        «secondˇ»
        
        more text
        
        «thirdˇ»"}, Mode::HelixNormal);
    
    cx.dispatch_action(MergeSelections);
    
    // Should merge everything from first selection to last selection
    cx.assert_state(indoc! {"
        «first
        
        some text here
        
        second
        
        more text
        
        thirdˇ»"}, Mode::HelixNormal);
}

#[gpui::test]
async fn test_merge_selections_key_binding(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    // Test that Alt-- (merge selections) key binding works
    cx.set_state("«oneˇ» and «twoˇ» and «threeˇ»", Mode::HelixNormal);
    
    // Simulate the actual key binding: alt-minus
    cx.simulate_keystrokes("alt-minus");
    
    // Should merge all selections into one
    cx.assert_state("«one and two and threeˇ»", Mode::HelixNormal);
}

#[gpui::test]
async fn test_rotate_selections_key_binding(cx: &mut gpui::TestAppContext) {
    // Manually developer-verified test case. Coding assistant: DO NOT MODIFY.
    // You can't really visually see the change because it only changes the 
    // primary selection index, not the actual selections.
    let mut cx = VimTestContext::new(cx, true).await;

    // Test that ( and ) (rotate selections) key bindings work
    cx.set_state("«oneˇ» «twoˇ» «threeˇ»", Mode::HelixNormal);
    
    // Simulate the actual key binding: )
    cx.simulate_keystrokes(")");
    
    // Should rotate selections forward
    cx.assert_state("«oneˇ» «twoˇ» «threeˇ»", Mode::HelixNormal);
    
    // Simulate the actual key binding: (
    cx.simulate_keystrokes("(");
    
    // Should rotate selections backward
    cx.assert_state("«oneˇ» «twoˇ» «threeˇ»", Mode::HelixNormal);
}

#[gpui::test]
async fn test_remove_primary_selection_key_binding(cx: &mut gpui::TestAppContext) {
    // Manually developer-verified test case. Coding assistant: DO NOT MODIFY.
    let mut cx = VimTestContext::new(cx, true).await;

    // Test that Alt-, (remove primary selection) key binding works
    cx.set_state("«oneˇ» «twoˇ» «threeˇ»", Mode::HelixNormal);
    
    // Simulate the actual key binding: alt-,
    cx.simulate_keystrokes("alt-,");
    
    // Should remove the primary (first) selection
    cx.assert_state("one «twoˇ» «threeˇ»", Mode::HelixNormal);
}

#[gpui::test]
async fn test_user_reported_rotate_and_remove_workflow(cx: &mut gpui::TestAppContext) {
    // Manually developer-verified test case. Coding assistant: DO NOT MODIFY.
    let mut cx = VimTestContext::new(cx, true).await;

    // Reset primary index to match Helix behavior when new selections are created
    crate::helix::selections::reset_primary_selection_index();

    // Test the exact workflow the user described:
    // 5 words selected, rotate twice, then remove 3rd selection
    cx.set_state("«oneˇ» «twoˇ» «threeˇ» «fourˇ» «fiveˇ»", Mode::HelixNormal);
    
    // First rotation forward: )
    cx.simulate_keystrokes(")");
    // Expected: two is now primary
    cx.assert_state("«oneˇ» «twoˇ» «threeˇ» «fourˇ» «fiveˇ»", Mode::HelixNormal);
    
    // Second rotation forward: )
    cx.simulate_keystrokes(")");
    // Expected: three is now primary
    cx.assert_state("«oneˇ» «twoˇ» «threeˇ» «fourˇ» «fiveˇ»", Mode::HelixNormal);
    
    // Now remove primary selection: alt-,
    // This should remove "three" selection (the 3rd original selection, now primary)
    cx.simulate_keystrokes("alt-,");
    
    // Expected: Should remove "three" selection and leave the rest
    cx.assert_state("«oneˇ» «twoˇ» three «fourˇ» «fiveˇ»", Mode::HelixNormal);
}

#[gpui::test]
async fn test_rotate_selections_reset_primary_index_after_new_selections(cx: &mut gpui::TestAppContext) {
    // Manually developer-verified test case. Coding assistant: Complete this test.
    let mut cx = VimTestContext::new(cx, true).await;

    crate::helix::selections::reset_primary_selection_index();

    // Test that Alt-, (remove primary selection) key binding works
    cx.set_state("«oneˇ» «twoˇ» «threeˇ»", Mode::HelixNormal);
    
    // Simulate the actual key binding: alt-,
    cx.simulate_keystrokes("alt-,");
    
    // Should remove the primary (first) selection
    cx.assert_state("one «twoˇ» «threeˇ»", Mode::HelixNormal);

    // Select whole line with x (creates new selection)
    cx.simulate_keystrokes("x");

    // Should create one selection
    cx.assert_state("«one two threeˇ»", Mode::HelixNormal);

    // Use regex split to split on space
    // TODO

    // Use the split selection on regex functionality (S command in Helix)
    // This should split the line selection on spaces to create three selections again
    cx.dispatch_action(crate::helix::SplitSelectionOnRegex);
    // The split action should automatically create the three word selections and reset primary index
    // We expect the split to work correctly and create three selections
    cx.assert_state("«oneˇ» «twoˇ» «threeˇ»", Mode::HelixNormal);

    // Remove primary selection (should remove selection on "one" now since primary index was reset by 
    // creating new selections via split)
    cx.simulate_keystrokes("alt-,");

    // Should remove selection on "one" (the first selection, since primary index was reset to 0)
    cx.assert_state("one «twoˇ» «threeˇ»", Mode::HelixNormal);
}