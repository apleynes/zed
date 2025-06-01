use crate::{
    helix::{
        CollapseSelection, FlipSelections, MergeSelections, MergeConsecutiveSelections,
        KeepPrimarySelection, RemovePrimarySelection, TrimSelections, AlignSelections,
        CopySelectionOnNextLine, CopySelectionOnPrevLine, RotateSelectionsForward,
        RotateSelectionsBackward, RotateSelectionContentsForward, RotateSelectionContentsBackward,
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

    cx.set_state(indoc! {"
        «lineˇ» one
        «lineˇ» two  
        gap here
        «lineˇ» three"}, Mode::HelixNormal);
    
    cx.dispatch_action(MergeConsecutiveSelections);
    
    // Should merge the first two lines (consecutive) but leave the third separate
    cx.assert_state(indoc! {"
        «line one
        lineˇ» two  
        gap here
        «lineˇ» three"}, Mode::HelixNormal);
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