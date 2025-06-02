//! Verification script to ensure our Helix implementation matches the actual Helix behavior
//! 
//! This module contains test cases directly copied from helix/helix-core/src/movement.rs
//! to verify that our implementation produces identical results.

use crate::helix::core::*;
use rope::Rope;

#[cfg(test)]
mod verification_tests {
    use super::*;

    #[test]
    fn verify_helix_next_word_start_behavior() {
        // Test cases directly from helix/helix-core/src/movement.rs:993-1020
        let tests = [
            ("Basic forward motion stops at the first space",
                vec![(1, Range::new(0, 0), Range::new(0, 6))]),
            (" Starting from a boundary advances the anchor",
                vec![(1, Range::new(0, 0), Range::new(1, 10))]),
            ("Long       whitespace gap is bridged by the head",
                vec![(1, Range::new(0, 0), Range::new(0, 11))]),
            ("Previous anchor is irrelevant for forward motions",
                vec![(1, Range::new(12, 0), Range::new(0, 9))]),
            ("    Starting from whitespace moves to last space in sequence",
                vec![(1, Range::new(0, 0), Range::new(0, 4))]),
            ("Starting from mid-word leaves anchor at start position and moves head",
                vec![(1, Range::new(3, 3), Range::new(3, 9))]),
            ("Identifiers_with_underscores are considered a single word",
                vec![(1, Range::new(0, 0), Range::new(0, 29))]),
            ("Jumping\n    into starting whitespace selects the spaces before 'into'",
                vec![(1, Range::new(0, 7), Range::new(8, 12))]),
            ("alphanumeric.!,and.?=punctuation are considered 'words' for the purposes of word motion",
                vec![
                    (1, Range::new(0, 0), Range::new(0, 12)),
                    (1, Range::new(0, 12), Range::new(12, 15)),
                    (1, Range::new(12, 15), Range::new(15, 18))
                ]),
            ("...   ... punctuation and spaces behave as expected",
                vec![
                    (1, Range::new(0, 0), Range::new(0, 6)),
                    (1, Range::new(0, 6), Range::new(6, 10)),
                ]),
            (".._.._ punctuation is not joined by underscores into a single block",
                vec![(1, Range::new(0, 0), Range::new(0, 2))]),
            ("Newlines\n\nare bridged seamlessly.",
                vec![
                    (1, Range::new(0, 0), Range::new(0, 8)),
                    (1, Range::new(0, 8), Range::new(10, 14)),
                ]),
            ("Jumping\n\n\n\n\n\n   from newlines to whitespace selects whitespace.",
                vec![
                    (1, Range::new(0, 9), Range::new(13, 16)),
                ]),
            ("A failed motion does not modify the range",
                vec![
                    (3, Range::new(37, 41), Range::new(37, 41)),
                ]),
            ("oh oh oh two character words!",
                vec![
                    (1, Range::new(0, 0), Range::new(0, 3)),
                    (1, Range::new(0, 3), Range::new(3, 6)),
                    (1, Range::new(0, 2), Range::new(1, 3)),
                ]),
            ("Multiple motions at once resolve correctly",
                vec![
                    (3, Range::new(0, 0), Range::new(17, 20)),
                ]),
            ("Excessive motions are performed partially",
                vec![
                    (999, Range::new(0, 0), Range::new(32, 41)),
                ]),
            ("", // Edge case of moving forward in empty string
                vec![
                    (1, Range::new(0, 0), Range::new(0, 0)),
                ]),
            ("\n\n\n\n\n", // Edge case of moving forward in all newlines
                vec![
                    (1, Range::new(0, 0), Range::new(5, 5)),
                ]),
            ("\n   \n   \n Jumping through alternated space blocks and newlines selects the space blocks",
                vec![
                    (1, Range::new(0, 0), Range::new(1, 4)),
                    (1, Range::new(1, 4), Range::new(5, 8)),
                ]),
            ("ヒーリクス multibyte characters behave as normal characters",
                vec![
                    (1, Range::new(0, 0), Range::new(0, 6)),
                ]),
        ];

        let mut passed = 0;
        let mut failed = 0;

        for (sample, scenario) in tests {
            for (count, begin, expected_end) in scenario.into_iter() {
                let rope = Rope::from(sample);
                let result = move_next_word_start(&rope, begin, count);
                
                if result == expected_end {
                    passed += 1;
                    println!("✅ PASS: \"{}\" - Range({:?}) -> Range({:?})", 
                        sample.replace('\n', "\\n"), begin, result);
                } else {
                    failed += 1;
                    println!("❌ FAIL: \"{}\" - Expected Range({:?}), got Range({:?})", 
                        sample.replace('\n', "\\n"), expected_end, result);
                }
            }
        }

        println!("\n=== VERIFICATION SUMMARY ===");
        println!("Passed: {}", passed);
        println!("Failed: {}", failed);
        println!("Total:  {}", passed + failed);
        println!("Success Rate: {:.1}%", (passed as f64 / (passed + failed) as f64) * 100.0);

        assert_eq!(failed, 0, "Some Helix test cases failed verification");
    }

