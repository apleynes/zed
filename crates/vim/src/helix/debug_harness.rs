//! Debug harness to compare Helix vs Zed rope behavior
//! 
//! This module helps identify differences between Helix's ropey::Rope
//! and Zed's rope::Rope that might affect word movement behavior.

// use crate::helix::core::*;
// use rope::Rope as ZedRope;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_rope_differences() {
        let test_cases = [
            "Basic text",
            "Text with spaces",
            "Text\nwith\nnewlines",
            "Jumping\n    \nback through a newline selects whitespace",
            "alphanumeric.!,and.?=punctuation are considered 'words' for the purposes of word motion",
        ];

        for text in test_cases {
            println!("\n=== Testing text: '{}' ===", text);
            let zed_rope = ZedRope::from(text);
            
            // Compare character iteration
            println!("Character enumeration:");
            for (i, ch) in zed_rope.chars().enumerate() {
                if i > 50 { break; }
                println!("  {}: {:?}", i, ch);
            }
            
            // Test grapheme boundaries
            println!("Grapheme boundary tests:");
            for i in 0..=text.len().min(20) {
                if i <= zed_rope.len() {
                    let prev = prev_grapheme_boundary(&zed_rope, i);
                    let next = next_grapheme_boundary(&zed_rope, i);
                    println!("  pos {}: prev={}, next={}", i, prev, next);
                }
            }
        }
    }

    #[test]
    fn debug_failing_case_detailed() {
        let text = "alphanumeric.!,and.?=punctuation are considered 'words' for the purposes of word motion";
        let rope = ZedRope::from(text);
        
        println!("=== Detailed analysis of failing case ===");
        println!("Text: '{}'", text);
        
        // Show character mapping around position 21
        for i in 15..25 {
            if i < rope.len() {
                let ch = rope.chars().nth(i).unwrap_or('\0');
                println!("Index {}: '{}'", i, ch);
            }
        }
        
        // Test the specific failing range
        let input_range = Range::new(30, 21);
        println!("\nInput range: {:?}", input_range);
        
        // Test prev_grapheme_boundary at position 21
        let pgb21 = prev_grapheme_boundary(&rope, 21);
        println!("prev_grapheme_boundary(21) = {}", pgb21);
        
        // Check what character is at each position
        let char_20 = rope.chars().nth(20).unwrap_or('\0');
        let char_21 = rope.chars().nth(21).unwrap_or('\0');
        let char_22 = rope.chars().nth(22).unwrap_or('\0');
        println!("char_20='{}', char_21='{}', char_22='{}'", char_20, char_21, char_22);
        
        // Test word character classification
        println!("is_word_char(char_20)={}", is_word_char(char_20));
        println!("is_word_char(char_21)={}", is_word_char(char_21));
        println!("is_word_char(char_22)={}", is_word_char(char_22));
        
        // Run the actual movement
        let result = move_prev_word_start(&rope, input_range, 1);
        println!("move_prev_word_start result: {:?}", result);
        println!("Expected: Range {{ anchor: 21, head: 18 }}");
    }

    #[test]
    fn compare_unicode_segmentation() {
        use unicode_segmentation::UnicodeSegmentation;
        
        let test_texts = [
            "simple",
            "with.punctuation",
            "text\nwith\nnewlines",
        ];
        
        for text in test_texts {
            println!("\n=== Unicode segmentation for: '{}' ===", text);
            
            // Show grapheme boundaries using unicode-segmentation directly
            let boundaries: Vec<usize> = text.grapheme_indices(true).map(|(i, _)| i).collect();
            println!("Grapheme boundaries: {:?}", boundaries);
            
            // Compare with our implementation
            let rope = ZedRope::from(text);
            for i in 0..=text.len() {
                if i <= rope.len() {
                    let our_prev = prev_grapheme_boundary(&rope, i);
                    
                    // Find expected prev boundary using unicode-segmentation
                    let expected_prev = boundaries.iter()
                        .rev()
                        .find(|&&pos| pos < i)
                        .copied()
                        .unwrap_or(0);
                    
                    if our_prev != expected_prev {
                        println!("MISMATCH at pos {}: our={}, expected={}", i, our_prev, expected_prev);
                    }
                }
            }
        }
    }

    #[test] 
    fn debug_range_preparation_logic() {
        let text = "alphanumeric.!,and.?=punctuation are considered 'words' for the purposes of word motion";
        let rope = ZedRope::from(text);
        
        println!("=== Range preparation debugging ===");
        
        // Test case: Range { anchor: 30, head: 21 }
        let range = Range::new(30, 21);
        println!("Input range: {:?}", range);
        println!("anchor < head: {}", range.anchor < range.head);
        
        // Show what happens in each branch of range preparation
        let _target = WordMotionTarget::PrevWordStart;
        let _is_prev = true;
        
        if range.anchor < range.head {
            println!("Taking 'anchor < head' branch");
            let pgb = prev_grapheme_boundary(&rope, range.head);
            println!("prev_grapheme_boundary({}) = {}", range.head, pgb);
            
            let all_chars: Vec<char> = rope.chars().collect();
            let current_char = all_chars.get(range.head).copied().unwrap_or('\0');
            let prev_char = if range.head > 0 { all_chars.get(range.head - 1).copied() } else { None };
            
            println!("current_char='{}', prev_char={:?}", current_char, prev_char);
            println!("is_word_char(current)={}", is_word_char(current_char));
            if let Some(pc) = prev_char {
                println!("char_is_line_ending(prev)={}", char_is_line_ending(pc));
            }
            
            let should_use_newline_case = is_word_char(current_char) && prev_char.map(char_is_line_ending).unwrap_or(false);
            println!("Should use newline case: {}", should_use_newline_case);
            
            if should_use_newline_case {
                println!("Would use Range::new({}, {})", pgb, pgb);
            } else {
                println!("Would use Range::new({}, {})", range.head, pgb);
            }
        } else {
            println!("Would take other branch");
        }
    }

    #[test]
    fn test_debug_word_movement_integration() {
        debug_word_movement_integration();
    }

    #[test]
    fn test_zed_selection_semantics() {
        println!("=== TESTING ZED SELECTION SEMANTICS ===");
        
        // Test what happens when we create a selection from position 6 to 9
        // in the text "The quick brown fox"
        
        // This should help us understand if the issue is in our coordinate conversion
        // or in how we understand Zed's selection model
        
        println!("Text: 'The quick brown fox'");
        println!("Positions:");
        let text = "The quick brown fox";
        for (i, ch) in text.chars().enumerate() {
            if i >= 5 && i <= 11 {
                println!("  {}: '{}'", i, ch);
            }
        }
        
        println!("\nIf we create a selection from position 6 to 9:");
        println!("  tail=6 ('i'), head=9 (' ')");
        println!("  This should select: '{}'", text.chars().skip(6).take(4).collect::<String>());
        println!("  Expected in test notation: «ick ˇ»");
        
        println!("\nIf we create a selection from position 6 to 8:");
        println!("  tail=6 ('i'), head=8 ('k')");
        println!("  This should select: '{}'", text.chars().skip(6).take(3).collect::<String>());
        println!("  In test notation: «ickˇ»");
        
        println!("\nCurrent test failure shows:");
        println!("  Expected: «ick ˇ» (selection includes space, cursor at space)");
        println!("  Actual:   «ickˇ» (selection excludes space, cursor at 'k')");
        
        println!("\nSo the issue is that we're setting head=8 instead of head=9");
    }

    #[gpui::test]
    async fn test_understand_zed_selection_ranges(cx: &mut gpui::TestAppContext) {
        println!("=== UNDERSTANDING ZED SELECTION RANGES ===");
        
        use crate::test::VimTestContext;
        use crate::Mode;
        
        let mut cx = VimTestContext::new(cx, true).await;
        
        // Set up text with cursor at position 6
        cx.set_state("The quick brown fox", Mode::Normal);
        
        // Test what the current state looks like
        let initial_state = cx.editor_state();
        println!("Initial state: {}", initial_state);
        
        // Now manually create different selections to understand the model
        
        // Test 1: Selection from 6 to 10 (should include "ick ")
        cx.update_editor(|editor, window, cx| {
            use editor::scroll::Autoscroll;
            
            editor.change_selections(Some(Autoscroll::fit()), window, cx, |s| {
                s.select_ranges([6..10]);
            });
        });
        
        let state = cx.editor_state();
        println!("Selection 6..10: {}", state);
        
        // Test 2: Selection from 6 to 9 (should include "ick")
        cx.update_editor(|editor, window, cx| {
            use editor::scroll::Autoscroll;
            
            editor.change_selections(Some(Autoscroll::fit()), window, cx, |s| {
                s.select_ranges([6..9]);
            });
        });
        
        let state = cx.editor_state();
        println!("Selection 6..9: {}", state);
        
        // Test 3: Selection from 6 to 8 (should include "ic")
        cx.update_editor(|editor, window, cx| {
            use editor::scroll::Autoscroll;
            
            editor.change_selections(Some(Autoscroll::fit()), window, cx, |s| {
                s.select_ranges([6..8]);
            });
        });
        
        let state = cx.editor_state();
        println!("Selection 6..8: {}", state);
        
        // Now let's see what the test expects
        println!("\nTest expectation analysis:");
        println!("Text: 'The quick brown fox'");
        println!("Positions: 0123456789...");
        println!("Expected: The qu«ick ˇ»brown fox");
        println!("This means selection from 6 to 9 with cursor at 9");
        println!("But cursor should appear at the space, not after it");
    }
}

