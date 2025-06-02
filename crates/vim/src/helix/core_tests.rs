//! Core movement function tests mirroring Helix test structure
//!
//! These tests directly validate the pure movement functions against Helix ground truth,
//! using the same test case format as helix-core/src/movement.rs

use super::core::*;
use rope::Rope;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_behaviour_when_moving_to_start_of_next_words() {
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
        ];

        for (sample, scenario) in tests {
            for (count, begin, expected_end) in scenario.into_iter() {
                let text = Rope::from(sample);
                let range = move_next_word_start(&text, begin, count);
                assert_eq!(range, expected_end, "Case failed: [{}]", sample);
            }
        }
    }

    #[test]
    fn test_behaviour_when_moving_to_start_of_next_long_words() {
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
            ("alphanumeric.!,and.?=punctuation are not treated any differently than alphanumerics",
                vec![(1, Range::new(0, 0), Range::new(0, 32))]),
            ("...   ... punctuation and spaces behave as expected",
                vec![
                    (1, Range::new(0, 0), Range::new(0, 6)),
                    (1, Range::new(0, 6), Range::new(6, 10)),
                ]),
            (".._.._ punctuation is joined by underscores into a single word",
                vec![(1, Range::new(0, 0), Range::new(0, 7))]),
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
        ];

        for (sample, scenario) in tests {
            for (count, begin, expected_end) in scenario.into_iter() {
                let text = Rope::from(sample);
                let range = move_next_long_word_start(&text, begin, count);
                assert_eq!(range, expected_end, "Case failed: [{}]", sample);
            }
        }
    }

    #[test]
    fn test_behaviour_when_moving_to_end_of_next_words() {
        let tests = [
            ("Basic forward motion from the start of a word to the end of it",
                vec![(1, Range::new(0, 0), Range::new(0, 5))]),
            ("Jump to end of a word from another word",
                vec![(1, Range::new(6, 6), Range::new(6, 13))]),
            ("Jump to end of a word when in the middle of it",
                vec![(1, Range::new(2, 2), Range::new(2, 5))]),
            ("Jump to end of a word from whitespace",
                vec![(1, Range::new(5, 5), Range::new(5, 13))]),
            ("Jump to end of a word from newline",
                vec![(1, Range::new(7, 7), Range::new(7, 15))]),
            ("alphanumeric.!,and.?=punctuation are considered 'words' for the purposes of word motion",
                vec![
                    (1, Range::new(0, 0), Range::new(0, 11)),
                    (1, Range::new(12, 12), Range::new(12, 14)),
                    (1, Range::new(15, 15), Range::new(15, 17))
                ]),
            ("...   ... punctuation and spaces behave as expected",
                vec![
                    (1, Range::new(0, 0), Range::new(0, 2)),
                    (1, Range::new(3, 3), Range::new(3, 5)),
                ]),
            (".._.._ punctuation is not joined by underscores into a single block",
                vec![(1, Range::new(0, 0), Range::new(0, 1))]),
            ("Newlines\n\nare bridged seamlessly.",
                vec![
                    (1, Range::new(0, 0), Range::new(0, 7)),
                    (1, Range::new(8, 8), Range::new(8, 10)),
                ]),
        ];

        for (sample, scenario) in tests {
            for (count, begin, expected_end) in scenario.into_iter() {
                let text = Rope::from(sample);
                let range = move_next_word_end(&text, begin, count);
                assert_eq!(range, expected_end, "Case failed: [{}]", sample);
            }
        }
    }

    #[test]
    fn test_behaviour_when_moving_to_start_of_previous_words() {
        let tests = [
            ("Basic backward motion from the middle of a word",
                vec![(1, Range::new(6, 6), Range::new(6, 0))]),
            ("Starting from whitespace moves to first space in sequence",
                vec![(1, Range::new(10, 10), Range::new(10, 6))]),
            ("Previous anchor is irrelevant for backward motions",
                vec![(1, Range::new(0, 12), Range::new(12, 6))]),
            ("Starting from mid-word leaves anchor at start position and moves head",
                vec![(1, Range::new(9, 9), Range::new(9, 3))]),
        ];

        for (sample, scenario) in tests {
            for (count, begin, expected_end) in scenario.into_iter() {
                let text = Rope::from(sample);
                let range = move_prev_word_start(&text, begin, count);
                assert_eq!(range, expected_end, "Case failed: [{}]", sample);
            }
        }
    }

    #[test]
    fn test_range_operations() {
        let range = Range::new(5, 10);
        assert_eq!(range.from(), 5);
        assert_eq!(range.to(), 10);
        assert_eq!(range.len(), 5);
        assert_eq!(range.direction(), Direction::Forward);
        assert!(!range.is_empty());
        
        let flipped = range.flip();
        assert_eq!(flipped.anchor, 10);
        assert_eq!(flipped.head, 5);
        assert_eq!(flipped.direction(), Direction::Backward);
        
        let point = Range::point(7);
        assert!(point.is_empty());
        assert_eq!(point.from(), 7);
        assert_eq!(point.to(), 7);
    }

    #[test]
    fn test_cursor_positioning() {
        let text = Rope::from("hello world");
        
        // Forward range: cursor should be before head
        let range = Range::new(0, 5);
        assert_eq!(range.cursor(&text), 4); // Before 'o'
        
        // Backward range: cursor should be at head
        let range = Range::new(5, 0);
        assert_eq!(range.cursor(&text), 0);
        
        // Point range: cursor should be at position
        let range = Range::point(3);
        assert_eq!(range.cursor(&text), 3);
    }

    #[test]
    fn test_put_cursor() {
        let text = Rope::from("hello world");
        let range = Range::new(0, 5);
        
        // Move cursor without extending
        let moved = range.put_cursor(&text, 8, false);
        assert_eq!(moved, Range::point(8));
        
        // Move cursor with extending
        let extended = range.put_cursor(&text, 8, true);
        assert_eq!(extended, Range::new(0, 8));
    }

    #[test]
    fn test_word_char_classification() {
        // Regular words (ignore_punctuation = false)
        assert!(super::is_word_char('a'));
        assert!(super::is_word_char('Z'));
        assert!(super::is_word_char('5'));
        assert!(super::is_word_char('_'));
        assert!(!super::is_word_char('.'));
        assert!(!super::is_word_char(' '));
        assert!(!super::is_word_char('\n'));

        // Note: Long word mode behavior would need different function
        assert!(super::is_word_char('a'));
        assert!(!super::is_word_char('.'));
        assert!(!super::is_word_char('!'));
        assert!(!super::is_word_char(' '));
        assert!(!super::is_word_char('\n'));
        assert!(!super::is_word_char('\t'));
    }

    #[test]
    fn test_boundary_conditions() {
        // Empty string
        let empty = Rope::from("");
        let range = Range::point(0);
        assert_eq!(move_next_word_start(&empty, range, 1), range);
        assert_eq!(move_prev_word_start(&empty, range, 1), range);
        
        // Single character
        let single = Rope::from("a");
        let range = Range::point(0);
        let result = move_next_word_start(&single, range, 1);
        // Should move to end since there's only one word
        assert_eq!(result.head, 1);
        
        // At end of text
        let text = Rope::from("hello");
        let range = Range::point(5);
        assert_eq!(move_next_word_start(&text, range, 1), range);
    }

    #[test]
    fn test_excessive_counts() {
        let text = Rope::from("one two three");
        let range = Range::point(0);
        
        // Move way more words than exist
        let result = move_next_word_start(&text, range, 999);
        // Should stop at the end
        assert!(result.head <= text.len());
    }

    #[test]
    fn test_multibyte_characters() {
        let text = Rope::from("ヒーリクス editor");
        let range = Range::point(0);
        
        // Should handle multibyte characters correctly
        let result = move_next_word_start(&text, range, 1);
        // Should select the full multibyte word plus space
        assert!(result.head > 6); // More than just ASCII length
    }
}