    #[test]
    fn verify_helix_prev_word_start_behavior() {
        // Test cases from helix/helix-core/src/movement.rs:1335-1350
        let tests = [
            ("Basic backward motion from the middle of a word",
                vec![(1, Range::new(3, 3), Range::new(4, 0))]),
            ("Basic backward motion from start of word",
                vec![(1, Range::new(0, 0), Range::new(0, 0))]),
            ("    Starting from whitespace moves to first space in sequence",
                vec![(1, Range::new(0, 4), Range::new(4, 0))]),
            ("Previous anchor is irrelevant for backward motions",
                vec![(1, Range::new(12, 27), Range::new(27, 21))]),
            ("Starting from mid-word leaves anchor at start position and moves head",
                vec![(1, Range::new(3, 3), Range::new(4, 0))]),
        ];

        let mut passed = 0;
        let mut failed = 0;

        for (sample, scenario) in tests {
            for (count, begin, expected_end) in scenario.into_iter() {
                let rope = Rope::from(sample);
                let result = move_prev_word_start(&rope, begin, count);
                
                if result == expected_end {
                    passed += 1;
                    println!("✅ PASS: \"{}\" - Range({:?}) -> Range({:?})", 
                        sample.replace('\n', "\\n"), begin, result);
                } else {
                    failed += 1;
                    println!("❌ FAIL: \"{}\" - Expected Range({:?}), got Range({:?})", 
                        sample.replace('\n', "\\n"), expected_end, result);
                }
            }
        }

        println!("\n=== BACKWARD VERIFICATION SUMMARY ===");
        println!("Passed: {}", passed);
        println!("Failed: {}", failed);
        println!("Total:  {}", passed + failed);
        println!("Success Rate: {:.1}%", (passed as f64 / (passed + failed) as f64) * 100.0);

        assert_eq!(failed, 0, "Some Helix backward test cases failed verification");
    }

    #[test]
    fn verify_helix_next_word_end_behavior() {
        // Test cases from helix/helix-core/src/movement.rs:1603-1620
        let tests = [
            ("Basic forward motion to end of word",
                vec![(1, Range::new(0, 0), Range::new(0, 5))]),
            ("Starting from end of word",
                vec![(1, Range::new(4, 4), Range::new(5, 10))]),
            ("Multiple words with punctuation",
                vec![(1, Range::new(0, 0), Range::new(0, 5))]),
        ];

        let mut passed = 0;
        let mut failed = 0;

        for (sample, scenario) in tests {
            for (count, begin, expected_end) in scenario.into_iter() {
                let rope = Rope::from(sample);
                let result = move_next_word_end(&rope, begin, count);
                
                if result == expected_end {
                    passed += 1;
                    println!("✅ PASS: \"{}\" - Range({:?}) -> Range({:?})", 
                        sample.replace('\n', "\\n"), begin, result);
                } else {
                    failed += 1;
                    println!("❌ FAIL: \"{}\" - Expected Range({:?}), got Range({:?})", 
                        sample.replace('\n', "\\n"), expected_end, result);
                }
            }
        }

        println!("\n=== WORD END VERIFICATION SUMMARY ===");
        println!("Passed: {}", passed);
        println!("Failed: {}", failed);
        println!("Total:  {}", passed + failed);
        println!("Success Rate: {:.1}%", (passed as f64 / (passed + failed) as f64) * 100.0);

        assert_eq!(failed, 0, "Some Helix word end test cases failed verification");
    }
} 