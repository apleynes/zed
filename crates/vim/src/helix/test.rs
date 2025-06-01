use indoc::indoc;
use crate::{state::Mode, test::VimTestContext, helix::*};

#[gpui::test]
async fn test_helix_collapse_selection(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    cx.set_state("hello «worldˇ»", Mode::HelixNormal);

    // Use the new helix action directly
    cx.dispatch_action(CollapseSelection);

    // Should collapse to cursor position
    cx.assert_state("hello worldˇ", Mode::HelixNormal);
}

#[gpui::test]
async fn test_helix_flip_selections(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    cx.set_state("hello «worldˇ»", Mode::HelixNormal);

    // Use the new helix action directly
    cx.dispatch_action(FlipSelections);

    // Should flip selection direction
    cx.assert_state("hello «ˇworld»", Mode::HelixNormal);
}

#[gpui::test]
async fn test_helix_merge_selections(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    // Set up multiple selections
    cx.set_state("«oneˇ» and «twoˇ»", Mode::HelixNormal);

    // Use the new helix action directly
    cx.dispatch_action(MergeSelections);

    // Should merge into one selection spanning both
    cx.assert_state("«one and twoˇ»", Mode::HelixNormal);
}

#[gpui::test]
async fn test_helix_keep_primary_selection(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    // Set up multiple selections
    cx.set_state("«oneˇ» and «twoˇ»", Mode::HelixNormal);

    // Use the new helix action directly
    cx.dispatch_action(KeepPrimarySelection);

    // Should keep only the first (primary) selection
    cx.assert_state("«oneˇ» and two", Mode::HelixNormal);
}

#[gpui::test]
async fn test_helix_match_brackets(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    cx.set_state(
        indoc! {"
        function(ˇarg) {
            return arg;
        }"},
        Mode::HelixNormal,
    );

    // Use the new helix match brackets action directly
    cx.dispatch_action(MatchBrackets);

    // Should move to matching bracket
    cx.assert_state(
        indoc! {"
        function(argˇ) {
            return arg;
        }"},
        Mode::HelixNormal,
    );
}

#[gpui::test]
async fn test_helix_copy_selection_on_next_line(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    cx.set_state(
        indoc! {"
        hello «worldˇ»
        foo bar"},
        Mode::HelixNormal,
    );

    // Use the new helix action directly
    cx.dispatch_action(CopySelectionOnNextLine);

    // Should add selection on next line
    cx.assert_state(
        indoc! {"
        hello «worldˇ»
        foo ba«rˇ»"},
        Mode::HelixNormal,
    );
}

#[gpui::test]
async fn test_helix_mode_preservation(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    cx.set_state("hello «worldˇ»", Mode::HelixNormal);

    // Verify we start in HelixNormal mode
    assert_eq!(cx.mode(), Mode::HelixNormal);

    // Perform a helix operation
    cx.dispatch_action(CollapseSelection);

    // Mode should be preserved
    assert_eq!(cx.mode(), Mode::HelixNormal);
}

#[gpui::test]
async fn test_helix_sub_keymap_match_mode(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    // Test the sub-keymap approach for match mode
    cx.set_state(
        indoc! {"
        function(ˇarg) {
            return arg;
        }"},
        Mode::HelixNormal,
    );

    // Use the sub-keymap sequence "m m" to trigger match brackets
    cx.simulate_keystrokes("m m");

    // Should move to matching bracket
    cx.assert_state(
        indoc! {"
        function(argˇ) {
            return arg;
        }"},
        Mode::HelixNormal,
    );

    // Verify mode is still HelixNormal (no mode switching)
    assert_eq!(cx.mode(), Mode::HelixNormal);
}

#[gpui::test]
async fn test_helix_mode_switching_to_insert(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    // Start in HelixNormal mode
    cx.set_state("hello worldˇ", Mode::HelixNormal);
    
    // Verify we start in HelixNormal mode
    assert_eq!(cx.mode(), Mode::HelixNormal);

    // Press 'i' to enter insert mode
    cx.simulate_keystrokes("i");
    
    // Verify mode switched to Insert
    assert_eq!(cx.mode(), Mode::Insert);
    
    // Press escape to return to HelixNormal mode
    cx.simulate_keystrokes("escape");
    
    // Verify we're back in HelixNormal mode
    assert_eq!(cx.mode(), Mode::HelixNormal);
}

#[gpui::test]
async fn test_helix_multiple_selections_workflow(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    // Start with multiple selections
    cx.set_state("«oneˇ» «twoˇ» «threeˇ»", Mode::HelixNormal);

    // Verify we have multiple selections
    let selections = cx.update_editor(|editor, _, cx| {
        editor.selections.all_adjusted(cx).len()
    });
    assert_eq!(selections, 3);

    // Merge consecutive selections
    cx.dispatch_action(MergeConsecutiveSelections);

    // Should still be in HelixNormal mode
    assert_eq!(cx.mode(), Mode::HelixNormal);

    // Keep only primary selection
    cx.dispatch_action(KeepPrimarySelection);

    // Should still be in HelixNormal mode with one selection
    assert_eq!(cx.mode(), Mode::HelixNormal);
    let final_selections = cx.update_editor(|editor, _, cx| {
        editor.selections.all_adjusted(cx).len()
    });
    assert_eq!(final_selections, 1);
}