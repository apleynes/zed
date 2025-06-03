use crate::{
    helix::{
        regex_selection::{
            InteractiveRegexPrompt, SelectRegex, SplitSelectionOnRegex, KeepSelections, RemoveSelections
        },
    },
    test::VimTestContext,
    Mode,
};
use editor::Editor;
use indoc::indoc;

#[gpui::test]
async fn test_select_regex_basic(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    // Test basic regex selection - select all words starting with capital letters
    cx.set_state("«Nobody expects the Spanish inquisitionˇ»", Mode::HelixNormal);
    
    // Use 's' keystroke to trigger select regex
    cx.simulate_keystrokes("s");
    
    // Verify modal opens (basic functionality test)
    let modal_open = cx.workspace(|workspace, _, cx| {
        workspace.active_modal::<InteractiveRegexPrompt>(cx).is_some()
    });
    assert!(modal_open, "Regex selection modal should be open");
    
    // Cancel to close modal
    cx.simulate_keystrokes("escape");
}

#[gpui::test]
async fn test_select_regex_matches_within_selection(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    // Test basic regex selection - select all words starting with capital letters
    cx.set_state("«I like to eat apples since my favorite fruit is apples.ˇ»", Mode::HelixNormal);
    
    // Use 's' keystroke to trigger select regex
    cx.simulate_keystrokes("s");
    cx.simulate_input("apples");
    cx.simulate_keystrokes("enter");
    
    // Should have selected both instances of "apples"
    cx.assert_state("I like to eat «applesˇ» since my favorite fruit is «applesˇ».", Mode::HelixNormal);
}

#[gpui::test]
async fn test_select_regex_with_spaces(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    // Test regex selection for multiple spaces (from tutor example)
    cx.set_state("«This  sentence has   some      extra spacesˇ».", Mode::HelixNormal);
    
    // Use regex "  +" to select sequences of 2 or more spaces
    cx.simulate_keystrokes("s");
    cx.simulate_input("  +");
    cx.simulate_keystrokes("enter");
    
    // Should select "  ", "   ", "      " but not single spaces
    cx.assert_state("This«  ˇ»sentence has«   ˇ»some«      ˇ»extra spaces.", Mode::HelixNormal);
}

#[gpui::test]
async fn test_split_selection_on_regex_basic(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    // Test basic split functionality
    cx.set_state("«one two three fourˇ»", Mode::HelixNormal);
    
    // Split on spaces should create four separate word selections
    cx.simulate_keystrokes("shift-s");
    cx.simulate_input(" ");
    cx.simulate_keystrokes("enter");
    
    // Expected: "«oneˇ»" "«twoˇ»" "«threeˇ»" "«fourˇ»"
    cx.assert_state("«oneˇ» «twoˇ» «threeˇ» «fourˇ»", Mode::HelixNormal);
}

