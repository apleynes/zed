//! Debug harness to compare Helix vs Zed rope behavior
//! 
//! This module helps identify differences between Helix's ropey::Rope
//! and Zed's rope::Rope that might affect word movement behavior.

use crate::helix::core::*;
use rope::Rope as ZedRope;

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
        let target = WordMotionTarget::PrevWordStart;
        let is_prev = true;
        
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
}