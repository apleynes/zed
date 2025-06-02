use crate::helix::core::*;
use rope::Rope;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_boundary_behavior() {
        // Test case: " Starting from" with cursor at position 0 (the space)
        let text = " Starting from";
        let rope = Rope::from(text);
        
        println!("=== Boundary behavior debugging ===");
        println!("Text: {:?}", text);
        println!("Text chars: {:?}", text.chars().collect::<Vec<_>>());
        
        // Initial cursor position 0 (the space)
        let initial_range = Range::new(0, 0);
        println!("Initial range: {:?}", initial_range);
        
        // Move to next word start
        let result = move_next_word_start(&rope, initial_range, 1);
        println!("Result range: {:?}", result);
        
        // Expected: Range { anchor: 0, head: 9 } to select " Starting"
        // But we might be getting Range { anchor: 0, head: 1 } to select just " "
        
        // Let's trace the character boundaries
        for i in 0..=10 {
            if i < text.len() {
                let ch = text.chars().nth(i).unwrap_or('\0');
                println!("Position {}: '{}' (is_word_char: {})", i, ch, is_word_char(ch));
            }
        }
        
        // Test the word boundary logic
        println!("\n=== Word boundary analysis ===");
        for i in 0..text.len()-1 {
            let ch1 = text.chars().nth(i).unwrap();
            let ch2 = text.chars().nth(i+1).unwrap();
            println!("Between {} and {}: '{}' -> '{}', is_word_boundary: {}", 
                     i, i+1, ch1, ch2, is_word_boundary(ch1, ch2));
        }
        
        // Debug the range_to_target function
        println!("\n=== range_to_target debugging ===");
        let target_range = range_to_target(&rope, initial_range, WordMotionTarget::NextWordStart);
        println!("range_to_target result: {:?}", target_range);
        
        // Test what happens if we start from position 1 instead
        let range_from_1 = Range::new(1, 1);
        let result_from_1 = move_next_word_start(&rope, range_from_1, 1);
        println!("From position 1: {:?} -> {:?}", range_from_1, result_from_1);
    }
    
    #[test]
    fn debug_expected_vs_actual() {
        let text = " Starting from";
        let rope = Rope::from(text);
        
        println!("=== Expected vs Actual ===");
        
        // What we expect: Range { anchor: 0, head: 9 }
        // This should select " Starting" (positions 0-8, head at 9)
        let expected = Range::new(0, 9);
        println!("Expected range: {:?}", expected);
        
        let start_slice = &text[expected.from()..expected.to()];
        println!("Expected selection: {:?}", start_slice);
        
        // What we're getting
        let actual = move_next_word_start(&rope, Range::new(0, 0), 1);
        println!("Actual range: {:?}", actual);
        
        if actual.to() <= text.len() {
            let actual_slice = &text[actual.from()..actual.to()];
            println!("Actual selection: {:?}", actual_slice);
        }
        
        // The test expects the head to advance to position 9 (after "Starting")
        // but we might be stopping at position 1 (after the space)
    }
    
    #[test]
    fn debug_boundary_specific_case() {
        let text = " Starting from";
        let rope = Rope::from(text);
        
        println!("=== Debugging Specific Boundary Case ===");
        println!("Text: {:?}", text);
        
        // Test case exactly as in the failing test
        let initial_range = Range::new(0, 0);
        println!("Initial range: {:?}", initial_range);
        
        // Step 1: What does range_to_target return?
        let range_target_result = range_to_target(&rope, initial_range, WordMotionTarget::NextWordStart);
        println!("range_to_target result: {:?}", range_target_result);
        
        // Step 2: What does the full move_next_word_start return?
        let final_result = move_next_word_start(&rope, initial_range, 1);
        println!("move_next_word_start result: {:?}", final_result);
        
        // Step 3: Let's see what the logic should be
        // When starting from whitespace (position 0, space), we should:
        // - Anchor should stay at 0 (the whitespace)
        // - Head should move to the end of the next word (position 9, after "Starting")
        
        println!("\n=== Analysis ===");
        println!("Position 0: '{}'", text.chars().nth(0).unwrap());
        println!("Position 9: '{}'", text.chars().nth(9).unwrap());
        println!("Expected selection would be: {:?}", &text[0..9]);
        
        // The issue seems to be that range_to_target is only moving to position 1
        // instead of finding the actual word boundary at position 9
    }
    
    #[test]
    fn debug_test_expectation() {
        let text = " Starting from";
        
        println!("=== Understanding Test Expectation ===");
        println!("Text: {:?}", text);
        
        // The test failure shows:
        // Expected: "« ˇ»Starting from" 
        // Actual: "« Startingˇ» from"
        
        // Let's break this down:
        // "« ˇ»Starting from" means selection from « to », cursor at ˇ
        // So the selection is just the space character, cursor at position 1
        
        // "« Startingˇ» from" means selection includes "Starting", cursor after "Starting"
        
        println!("Expected range interpretation:");
        println!("  Selection: just the space character");
        println!("  Range: anchor=0, head=1");
        println!("  Selected text: {:?}", &text[0..1]);
        
        println!("Actual range interpretation:");
        println!("  Selection: 'Starting' (positions 1-9)");  
        println!("  Range: anchor=1, head=9");
        println!("  Selected text: {:?}", &text[1..9]);
        
        // So the expected behavior is Range { anchor: 0, head: 1 }
        // But we're getting something that selects the word, not the space
        
        println!("\nThe first call to range_to_target returns Range {{ anchor: 0, head: 1 }}");
        println!("This is exactly what we want! The problem is the iteration continues.");
    }
}