pub fn debug_word_movement_integration() {
    println!("=== DEBUGGING WORD MOVEMENT INTEGRATION ===");
    
    // Test the exact scenario from the failing test
    let text = "The quick brown fox";
    let rope = rope::Rope::from(text);
    
    println!("Text: '{}'", text);
    
    // Debug character positions
    for (i, ch) in text.chars().enumerate() {
        println!("Position {}: '{}'", i, ch);
    }
    
    // Test starting from position 6 (after "qu" in "quick")
    let start_pos = 6;
    println!("\nStarting from position {}: '{}'", start_pos, text.chars().nth(start_pos).unwrap_or('?'));
    
    // Create Helix range
    let helix_range = super::core::Range::new(start_pos, start_pos);
    println!("Input Helix range: {:?}", helix_range);
    
    // Apply word movement
    let result_range = super::core::move_next_word_start(&rope, helix_range, 1);
    println!("Result Helix range: {:?}", result_range);
    
    // Show what text is selected
    let selected_text = text.chars().skip(result_range.anchor).take(result_range.head - result_range.anchor).collect::<String>();
    println!("Selected text: '{}'", selected_text);
    
    // Expected: should select from position 6 to position 10 (space after "quick")
    // "ick " should be selected
    println!("Expected: 'ick ' (positions 6-10)");
    
    // Test what the test expects
    // The test shows: The qu«ick ˇ»brown fox
    // This means selection from position 6 to position 10 (inclusive of space, exclusive of 'b')
    println!("Test expects selection from 6 to 10, which is: '{}'", 
             text.chars().skip(6).take(4).collect::<String>());
    
    // Now debug what happens in the coordinate conversion
    println!("\n=== COORDINATE CONVERSION DEBUG ===");
    
    // Simulate what happens in the integration layer
    // This is what the movement.rs code does:
    
    // 1. Convert back to Zed coordinate system
    println!("Helix result: anchor={}, head={}", result_range.anchor, result_range.head);
    
    // 2. Test the cursor positioning logic
    let cursor_pos = result_range.cursor(&rope);
    println!("Cursor position from Helix: {} ('{}')", cursor_pos, text.chars().nth(cursor_pos).unwrap_or('?'));
    
    // 3. In Zed, we convert offset to point, then point to DisplayPoint
    // Let's simulate this with a simple text buffer
    
    // The issue might be that Helix ranges are inclusive but Zed selections are different
    // Let's check what character is at each position
    println!("Character at anchor ({}): '{}'", result_range.anchor, text.chars().nth(result_range.anchor).unwrap_or('?'));
    println!("Character at head ({}): '{}'", result_range.head, text.chars().nth(result_range.head).unwrap_or('?'));
    
    // Test the cursor positioning logic step by step
    println!("\n=== CURSOR POSITIONING LOGIC ===");
    println!("Range direction: {:?}", result_range.direction());
    println!("Is forward selection: {}", result_range.head > result_range.anchor);
    
    if result_range.head > result_range.anchor {
        let prev_boundary = super::core::prev_grapheme_boundary(&rope, result_range.head);
        println!("prev_grapheme_boundary({}) = {} ('{}')", result_range.head, prev_boundary, text.chars().nth(prev_boundary).unwrap_or('?'));
    }
    
    // Expected test result analysis
    println!("\n=== EXPECTED TEST RESULT ANALYSIS ===");
    println!("Test expects: The qu«ick ˇ»brown fox");
    println!("This means:");
    println!("- Selection starts at position 6 ('i')");
    println!("- Selection ends at position 9 (' ')");  
    println!("- Cursor appears at position 9 (' ')");
    println!("- But the selection should include the space");
    
    // So the issue might be that we need the selection to go from 6 to 10 (including space)
    // But the cursor should appear at position 9 (the space itself)
    
    // Let me test what happens if we create a range from 6 to 10
    let test_range = super::core::Range::new(6, 10);
    let test_cursor = test_range.cursor(&rope);
    println!("Test range 6->10 cursor: {} ('{}')", test_cursor, text.chars().nth(test_cursor).unwrap_or('?'));
    
    // What about 6 to 9?
    let test_range2 = super::core::Range::new(6, 9);
    let test_cursor2 = test_range2.cursor(&rope);
    println!("Test range 6->9 cursor: {} ('{}')", test_cursor2, text.chars().nth(test_cursor2).unwrap_or('?'));
}