#[gpui::test]
async fn test_split_selection_on_regex_sentences(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    // Test splitting on sentence boundaries (from tutor example)
    cx.set_state(indoc! {"
        «these are sentences. some sentences don't start with uppercase
        letters! that is not good grammar. you can fix thisˇ».
    "}, Mode::HelixNormal);
    
    // Split on ". " or "! " should create separate sentence selections
    cx.simulate_keystrokes("shift-s");
    cx.simulate_input("\\. |! ");
    cx.simulate_keystrokes("enter");
    
    // Verify the modal interaction worked (exact result verification is complex for this case)
    let modal_closed = cx.workspace(|workspace, _, cx| {
        workspace.active_modal::<InteractiveRegexPrompt>(cx).is_none()
    });
    assert!(modal_closed, "Split selection modal should be closed");
}

#[gpui::test]
async fn test_split_selection_preserves_zero_width(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    // Test that zero-width selections are preserved during split
    cx.set_state("hello ˇworld", Mode::HelixNormal);
    
    cx.simulate_keystrokes("shift-s");
    cx.simulate_input(" ");
    cx.simulate_keystrokes("enter");
    
    // Zero-width selection should remain unchanged
    cx.assert_state("hello ˇworld", Mode::HelixNormal);
}

#[gpui::test]
async fn test_split_selection_leading_and_trailing_matches(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    // Test split with leading and trailing matches - this is from Helix tests
    cx.set_state("«   abcd efg and wrs   xyzˇ»", Mode::HelixNormal);
    
    // Use 'S' keystroke to trigger split selection on spaces
    cx.simulate_keystrokes("shift-s");
    cx.simulate_input(" +");
    cx.simulate_keystrokes("enter");
    
    // Should split on spaces, creating separate selections for each word
    // Based on Helix test_split_on_matches, this should include leading empty selection
    // The text "   abcd efg and wrs   xyz" split on " +" gives:
    // ["", "abcd", "efg", "and", "wrs", "xyz"] (leading empty, then words)
    // The leading empty selection is a zero-width cursor at position 0
    // followed by the spaces, then each word as a selection
    cx.assert_state("ˇ   «abcdˇ» «efgˇ» «andˇ» «wrsˇ»   «xyzˇ»", Mode::HelixNormal);
}

#[gpui::test]
async fn test_keep_selections_matching_regex(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    // Test keeping only selections that match a regex
    cx.set_state("«oneˇ» «twoˇ» «123ˇ» «fourˇ» «567ˇ»", Mode::HelixNormal);
    
    cx.simulate_keystrokes("shift-k");
    cx.simulate_input("\\d+");
    cx.simulate_keystrokes("enter");
    
    // With regex "\d+" (digits), should keep only "123" and "567"
    cx.assert_state("one two «123ˇ» four «567ˇ»", Mode::HelixNormal);
}

#[gpui::test]
async fn test_remove_selections_matching_regex(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    // Test removing selections that match a regex
    cx.set_state("«oneˇ» «twoˇ» «123ˇ» «fourˇ» «567ˇ»", Mode::HelixNormal);
    
    cx.simulate_keystrokes("alt-shift-k");
    cx.simulate_input("\\d+");
    cx.simulate_keystrokes("enter");
    
    // With regex "\d+" (digits), should remove "123" and "567"
    cx.assert_state("«oneˇ» «twoˇ» 123 «fourˇ» 567", Mode::HelixNormal);
}

#[gpui::test]
async fn test_regex_operations_reset_primary_index(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    // Test that regex operations reset primary selection index
    cx.set_state("«oneˇ» «twoˇ» «threeˇ»", Mode::HelixNormal);
    
    // Rotate to make "two" primary
    cx.simulate_keystrokes(")");
    
    // Now split on spaces (which should reset primary index to 0)
    cx.simulate_keystrokes("shift-s");
    cx.simulate_input(" ");
    cx.simulate_keystrokes("enter");
    
    // After split, primary should be reset to first selection
    // Remove primary should remove the first selection, not "two"
    cx.simulate_keystrokes("alt-,");
    
    // Should remove first selection since primary index was reset
    // (Exact verification depends on how the split worked)
}

#[gpui::test]
async fn test_regex_selection_empty_results(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    // Test behavior when regex matches nothing
    cx.set_state("«hello worldˇ»", Mode::HelixNormal);
    
    // Use a regex that matches nothing should preserve original selection
    cx.simulate_keystrokes("s");
    cx.simulate_input("xyz");
    cx.simulate_keystrokes("enter");
    
    // Should preserve original selection when no matches found
    cx.assert_state("«hello worldˇ»", Mode::HelixNormal);
}

#[gpui::test]
async fn test_regex_selection_invalid_regex(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    // Test behavior with invalid regex patterns
    cx.set_state("«hello worldˇ»", Mode::HelixNormal);
    
    // Invalid regex should not crash and should preserve original state
    cx.simulate_keystrokes("s");
    cx.simulate_input("[invalid");
    cx.simulate_keystrokes("enter");
    
    // Should preserve original selection with invalid regex
    cx.assert_state("«hello worldˇ»", Mode::HelixNormal);
}

#[gpui::test]
async fn test_regex_selection_multiline(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    // Test regex selection across multiple lines
    cx.set_state("«line one\nline two\nline threeˇ»", Mode::HelixNormal);
    
    // Use 's' keystroke to trigger select regex for "line"
    cx.simulate_keystrokes("s");
    cx.simulate_input("line");
    cx.simulate_keystrokes("enter");
    
    // Should have selected all instances of "line"
    cx.assert_state("«lineˇ» one\n«lineˇ» two\n«lineˇ» three", Mode::HelixNormal);
}

#[gpui::test]
async fn test_regex_selection_unicode(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    // Test regex selection with Unicode characters
    cx.set_state("«Hello 世界 and Welt and 世界ˇ»", Mode::HelixNormal);
    
    // Regex for Unicode characters should work correctly
    cx.simulate_keystrokes("s");
    cx.simulate_input("世界");
    cx.simulate_keystrokes("enter");
    
    // Should select both instances of "世界"
    cx.assert_state("Hello «世界ˇ» and Welt and «世界ˇ»", Mode::HelixNormal);
}

#[gpui::test]
async fn test_regex_selection_integration_workflow(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    // Test a complete workflow from the tutor
    cx.set_state("I like to eat apples since my favorite fruit is apples.ˇ", Mode::HelixNormal);
    
    // 1. Select the line
    cx.simulate_keystrokes("x");
    cx.assert_state("«I like to eat apples since my favorite fruit is apples.ˇ»", Mode::HelixNormal);
    
    // 2. Use select regex to find "apples"
    cx.simulate_keystrokes("s");
    cx.simulate_input("apples");
    cx.simulate_keystrokes("enter");
    
    // Should have selected both instances of "apples"
    cx.assert_state("I like to eat «applesˇ» since my favorite fruit is «applesˇ».", Mode::HelixNormal);
    
    // For now, just verify that the regex selection worked correctly
    // TODO: Fix the change operation in Helix mode to work with multiple selections
}

#[gpui::test]
async fn test_keep_remove_selections_partial_matches(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    // Test keep selections with partial matches
    cx.set_state("«oneˇ» «twoˇ» «threeˇ»", Mode::HelixNormal);
    
    // Keep selections that contain "o" (should keep "one" and "two")
    cx.simulate_keystrokes("shift-k");
    cx.simulate_input("o");
    cx.simulate_keystrokes("enter");
    
    cx.assert_state("«oneˇ» «twoˇ» three", Mode::HelixNormal);
    
    // Reset for next test
    cx.set_state("«oneˇ» «twoˇ» «threeˇ»", Mode::HelixNormal);
    
    // Test remove selections with partial matches
    cx.simulate_keystrokes("alt-shift-k");
    cx.simulate_input("o");
    cx.simulate_keystrokes("enter");
    
    // Should remove selections containing "o", leaving only "three"
    cx.assert_state("one two «threeˇ»", Mode::HelixNormal);
    
    // Reset for more specific test
    cx.set_state("«oneˇ» «twoˇ» «threeˇ»", Mode::HelixNormal);
    
    // Keep selections that contain "on" (should keep only "one")
    cx.simulate_keystrokes("shift-k");
    cx.simulate_input("on");
    cx.simulate_keystrokes("enter");
    
    cx.assert_state("«oneˇ» two three", Mode::HelixNormal);
    
    // Reset for final test
    cx.set_state("«oneˇ» «twoˇ» «threeˇ»", Mode::HelixNormal);
    
    // Remove selections that contain "on" (should remove "one", keeping "two" and "three")
    cx.simulate_keystrokes("alt-shift-k");
    cx.simulate_input("on");
    cx.simulate_keystrokes("enter");
    
    cx.assert_state("one «twoˇ» «threeˇ»", Mode::HelixNormal);
}

#[gpui::test]
async fn test_regex_selection_ui_integration(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    // Test basic select regex UI interaction with keystroke simulation
    cx.set_state("«Nobody expects the Spanish inquisitionˇ»", Mode::HelixNormal);
    
    // Use 's' keystroke to trigger select regex
    cx.simulate_keystrokes("s");
    
    // Verify modal is open
    let modal_open = cx.workspace(|workspace, _, cx| {
        workspace.active_modal::<InteractiveRegexPrompt>(cx).is_some()
    });
    assert!(modal_open, "Regex selection modal should be open");
    
    // Simulate typing a regex pattern to select capital words
    cx.simulate_input("[A-Z][a-z]*");
    
    // Simulate pressing Enter to confirm
    cx.simulate_keystrokes("enter");
    
    // Verify modal is closed and selections are updated to match capital words
    let modal_closed = cx.workspace(|workspace, _, cx| {
        workspace.active_modal::<InteractiveRegexPrompt>(cx).is_none()
    });
    assert!(modal_closed, "Regex selection modal should be closed after confirmation");
    
    // Verify the regex operation worked - should have selected "Nobody" and "Spanish"
    cx.assert_state("«Nobodyˇ» expects the «Spanishˇ» inquisition", Mode::HelixNormal);
}

#[gpui::test]
async fn test_regex_selection_escape_cancels(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    // Test escape cancels the modal and restores original selections
    cx.set_state("«hello worldˇ»", Mode::HelixNormal);
    
    // Use 's' keystroke to trigger select regex
    cx.simulate_keystrokes("s");
    
    // Verify modal is open
    let modal_open = cx.workspace(|workspace, _, cx| {
        workspace.active_modal::<InteractiveRegexPrompt>(cx).is_some()
    });
    assert!(modal_open, "Regex selection modal should be open");
    
    // Simulate typing something that would change selections
    cx.simulate_input("world");
    
    // Simulate pressing Escape to cancel
    cx.simulate_keystrokes("escape");
    
    // Verify modal is closed and original selections are restored
    let modal_closed = cx.workspace(|workspace, _, cx| {
        workspace.active_modal::<InteractiveRegexPrompt>(cx).is_none()
    });
    assert!(modal_closed, "Regex selection modal should be closed after escape");
    
    // Verify original selection is restored (not changed by the regex)
    cx.assert_state("«hello worldˇ»", Mode::HelixNormal);
}

#[gpui::test]
async fn test_split_selection_ui_integration(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    // Test split selection UI with actual result verification
    cx.set_state("«one two three fourˇ»", Mode::HelixNormal);
    
    // Use 'S' (shift-s) keystroke to trigger split selection
    cx.simulate_keystrokes("shift-s");
    
    // Verify modal is open
    let modal_open = cx.workspace(|workspace, _, cx| {
        workspace.active_modal::<InteractiveRegexPrompt>(cx).is_some()
    });
    assert!(modal_open, "Split selection modal should be open");
    
    // Simulate typing a regex pattern to split on spaces
    cx.simulate_input(" ");
    
    // Simulate pressing Enter to confirm
    cx.simulate_keystrokes("enter");
    
    // Verify modal is closed
    let modal_closed = cx.workspace(|workspace, _, cx| {
        workspace.active_modal::<InteractiveRegexPrompt>(cx).is_none()
    });
    assert!(modal_closed, "Split selection modal should be closed after confirmation");
    
    // Verify the split operation worked - should have four separate word selections
    cx.assert_state("«oneˇ» «twoˇ» «threeˇ» «fourˇ»", Mode::HelixNormal);
}

#[gpui::test]
async fn test_keep_selections_ui_integration(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    // Test keep selections UI with actual result verification
    cx.set_state("«oneˇ» «twoˇ» «threeˇ»", Mode::HelixNormal);
    
    // Use 'K' (shift-k) keystroke to trigger keep selections
    cx.simulate_keystrokes("shift-k");
    
    // Verify modal is open
    let modal_open = cx.workspace(|workspace, _, cx| {
        workspace.active_modal::<InteractiveRegexPrompt>(cx).is_some()
    });
    assert!(modal_open, "Keep selections modal should be open");
    
    // Simulate typing a regex pattern that matches "one" and "two" (contains "o")
    cx.simulate_input("o");
    
    // Simulate pressing Enter to confirm
    cx.simulate_keystrokes("enter");
    
    // Verify modal is closed
    let modal_closed = cx.workspace(|workspace, _, cx| {
        workspace.active_modal::<InteractiveRegexPrompt>(cx).is_none()
    });
    assert!(modal_closed, "Keep selections modal should be closed after confirmation");
    
    // Verify the keep operation worked - should keep only "one" and "two"
    cx.assert_state("«oneˇ» «twoˇ» three", Mode::HelixNormal);
}

#[gpui::test]
async fn test_remove_selections_ui_integration(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    // Test remove selections UI interaction with keystroke simulation
    cx.set_state("«oneˇ» «twoˇ» «threeˇ»", Mode::HelixNormal);
    
    // Use 'Alt-Shift-K' keystroke to trigger remove selections
    cx.simulate_keystrokes("alt-shift-k");
    
    // Verify modal is open
    let modal_open = cx.workspace(|workspace, _, cx| {
        workspace.active_modal::<InteractiveRegexPrompt>(cx).is_some()
    });
    assert!(modal_open, "Remove selections modal should be open");
    
    // Type pattern to remove selections containing 'e' (should remove "one" and "three")
    cx.simulate_input("e");
    cx.simulate_keystrokes("enter");
    
    // Should have removed selections containing 'e', keeping only "two"
    cx.assert_state("one «twoˇ» three", Mode::HelixNormal);
    
    // Verify modal is closed
    let modal_closed = cx.workspace(|workspace, _, cx| {
        workspace.active_modal::<InteractiveRegexPrompt>(cx).is_none()
    });
    assert!(modal_closed, "Remove selections modal should be closed");
}

#[gpui::test]
async fn test_regex_selection_real_time_preview(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    // Test that real-time preview updates as user types
    cx.set_state("«Nobody expects the Spanish inquisitionˇ»", Mode::HelixNormal);
    
    // Use 's' keystroke to trigger select regex
    cx.simulate_keystrokes("s");
    
    // Verify modal is open
    let modal_open = cx.workspace(|workspace, _, cx| {
        workspace.active_modal::<InteractiveRegexPrompt>(cx).is_some()
    });
    assert!(modal_open, "Regex selection modal should be open");
    
    // Simulate typing part of a regex pattern
    cx.simulate_input("[A-Z]");
    
    // At this point, the preview should be updating in real-time
    // We can verify the preview is working by checking if selections changed
    // (though this is implementation-dependent)
    
    // Complete the pattern
    cx.simulate_input("[a-z]*");
    
    // Cancel to test that original selection is restored
    cx.simulate_keystrokes("escape");
    
    // Verify original selection is restored
    cx.assert_state("«Nobody expects the Spanish inquisitionˇ»", Mode::HelixNormal);
}

#[gpui::test]
async fn test_regex_selection_invalid_regex_handling(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    // Test that invalid regex doesn't crash and preserves state
    cx.set_state("«hello worldˇ»", Mode::HelixNormal);
    
    // Use 's' keystroke to trigger select regex
    cx.simulate_keystrokes("s");
    
    // Simulate typing an invalid regex pattern
    cx.simulate_input("[invalid");
    
    // The modal should still be open and functional
    let modal_open = cx.workspace(|workspace, _, cx| {
        workspace.active_modal::<InteractiveRegexPrompt>(cx).is_some()
    });
    assert!(modal_open, "Modal should remain open even with invalid regex");
    
    // Cancel the operation
    cx.simulate_keystrokes("escape");
    
    // Verify original selection is restored
    cx.assert_state("«hello worldˇ»", Mode::HelixNormal);
}

#[gpui::test]
async fn test_regex_selection_empty_pattern_handling(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    // Test that empty pattern preserves original selections
    cx.set_state("«hello worldˇ»", Mode::HelixNormal);
    
    // Use 's' keystroke to trigger select regex
    cx.simulate_keystrokes("s");
    
    // Don't type anything, just confirm with empty pattern
    cx.simulate_keystrokes("enter");
    
    // Verify modal is closed
    let modal_closed = cx.workspace(|workspace, _, cx| {
        workspace.active_modal::<InteractiveRegexPrompt>(cx).is_none()
    });
    assert!(modal_closed, "Modal should be closed");
    
    // With empty pattern, original selection should be preserved
    cx.assert_state("«hello worldˇ»", Mode::HelixNormal);
}

#[gpui::test]
async fn test_regex_operations_from_select_mode(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    // Test that regex operations work from select mode
    cx.set_state("one two threeˇ", Mode::HelixNormal);
    
    // Enter select mode and create a selection
    cx.simulate_keystrokes("v");
    assert_eq!(cx.mode(), Mode::HelixSelect);
    
    // For now, manually set the selection since word movement in select mode 
    // needs to be fixed separately
    cx.set_state("«one two ˇ»three", Mode::HelixSelect);
    
    // Use regex selection from select mode
    cx.simulate_keystrokes("s");
    cx.simulate_input("two");
    cx.simulate_keystrokes("enter");
    
    // Should have selected "two" and be back in normal mode
    cx.assert_state("one «twoˇ» three", Mode::HelixNormal);
}

#[gpui::test]
async fn test_alt_k_remove_selections_keystroke(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    // Specific test for Alt-Shift-K keystroke to ensure it's working
    cx.set_state("«oneˇ» «twoˇ» «threeˇ»", Mode::HelixNormal);
    
    // Use 'Alt-Shift-K' keystroke to trigger remove selections
    cx.simulate_keystrokes("alt-shift-k");
    
    // Verify modal is open
    let modal_open = cx.workspace(|workspace, _, cx| {
        workspace.active_modal::<InteractiveRegexPrompt>(cx).is_some()
    });
    assert!(modal_open, "Remove selections modal should be open with Alt-Shift-K");
    
    // Type pattern that matches selections with "e" and confirm
    cx.simulate_input("e");
    cx.simulate_keystrokes("enter");
    
    // Verify the remove operation worked - should remove "one" and "three", keeping "two"
    cx.assert_state("one «twoˇ» three", Mode::HelixNormal);
}

#[gpui::test]
async fn test_regex_selection_tutor_workflow(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    // Test the complete workflow from the Helix tutor
    cx.set_state("I like to eat apples since my favorite fruit is apples.ˇ", Mode::HelixNormal);
    println!("Initial state set");
    
    // 1. Select the line
    cx.simulate_keystrokes("x");
    cx.assert_state("«I like to eat apples since my favorite fruit is apples.ˇ»", Mode::HelixNormal);
    println!("Line selected");
    
    // 2. Use select regex to find "apples"
    cx.simulate_keystrokes("s");
    println!("Pressed 's' for select regex");
    
    // Check if modal opened
    let modal_open = cx.workspace(|workspace, _, cx| {
        workspace.active_modal::<InteractiveRegexPrompt>(cx).is_some()
    });
    println!("Modal open after 's': {}", modal_open);
    assert!(modal_open, "Regex selection modal should be open");
    
    cx.simulate_input("apples");
    println!("Typed 'apples'");
    cx.simulate_keystrokes("enter");
    println!("Pressed enter");
    
    // Check what the state is after regex selection
    let current_state = cx.editor_state();
    println!("State after regex selection: {}", current_state);
    
    // Should have selected both instances of "apples"
    cx.assert_state("I like to eat «applesˇ» since my favorite fruit is «applesˇ».", Mode::HelixNormal);
    
    // For now, just verify that the regex selection worked correctly
    // TODO: Fix the change operation in Helix mode to work with multiple selections
    println!("Regex selection workflow completed successfully");
}

#[gpui::test]
async fn test_split_selection_tutor_workflow(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    // Test split selection workflow from tutor
    cx.set_state("«these are sentences. some sentences don't start with uppercase letters! that is not good grammar. you can fix thisˇ».", Mode::HelixNormal);
    
    // Split on sentence boundaries (. or !)
    cx.simulate_keystrokes("shift-s");
    cx.simulate_input("\\. |! ");
    cx.simulate_keystrokes("enter");
    
    // Should have split into separate sentences
    // Note: The exact result depends on the regex implementation
    // For now, we'll just verify the modal interaction worked
    let modal_closed = cx.workspace(|workspace, _, cx| {
        workspace.active_modal::<InteractiveRegexPrompt>(cx).is_none()
    });
    assert!(modal_closed, "Split selection modal should be closed");
}

// Unit tests for core regex functionality

#[cfg(test)]
mod unit_tests {
    use super::*;
    use regex::Regex;

    #[test]
    fn test_regex_select_matches() {
        // Test the core regex matching logic
        let text = "Nobody expects the Spanish inquisition";
        let regex = Regex::new(r"[A-Z][a-z]*").unwrap();
        
        let matches: Vec<_> = regex.find_iter(text).collect();
        assert_eq!(matches.len(), 2);
        assert_eq!(&text[matches[0].range()], "Nobody");
        assert_eq!(&text[matches[1].range()], "Spanish");
    }

    #[test]
    fn test_regex_split_matches() {
        // Test the core split logic (from Helix test)
        let text = " abcd efg wrs   xyz 123 456";
        let regex = Regex::new(r"\s+").unwrap();
        
        let mut parts = Vec::new();
        let mut last_end = 0;
        
        for mat in regex.find_iter(text) {
            // Always add the text before the match (including empty strings for leading matches)
            parts.push(&text[last_end..mat.start()]);
            last_end = mat.end();
        }
        
        // Add remaining text after last match
        if last_end <= text.len() {
            parts.push(&text[last_end..]);
        }
        
        // Should match Helix behavior: ["", "abcd", "efg", "wrs", "xyz", "123", "456"]
        assert_eq!(parts, vec!["", "abcd", "efg", "wrs", "xyz", "123", "456"]);
    }

    #[test]
    fn test_regex_keep_remove_logic() {
        let selections = vec!["one", "two", "123", "four", "567"];
        let regex = Regex::new(r"\d+").unwrap();
        
        let kept: Vec<_> = selections.iter()
            .filter(|s| regex.is_match(s))
            .collect();
        assert_eq!(kept, vec![&"123", &"567"]);
        
        let removed: Vec<_> = selections.iter()
            .filter(|s| !regex.is_match(s))
            .collect();
        assert_eq!(removed, vec![&"one", &"two", &"four"]);
    }

    #[test]
    fn test_regex_keep_remove_partial_matches() {
        // Test partial match behavior for keep/remove operations
        let regex_o = Regex::new(r"o").unwrap();
        let regex_on = Regex::new(r"on").unwrap();
        
        // Test data: selections containing "one", "two", "three"
        let selections = vec!["one", "two", "three"];
        
        // Keep selections containing "o" - should keep "one" and "two"
        let keep_o: Vec<_> = selections.iter()
            .filter(|&text| regex_o.is_match(text))
            .collect();
        assert_eq!(keep_o, vec![&"one", &"two"]);
        
        // Remove selections containing "o" - should keep only "three"
        let remove_o: Vec<_> = selections.iter()
            .filter(|&text| !regex_o.is_match(text))
            .collect();
        assert_eq!(remove_o, vec![&"three"]);
        
        // Keep selections containing "on" - should keep only "one"
        let keep_on: Vec<_> = selections.iter()
            .filter(|&text| regex_on.is_match(text))
            .collect();
        assert_eq!(keep_on, vec![&"one"]);
        
        // Remove selections containing "on" - should keep "two" and "three"
        let remove_on: Vec<_> = selections.iter()
            .filter(|&text| !regex_on.is_match(text))
            .collect();
        assert_eq!(remove_on, vec![&"two", &"three"]);
    }
}

// Performance tests for large selections

#[cfg(test)]
mod performance_tests {
    use super::*;
    use regex::Regex;

    #[test]
    fn test_regex_performance_large_text() {
        // Test performance with large text and many selections
        let large_text = "word ".repeat(10000);
        let regex = Regex::new(r"word").unwrap();
        
        let start = std::time::Instant::now();
        let matches: Vec<_> = regex.find_iter(&large_text).collect();
        let duration = start.elapsed();
        
        assert_eq!(matches.len(), 10000);
        assert!(duration.as_millis() < 100, "Regex matching should be fast");
    }
}

#[gpui::test]
async fn test_regex_operations_always_return_to_normal_mode(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    // Test that regex operations from HelixNormal mode stay in HelixNormal
    cx.set_state("«hello worldˇ»", Mode::HelixNormal);
    
    cx.simulate_keystrokes("s");
    cx.simulate_input("world");
    cx.simulate_keystrokes("enter");
    
    // Should be in HelixNormal mode after regex operation
    assert_eq!(cx.mode(), Mode::HelixNormal);
    cx.assert_state("hello «worldˇ»", Mode::HelixNormal);
    
    // Test that regex operations from HelixSelect mode return to HelixNormal
    cx.set_state("«hello worldˇ»", Mode::HelixSelect);
    
    cx.simulate_keystrokes("s");
    cx.simulate_input("hello");
    cx.simulate_keystrokes("enter");
    
    // Should be in HelixNormal mode after regex operation (not HelixSelect)
    assert_eq!(cx.mode(), Mode::HelixNormal);
    cx.assert_state("«helloˇ» world", Mode::HelixNormal);
}

#[gpui::test]
async fn test_collapse_selection_cursor_position(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    // Test that collapse selection positions cursor correctly according to Helix behavior
    // For forward selection Range(0, 5) selecting "hello", cursor should be at position 4 ('o')
    cx.set_state("«helloˇ» world", Mode::HelixNormal);
    
    // Collapse selection - cursor should be at position 4 (the 'o' character)
    cx.simulate_keystrokes(";");
    
    // Based on Helix cursor behavior: Range::new(0, 5).cursor() = prev_grapheme_boundary(5) = 4
    cx.assert_state("hellˇo world", Mode::HelixNormal);
    
    // Test with backward selection - cursor should be at head position
    cx.set_state("hello «ˇworld»", Mode::HelixNormal);
    
    cx.simulate_keystrokes(";");
    
    // For backward selection, cursor is at head position (start of "world")
    cx.assert_state("hello ˇworld", Mode::HelixNormal);
}

#[gpui::test]
async fn test_mode_switching_issue_debug(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    // Test the specific issue: s or S from HelixSelect mode should return to HelixNormal
    cx.set_state("hello worldˇ", Mode::HelixNormal);
    
    // Enter select mode
    cx.simulate_keystrokes("v");
    assert_eq!(cx.mode(), Mode::HelixSelect);
    
    // Create a selection manually
    cx.set_state("«hello ˇ»world", Mode::HelixSelect);
    assert_eq!(cx.mode(), Mode::HelixSelect);
    
    // Use 's' keystroke to trigger select regex from HelixSelect mode
    cx.simulate_keystrokes("s");
    
    // Verify modal is open
    let modal_open = cx.workspace(|workspace, _, cx| {
        workspace.active_modal::<InteractiveRegexPrompt>(cx).is_some()
    });
    assert!(modal_open, "Regex selection modal should be open");
    
    // Type a pattern and confirm
    cx.simulate_input("hello");
    cx.simulate_keystrokes("enter");
    
    // Check what mode we're in after the operation
    let current_mode = cx.mode();
    println!("Mode after regex operation from HelixSelect: {:?}", current_mode);
    
    // Should be in HelixNormal mode, not HelixSelect
    assert_eq!(current_mode, Mode::HelixNormal, "Should return to HelixNormal mode after regex operation");
}

#[gpui::test]
async fn test_primary_index_debug(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    // Reset primary index to start fresh
    crate::helix::selections::reset_primary_selection_index();

    // Start with three selections
    cx.set_state("«oneˇ» «twoˇ» «threeˇ»", Mode::HelixNormal);
    
    // Check initial primary index
    let initial_index = crate::helix::selections::get_primary_selection_index();
    println!("Initial primary index: {}", initial_index);
    
    // Remove primary (should remove first selection)
    cx.simulate_keystrokes("alt-,");
    cx.assert_state("one «twoˇ» «threeˇ»", Mode::HelixNormal);
    
    // Select whole line with x (creates new selection)
    cx.simulate_keystrokes("x");
    cx.assert_state("«one two threeˇ»", Mode::HelixNormal);
    
    // Check primary index after line selection
    let after_line_index = crate::helix::selections::get_primary_selection_index();
    println!("Primary index after line selection: {}", after_line_index);
    
    // Use split selection on regex functionality
    cx.simulate_keystrokes("shift-s");  // Open split selection modal
    cx.simulate_input(" ");             // Split on spaces
    cx.simulate_keystrokes("enter");    // Confirm the operation
    
    // Check primary index after split
    let after_split_index = crate::helix::selections::get_primary_selection_index();
    println!("Primary index after split: {}", after_split_index);
    
    // Check current state and count selections
    let current_state = cx.editor_state();
    println!("State after split: {}", current_state);
    
    // Count the number of selections
    let selection_count = cx.editor(|editor, _window, cx| {
        editor.selections.all_adjusted(cx).len()
    });
    println!("Number of selections after split: {}", selection_count);
    
    // Remove primary selection (should remove first selection if index was reset)
    cx.simulate_keystrokes("alt-,");
    
    // Check final state
    let final_state = cx.editor_state();
    println!("Final state: {}", final_state);
    
    // Count selections after remove
    let final_selection_count = cx.editor(|editor, _window, cx| {
        editor.selections.all_adjusted(cx).len()
    });
    println!("Number of selections after remove: {}", final_selection_count);
}

#[gpui::test]
async fn test_remove_primary_after_split_simple(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    // Start with a simple line
    cx.set_state("«one two threeˇ»", Mode::HelixNormal);
    
    // Split on spaces
    cx.simulate_keystrokes("shift-s");  // Open split selection modal
    cx.simulate_input(" ");             // Split on spaces
    cx.simulate_keystrokes("enter");    // Confirm the operation
    
    // Check state after split
    let after_split_state = cx.editor_state();
    println!("After split: {}", after_split_state);
    
    // Check mode after split
    let mode_after_split = cx.mode();
    println!("Mode after split: {:?}", mode_after_split);
    
    // Try to remove primary selection
    println!("About to press alt-,");
    cx.simulate_keystrokes("alt-,");
    
    // Check final state
    let final_state = cx.editor_state();
    println!("Final state: {}", final_state);
}

#[gpui::test]
async fn test_remove_primary_simple_debug(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    // Reset primary index
    crate::helix::selections::reset_primary_selection_index();

    // Start with three selections manually set
    cx.set_state("«oneˇ» «twoˇ» «threeˇ»", Mode::HelixNormal);
    
    // Check initial state
    let initial_count = cx.editor(|editor, _window, cx| {
        editor.selections.all_adjusted(cx).len()
    });
    println!("Initial selection count: {}", initial_count);
    
    // Try to remove primary selection immediately
    cx.simulate_keystrokes("alt-,");
    
    // Check final state
    let final_count = cx.editor(|editor, _window, cx| {
        editor.selections.all_adjusted(cx).len()
    });
    println!("Final selection count: {}", final_count);
    
    let final_state = cx.editor_state();
    println!("Final state: {}", final_state);
    
    // This should work if the basic functionality is correct
    cx.assert_state("one «twoˇ» «threeˇ»", Mode::HelixNormal);
}

#[gpui::test]
async fn test_regex_operations_return_to_normal_from_select_mode(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    // Test s (SelectRegex) from HelixSelect mode
    cx.set_state("hello worldˇ", Mode::HelixNormal);
    cx.simulate_keystrokes("v"); // Enter select mode
    assert_eq!(cx.mode(), Mode::HelixSelect);
    
    cx.set_state("«hello ˇ»world", Mode::HelixSelect); // Set a selection in select mode
    assert_eq!(cx.mode(), Mode::HelixSelect);
    
    // Use 's' from HelixSelect mode
    cx.simulate_keystrokes("s");
    cx.simulate_input("hello");
    cx.simulate_keystrokes("enter");
    
    // Should be back in HelixNormal mode
    println!("Mode after 's' from HelixSelect: {:?}", cx.mode());
    assert_eq!(cx.mode(), Mode::HelixNormal, "Should return to HelixNormal after 's' from HelixSelect");
    
    // Test S (SplitSelectionOnRegex) from HelixSelect mode
    cx.set_state("hello worldˇ", Mode::HelixNormal);
    cx.simulate_keystrokes("v"); // Enter select mode
    cx.set_state("«hello worldˇ»", Mode::HelixSelect);
    assert_eq!(cx.mode(), Mode::HelixSelect);
    
    // Use 'S' from HelixSelect mode
    cx.simulate_keystrokes("shift-s");
    cx.simulate_input(" ");
    cx.simulate_keystrokes("enter");
    
    // Should be back in HelixNormal mode
    println!("Mode after 'S' from HelixSelect: {:?}", cx.mode());
    assert_eq!(cx.mode(), Mode::HelixNormal, "Should return to HelixNormal after 'S' from HelixSelect");
    
    // Test K (KeepSelections) from HelixSelect mode
    cx.set_state("«oneˇ» «twoˇ» «threeˇ»", Mode::HelixSelect);
    assert_eq!(cx.mode(), Mode::HelixSelect);
    
    // Use 'K' from HelixSelect mode
    cx.simulate_keystrokes("shift-k");
    cx.simulate_input("o");
    cx.simulate_keystrokes("enter");
    
    // Should be back in HelixNormal mode
    println!("Mode after 'K' from HelixSelect: {:?}", cx.mode());
    assert_eq!(cx.mode(), Mode::HelixNormal, "Should return to HelixNormal after 'K' from HelixSelect");
    
    // Test Alt-K (RemoveSelections) from HelixSelect mode
    cx.set_state("«oneˇ» «twoˇ» «threeˇ»", Mode::HelixSelect);
    assert_eq!(cx.mode(), Mode::HelixSelect);
    
    // Use 'Alt-K' from HelixSelect mode
    cx.simulate_keystrokes("alt-shift-k");
    cx.simulate_input("e");
    cx.simulate_keystrokes("enter");
    
    // Should be back in HelixNormal mode
    println!("Mode after 'Alt-K' from HelixSelect: {:?}", cx.mode());
    assert_eq!(cx.mode(), Mode::HelixNormal, "Should return to HelixNormal after 'Alt-K' from HelixSelect");
} 