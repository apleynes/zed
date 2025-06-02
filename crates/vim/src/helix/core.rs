//! Core Helix movement and selection functions
//! 
//! This module implements pure functions for Helix-style text navigation and selection,
//! directly mirroring the structure and behavior of helix-core/src/movement.rs.

use rope::Rope;

/// Direction for movements
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Forward,
    Backward,
}

/// Movement behavior for extending vs moving selections
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Movement {
    Move,
    Extend,
}

/// Word motion targets matching Helix's WordMotionTarget enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WordMotionTarget {
    NextWordStart,
    NextWordEnd,
    PrevWordStart,
    PrevWordEnd,
    NextLongWordStart,
    NextLongWordEnd,
    PrevLongWordStart, 
    PrevLongWordEnd,
    NextSubWordStart,
    NextSubWordEnd,
    PrevSubWordStart,
    PrevSubWordEnd,
}

/// Character categories for word boundary detection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharCategory {
    Whitespace,
    Eol,
    Word,
    Punctuation,
    Unknown,
}

/// A text range with anchor and head positions, mirroring Helix's Range
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Range {
    /// The anchor of the range: the side that doesn't move when extending
    pub anchor: usize,
    /// The head of the range, moved when extending
    pub head: usize,
}

impl Range {
    /// Create a new range with anchor and head positions
    pub fn new(anchor: usize, head: usize) -> Self {
        Self { anchor, head }
    }

    /// Create a point range (cursor position)
    pub fn point(head: usize) -> Self {
        Self::new(head, head)
    }

    /// Start of the range (minimum position)
    pub fn from(&self) -> usize {
        std::cmp::min(self.anchor, self.head)
    }

    /// End of the range (maximum position)
    pub fn to(&self) -> usize {
        std::cmp::max(self.anchor, self.head)
    }

    /// Length of the range
    pub fn len(&self) -> usize {
        self.to() - self.from()
    }

    /// Check if range is empty (anchor == head)
    pub fn is_empty(&self) -> bool {
        self.anchor == self.head
    }

    /// Get the direction of the range
    pub fn direction(&self) -> Direction {
        if self.head >= self.anchor {
            Direction::Forward
        } else {
            Direction::Backward
        }
    }

    /// Flip the range direction (swap anchor and head)
    pub fn flip(&self) -> Self {
        Self::new(self.head, self.anchor)
    }

    /// Get cursor position (with block cursor semantics like Helix)
    pub fn cursor(&self, text: &Rope) -> usize {
        // Helix cursor positioning: for forward ranges, cursor is at head - 1 (before the head)
        // For backward ranges and point ranges, cursor is at head
        if self.anchor < self.head {
            // Forward range: cursor is before the head (like Helix's block cursor)
            prev_grapheme_boundary(text, self.head)
        } else {
            // Backward range or point range: cursor is at head
            self.head
        }
    }

    /// Put cursor at position, optionally extending the selection
    pub fn put_cursor(&self, _text: &Rope, pos: usize, extend: bool) -> Self {
        if extend {
            Self::new(self.anchor, pos)
        } else {
            Self::point(pos)
        }
    }
}

impl From<(usize, usize)> for Range {
    fn from((anchor, head): (usize, usize)) -> Self {
        Self::new(anchor, head)
    }
}

// Simplified grapheme boundary implementation compatible with Zed's Rope

pub fn nth_prev_grapheme_boundary(text: &Rope, char_idx: usize, n: usize) -> usize {
    // Bounds check
    if char_idx == 0 || n == 0 {
        return char_idx;
    }
    
    // For Rope, we work with character indices, not byte indices
    // Each character is one position regardless of its byte length
    let char_count = text.chars().count();
    
    if char_idx > char_count {
        return char_count;
    }
    
    // Simple character-based boundary calculation
    // Each character is its own grapheme boundary for our purposes
    char_idx.saturating_sub(n)
}

pub fn prev_grapheme_boundary(text: &Rope, char_idx: usize) -> usize {
    nth_prev_grapheme_boundary(text, char_idx, 1)
}

pub fn nth_next_grapheme_boundary(text: &Rope, char_idx: usize, n: usize) -> usize {
    // Bounds check
    if n == 0 {
        return char_idx;
    }
    
    // For Rope, we work with character indices, not byte indices
    let char_count = text.chars().count();
    
    if char_idx >= char_count {
        return char_count;
    }
    
    // Simple character-based boundary calculation
    // Each character is its own grapheme boundary for our purposes
    (char_idx + n).min(char_count)
}

pub fn next_grapheme_boundary(text: &Rope, char_idx: usize) -> usize {
    nth_next_grapheme_boundary(text, char_idx, 1)
}

// Word movement functions matching Helix API

/// Move to start of next word, creating a selection that spans the word
pub fn move_next_word_start(text: &Rope, range: Range, count: usize) -> Range {
    word_move(text, range, count, WordMotionTarget::NextWordStart)
}

/// Move to end of next word, creating a selection that spans to word end
pub fn move_next_word_end(text: &Rope, range: Range, count: usize) -> Range {
    word_move(text, range, count, WordMotionTarget::NextWordEnd)
}

/// Move to start of previous word
pub fn move_prev_word_start(text: &Rope, range: Range, count: usize) -> Range {
    word_move(text, range, count, WordMotionTarget::PrevWordStart)
}

/// Move to end of previous word
pub fn move_prev_word_end(text: &Rope, range: Range, count: usize) -> Range {
    word_move(text, range, count, WordMotionTarget::PrevWordEnd)
}

/// Move to start of next long word (ignore punctuation)
pub fn move_next_long_word_start(text: &Rope, range: Range, count: usize) -> Range {
    word_move(text, range, count, WordMotionTarget::NextLongWordStart)
}

/// Move to end of next long word (ignore punctuation)
pub fn move_next_long_word_end(text: &Rope, range: Range, count: usize) -> Range {
    word_move(text, range, count, WordMotionTarget::NextLongWordEnd)
}

/// Move to start of previous long word (ignore punctuation)
pub fn move_prev_long_word_start(text: &Rope, range: Range, count: usize) -> Range {
    word_move(text, range, count, WordMotionTarget::PrevLongWordStart)
}

/// Move to end of previous long word (ignore punctuation)
pub fn move_prev_long_word_end(text: &Rope, range: Range, count: usize) -> Range {
    word_move(text, range, count, WordMotionTarget::PrevLongWordEnd)
}

/// Word movement function that mirrors Helix's implementation exactly
fn word_move(text: &Rope, range: Range, count: usize, target: WordMotionTarget) -> Range {
    let is_prev = matches!(
        target,
        WordMotionTarget::PrevWordStart
            | WordMotionTarget::PrevLongWordStart
            | WordMotionTarget::PrevSubWordStart
            | WordMotionTarget::PrevWordEnd
            | WordMotionTarget::PrevLongWordEnd
            | WordMotionTarget::PrevSubWordEnd
    );

    // Special-case early-out
    if (is_prev && range.head == 0) || (!is_prev && range.head == text.len()) {
        return range;
    }

    // Prepare the range appropriately based on the target movement direction.
    // This is addressing two things at once:
    //   1. Block-cursor semantics.
    //   2. The anchor position being irrelevant to the output result.
    #[allow(clippy::collapsible_else_if)] // Makes the structure clearer in this case.
    let start_range = if is_prev {
        if range.anchor < range.head {
            Range::new(range.head, prev_grapheme_boundary(text, range.head))
        } else {
            Range::new(next_grapheme_boundary(text, range.head), range.head)
        }
    } else {
        if range.anchor < range.head {
            Range::new(prev_grapheme_boundary(text, range.head), range.head)
        } else {
            Range::new(range.head, next_grapheme_boundary(text, range.head))
        }
    };

    // Do the main work
    let mut range = start_range;
    
    for _ in 0..count {
        let next_range = range_to_target(text, range, target);
        if range == next_range {
            break;
        }
        range = next_range;
    }
    
    range
}

/// Core range_to_target implementation adapted from Helix
pub fn range_to_target(text: &Rope, origin: Range, target: WordMotionTarget) -> Range {
    let is_prev = matches!(
        target,
        WordMotionTarget::PrevWordStart
            | WordMotionTarget::PrevLongWordStart
            | WordMotionTarget::PrevSubWordStart
            | WordMotionTarget::PrevWordEnd
            | WordMotionTarget::PrevLongWordEnd
            | WordMotionTarget::PrevSubWordEnd
    );

    let mut anchor = origin.anchor;
    let mut head = origin.head;
    
    // Get all characters for safe multibyte character handling
    let all_chars: Vec<char> = text.chars().collect();
    
    // Function to advance index in the appropriate motion direction.
    let advance: &dyn Fn(&mut usize) = if is_prev {
        &|idx| *idx = idx.saturating_sub(1)
    } else {
        &|idx| *idx += 1
    };

    // Get previous character for boundary detection
    let mut prev_ch = if head > 0 {
        all_chars.get(head.saturating_sub(1)).copied()
    } else {
        None
    };

    // Skip any initial newline characters.
    if is_prev {
        while head > 0 {
            if let Some(ch) = all_chars.get(head - 1) {
                if char_is_line_ending(*ch) {
                    prev_ch = Some(*ch);
                    advance(&mut head);
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    } else {
        while head < all_chars.len() {
            if let Some(ch) = all_chars.get(head) {
                if char_is_line_ending(*ch) {
                    prev_ch = Some(*ch);
                    advance(&mut head);
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }
    
    if prev_ch.map(char_is_line_ending).unwrap_or(false) {
        anchor = head;
    }

    // Find our target position(s).
    let head_start = head;
    
    if is_prev {
        while head > 0 {
            let next_ch = all_chars.get(head - 1).copied().unwrap_or('\0');
            
            if prev_ch.is_none() || reached_target(target, prev_ch.unwrap(), next_ch) {
                if head == head_start {
                    anchor = head; // First boundary advances the anchor
                } else {
                    break;
                }
            }
            prev_ch = Some(next_ch);
            advance(&mut head);
        }
        

    } else {
        while head < all_chars.len() {
            let next_ch = all_chars.get(head).copied().unwrap_or('\0');
            
            if prev_ch.is_none() || reached_target(target, prev_ch.unwrap(), next_ch) {
                if head == head_start {
                    anchor = head; // First boundary advances the anchor
                } else {
                    break;
                }
            }
            prev_ch = Some(next_ch);
            advance(&mut head);
        }
    }

    Range::new(anchor, head)
}

/// Check if we've reached the target boundary - exact Helix logic
fn reached_target(target: WordMotionTarget, prev_ch: char, next_ch: char) -> bool {
    match target {
        WordMotionTarget::NextWordStart | WordMotionTarget::PrevWordEnd => {
            is_word_boundary(prev_ch, next_ch)
                && (char_is_line_ending(next_ch) || !next_ch.is_whitespace())
        }
        WordMotionTarget::NextWordEnd | WordMotionTarget::PrevWordStart => {
            is_word_boundary(prev_ch, next_ch)
                && (!prev_ch.is_whitespace() || char_is_line_ending(next_ch))
        }
        WordMotionTarget::NextLongWordStart | WordMotionTarget::PrevLongWordEnd => {
            is_long_word_boundary(prev_ch, next_ch)
                && (char_is_line_ending(next_ch) || !next_ch.is_whitespace())
        }
        WordMotionTarget::NextLongWordEnd | WordMotionTarget::PrevLongWordStart => {
            is_long_word_boundary(prev_ch, next_ch)
                && (!prev_ch.is_whitespace() || char_is_line_ending(next_ch))
        }
        WordMotionTarget::NextSubWordStart => {
            is_sub_word_boundary(prev_ch, next_ch, Direction::Forward)
                && (char_is_line_ending(next_ch) || !(next_ch.is_whitespace() || next_ch == '_'))
        }
        WordMotionTarget::PrevSubWordEnd => {
            is_sub_word_boundary(prev_ch, next_ch, Direction::Backward)
                && (char_is_line_ending(next_ch) || !(next_ch.is_whitespace() || next_ch == '_'))
        }
        WordMotionTarget::NextSubWordEnd => {
            is_sub_word_boundary(prev_ch, next_ch, Direction::Forward)
                && (!(prev_ch.is_whitespace() || prev_ch == '_') || char_is_line_ending(next_ch))
        }
        WordMotionTarget::PrevSubWordStart => {
            is_sub_word_boundary(prev_ch, next_ch, Direction::Backward)
                && (!(prev_ch.is_whitespace() || prev_ch == '_') || char_is_line_ending(next_ch))
        }
    }
}

// Character classification helpers

/// Check if character is considered part of a word
pub fn is_word_char(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_'
}

/// Categorize a character for word boundary detection
pub fn categorize_char(ch: char) -> CharCategory {
    if char_is_line_ending(ch) {
        CharCategory::Eol
    } else if ch.is_whitespace() {
        CharCategory::Whitespace
    } else if is_word_char(ch) {
        CharCategory::Word
    } else {
        // Use more comprehensive punctuation detection like Helix
        CharCategory::Punctuation
    }
}

/// Check if character is a line ending
pub fn char_is_line_ending(ch: char) -> bool {
    matches!(ch, '\n' | '\r')
}

/// Check if there's a word boundary between two characters
pub fn is_word_boundary(a: char, b: char) -> bool {
    categorize_char(a) != categorize_char(b)
}

/// Check if there's a long word boundary between two characters
fn is_long_word_boundary(a: char, b: char) -> bool {
    match (categorize_char(a), categorize_char(b)) {
        (CharCategory::Word, CharCategory::Punctuation)
        | (CharCategory::Punctuation, CharCategory::Word) => false,
        (a, b) if a != b => true,
        _ => false,
    }
}

/// Check if there's a sub-word boundary (camelCase/snake_case)
fn is_sub_word_boundary(a: char, b: char, dir: Direction) -> bool {
    match (categorize_char(a), categorize_char(b)) {
        (CharCategory::Word, CharCategory::Word) => {
            if (a == '_') != (b == '_') {
                return true;
            }
            // Subword boundaries are directional
            match dir {
                Direction::Forward => a.is_lowercase() && b.is_uppercase(),
                Direction::Backward => a.is_uppercase() && b.is_lowercase(),
            }
        }
        (a, b) => a != b,
    }
}

/// Find character functions matching Helix's exact implementation

/// Find next character (f command) - inclusive
pub fn find_next_char(text: &Rope, range: Range, ch: char, count: usize) -> Range {
    find_char_impl(text, range, ch, count, true, crate::helix::core::Direction::Forward)
}

/// Find previous character (F command) - inclusive  
pub fn find_prev_char(text: &Rope, range: Range, ch: char, count: usize) -> Range {
    find_char_impl(text, range, ch, count, true, crate::helix::core::Direction::Backward)
}

/// Till next character (t command) - exclusive
pub fn till_next_char(text: &Rope, range: Range, ch: char, count: usize) -> Range {
    find_char_impl(text, range, ch, count, false, crate::helix::core::Direction::Forward)
}

/// Till previous character (T command) - exclusive
pub fn till_prev_char(text: &Rope, range: Range, ch: char, count: usize) -> Range {
    find_char_impl(text, range, ch, count, false, crate::helix::core::Direction::Backward)
}

/// Core find character implementation matching Helix exactly
fn find_char_impl(text: &Rope, range: Range, ch: char, count: usize, inclusive: bool, direction: Direction) -> Range {
    // Calculate search start position exactly like Helix
    let search_start_pos = if range.anchor < range.head {
        // Forward range: start from head - 1 (like Helix's range.head - 1)
        range.head.saturating_sub(1)
    } else {
        // Backward range or point range: start from head
        range.head
    };
    
    let found_pos = match direction {
        Direction::Forward => find_next_char_impl(text, ch, search_start_pos, count, inclusive),
        Direction::Backward => find_prev_char_impl(text, ch, search_start_pos, count, inclusive),
    };
    
    if let Some(pos) = found_pos {
        // Create selection from current cursor to found position (Helix style)
        // Use range.cursor() to get the current cursor position like Helix does
        let cursor_pos = range.cursor(text);
        
        // Create the new range exactly like Helix: Range::point(cursor).put_cursor(text, pos, true)
        Range::new(cursor_pos, pos)
    } else {
        // No match found, return original range
        range
    }
}

/// Find next character implementation matching Helix exactly
fn find_next_char_impl(text: &Rope, ch: char, pos: usize, n: usize, inclusive: bool) -> Option<usize> {
    // Start search from pos + 1 (like Helix)
    let start_pos = (pos + 1).min(text.len());
    
    if inclusive {
        find_nth_next(text, ch, start_pos, n)
    } else {
        // For exclusive finds (till), check if we're already at the character
        let n = match text.chars().nth(start_pos) {
            Some(next_ch) if next_ch == ch => n + 1,
            _ => n,
        };
        find_nth_next(text, ch, start_pos, n).map(|n| n.saturating_sub(1))
    }
}

/// Find previous character implementation matching Helix exactly  
fn find_prev_char_impl(text: &Rope, ch: char, pos: usize, n: usize, inclusive: bool) -> Option<usize> {
    if inclusive {
        find_nth_prev(text, ch, pos, n)
    } else {
        // For exclusive finds (till), check if we're already at the character
        let n = match text.chars().nth(pos.saturating_sub(1)) {
            Some(next_ch) if next_ch == ch => n + 1,
            _ => n,
        };
        find_nth_prev(text, ch, pos, n).map(|n| (n + 1).min(text.len()))
    }
}

/// Find nth next character occurrence
fn find_nth_next(text: &Rope, ch: char, mut pos: usize, n: usize) -> Option<usize> {
    if pos >= text.len() || n == 0 {
        return None;
    }
    
    let chars: Vec<char> = text.chars().collect();
    
    for _ in 0..n {
        loop {
            if pos >= chars.len() {
                return None;
            }
            
            let c = chars[pos];
            pos += 1;
            
            if c == ch {
                break;
            }
        }
    }
    
    Some(pos - 1)
}

/// Find nth previous character occurrence
fn find_nth_prev(text: &Rope, ch: char, mut pos: usize, n: usize) -> Option<usize> {
    if pos == 0 || n == 0 {
        return None;
    }
    
    let chars: Vec<char> = text.chars().collect();
    
    // Start from pos - 1 to exclude the current position (like find_nth_next starts from pos + 1)
    if pos > 0 {
        pos -= 1;
    } else {
        return None;
    }
    
    for _i in 0..n {
        loop {
            let c = chars[pos];
            
            if c == ch {
                break;
            }
            
            if pos == 0 {
                return None;
            }
            pos -= 1;
        }
        
        // If we need to find more occurrences, continue searching
        if _i < n - 1 {
            if pos == 0 {
                return None;
            }
            pos -= 1;
        }
    }
    
    Some(pos)
}

/// Convert character index to byte offset in a Rope
pub fn char_index_to_byte_offset(text: &Rope, char_index: usize) -> usize {
    text.chars().take(char_index).map(|c| c.len_utf8()).sum()
}

/// Convert byte offset to character index in a Rope
pub fn byte_offset_to_char_index(text: &Rope, byte_offset: usize) -> usize {
    let mut current_byte = 0;
    let mut char_count = 0;
    
    for ch in text.chars() {
        if current_byte >= byte_offset {
            break;
        }
        current_byte += ch.len_utf8();
        char_count += 1;
    }
    
    char_count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_helix_word_vs_word_behavior() {
        // Test the exact example from Helix tutor:
        // "one-of-a-kind" should require 7 keystrokes with w (word movements)
        // but only 1 keystroke with W (WORD movements)
        
        let text = Rope::from("Helix is a one-of-a-kind \"modal\" text editor");
        
        // Test word movements (w) - should stop at punctuation
        let mut range = Range::new(11, 11); // Start at 'o' in "one-of-a-kind"
        
        // First w should go to the dash
        range = move_next_word_start(&text, range, 1);
        
        // Second w should go to "of"
        range = move_next_word_start(&text, range, 1);
        
        // Test WORD movements (W) - should treat punctuation as part of word
        let mut long_range = Range::new(11, 11); // Start at same position
        long_range = move_next_long_word_start(&text, long_range, 1);
        
        // Verify that w stops at punctuation but W doesn't
        assert!(range.head < long_range.head);
    }

    #[test]
    fn test_char_categorization_exact() {
        // Test exact character categorization like Helix
        assert_eq!(categorize_char('a'), CharCategory::Word);
        assert_eq!(categorize_char('_'), CharCategory::Word);  // underscore is word char
        assert_eq!(categorize_char('5'), CharCategory::Word);
        assert_eq!(categorize_char(' '), CharCategory::Whitespace);
        assert_eq!(categorize_char('\n'), CharCategory::Eol);
        assert_eq!(categorize_char('.'), CharCategory::Punctuation);
        assert_eq!(categorize_char('-'), CharCategory::Punctuation);  // dash is punctuation
        assert_eq!(categorize_char('"'), CharCategory::Punctuation);  // quote is punctuation
        
        // Test word boundaries
        assert!(is_word_boundary('a', '-'));  // word to punctuation
        assert!(is_word_boundary('-', 'a'));  // punctuation to word
        assert!(!is_word_boundary('a', '_')); // both are word chars
        assert!(!is_word_boundary('_', 'a')); // both are word chars
        
        // Test long word boundaries
        assert!(!is_long_word_boundary('a', '-'));  // word and punctuation are same for WORD
        assert!(!is_long_word_boundary('-', 'a'));  // word and punctuation are same for WORD
        assert!(is_long_word_boundary('a', ' '));   // word to whitespace
        assert!(is_long_word_boundary('-', ' '));   // punctuation to whitespace
    }

    #[test]
    fn test_basic_word_movement() {
        let tests = [
            ("Basic forward motion stops at the first space",
                vec![(1, Range::new(0, 0), Range::new(0, 6))]),
            (" Starting from a boundary advances the anchor",
                vec![(1, Range::new(0, 0), Range::new(1, 10))]),
            ("Long       whitespace gap is bridged by the head",
                vec![(1, Range::new(0, 0), Range::new(0, 11))]),
            ("    Starting from whitespace moves to last space in sequence",
                vec![(1, Range::new(0, 0), Range::new(0, 4))]),
            ("Starting from mid-word leaves anchor at start position and moves head",
                vec![(1, Range::new(3, 3), Range::new(3, 9))]),
        ];

        for (sample, scenario) in tests {
            for (count, begin, expected_end) in scenario.into_iter() {
                let text = Rope::from(sample);
                let range = move_next_word_start(&text, begin, count);
                assert_eq!(range, expected_end);
            }
        }
    }

    #[test]
    fn debug_boundary_behavior() {
        let text = Rope::from(" Starting from");
        let input_range = Range::new(0, 0);
        
        // Test the range preparation step
        let is_prev = false; // NextWordStart
        let start_range = if is_prev {
            if input_range.anchor < input_range.head {
                Range::new(input_range.head, prev_grapheme_boundary(&text, input_range.head))
            } else {
                Range::new(next_grapheme_boundary(&text, input_range.head), input_range.head)
            }
        } else {
            if input_range.anchor < input_range.head {
                Range::new(prev_grapheme_boundary(&text, input_range.head), input_range.head)
            } else {
                Range::new(input_range.head, next_grapheme_boundary(&text, input_range.head))
            }
        };
        
        // Test the range_to_target step
        let result_range = range_to_target(&text, start_range, WordMotionTarget::NextWordStart);
        
        // Test the full function
        let full_result = move_next_word_start(&text, input_range, 1);
        
        // Expected from Helix test case
        let expected = Range::new(1, 10);
        
        assert_eq!(full_result, expected);
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
    fn test_grapheme_boundaries() {
        let text = Rope::from("hello world");
        
        assert_eq!(next_grapheme_boundary(&text, 0), 1);
        assert_eq!(prev_grapheme_boundary(&text, 5), 4);
        assert_eq!(next_grapheme_boundary(&text, 10), 11);
        assert_eq!(prev_grapheme_boundary(&text, 1), 0);
    }

    #[test]
    fn test_multibyte_grapheme_boundaries() {
        let text = Rope::from("héllo wörld");
        
        // Test with accented characters
        let pos = next_grapheme_boundary(&text, 0);
        assert!(pos > 0);
        
        let prev_pos = prev_grapheme_boundary(&text, pos);
        assert_eq!(prev_pos, 0);
    }

    #[test]
    fn test_helix_whitespace_backward_exact() {
        let text = Rope::from("    Starting from whitespace moves to first space in sequence");
        let input_range = Range::new(0, 4);  // Helix test input
        let expected_range = Range::new(4, 0);  // Helix test expected output
        
        let result_range = move_prev_word_start(&text, input_range, 1);
        
        assert_eq!(result_range, expected_range);
    }

    #[test]
    fn test_cursor_at_position_4_backward() {
        // Our actual scenario: cursor at position 4 (point range)
        let text = Rope::from("    Starting from whitespace moves to first space in sequence");
        let input_range = Range::new(4, 4);  // Cursor at position 4 (point range)
        
        let result_range = move_prev_word_start(&text, input_range, 1);
        
        assert!(result_range.head <= result_range.anchor); // Should be backward selection
        assert!(result_range.head == 0); // Should go to start of whitespace
    }

    #[test]
    fn test_helix_newline_exact() {
        let text = Rope::from("Jumping\n    into starting");
        let input_range = Range::new(0, 7);  // Helix test input (cursor after "Jumping")
        let expected_range = Range::new(8, 12);  // Helix test expected output
        
        let result_range = move_next_word_start(&text, input_range, 1);
        
        assert_eq!(result_range, expected_range);
    }

    #[test]
    fn test_comprehensive_helix_behavior() {
        // Test comprehensive Helix behavior with the tutor example
        let text = Rope::from("Helix is a one-of-a-kind \"modal\" text editor");
        
        // Test 1: Word movements (w) should stop at punctuation
        let mut range = Range::new(11, 11); // Start at 'o' in "one-of-a-kind"
        
        // Let's see what actually happens with successive w movements
        for i in 1..=7 {
            range = move_next_word_start(&text, range, 1);
        }
        
        // Test 2: WORD movements (W) should treat punctuation as part of word
        let mut long_range = Range::new(11, 11); // Start at same position
        long_range = move_next_long_word_start(&text, long_range, 1);
        
        // W should skip over the entire "one-of-a-kind" as one WORD
        assert!(long_range.head > 20);
        
        // Test 3: Character categorization verification
        let test_chars = [
            ('a', CharCategory::Word, "letter"),
            ('_', CharCategory::Word, "underscore"),
            ('5', CharCategory::Word, "digit"),
            ('-', CharCategory::Punctuation, "dash"),
            ('"', CharCategory::Punctuation, "quote"),
            ('.', CharCategory::Punctuation, "period"),
            (' ', CharCategory::Whitespace, "space"),
            ('\n', CharCategory::Eol, "newline"),
        ];
        
        for (ch, expected_category, description) in test_chars {
            let actual_category = categorize_char(ch);
            assert_eq!(actual_category, expected_category);
        }
        
        // Test 4: Word boundary verification
        let boundary_tests = [
            ('a', '-', true, "word to punctuation"),
            ('-', 'a', true, "punctuation to word"),
            ('a', '_', false, "word to word (underscore)"),
            ('_', 'a', false, "word to word (underscore)"),
            ('a', 'b', false, "word to word"),
            ('-', '.', false, "punctuation to punctuation"),
        ];
        
        for (a, b, expected_boundary, description) in boundary_tests {
            let is_boundary = is_word_boundary(a, b);
            assert_eq!(is_boundary, expected_boundary);
        }
        
        // Test 5: Long word boundary verification
        let long_boundary_tests = [
            ('a', '-', false, "word and punctuation are same for WORD"),
            ('-', 'a', false, "punctuation and word are same for WORD"),
            ('a', ' ', true, "word to whitespace"),
            ('-', ' ', true, "punctuation to whitespace"),
            (' ', 'a', true, "whitespace to word"),
            (' ', '-', true, "whitespace to punctuation"),
        ];
        
        for (a, b, expected_boundary, description) in long_boundary_tests {
            let is_boundary = is_long_word_boundary(a, b);
            assert_eq!(is_boundary, expected_boundary);
        }
    }

    #[test]
    fn test_find_character_comprehensive() {
        // Test comprehensive find character behavior based on Helix tutor
        let text = Rope::from("Helix is a one-of-a-kind \"modal\" text editor");
        
        // Test 1: Find forward (f) - should find character and include it
        let range = Range::new(0, 0); // Start at beginning
        let result = find_next_char(&text, range, 'o', 1);
        
        // Should find the 'o' in "one-of-a-kind" at position 11 and include it
        assert!(result.head > result.anchor);
        assert!(text.chars().nth(result.head) == Some('o'));
        
        // Test 2: Find backward (F) - should find character and include it
        let range = Range::new(20, 20); // Start at middle of text
        let result = find_prev_char(&text, range, 'o', 1);
        
        // Should find the 'o' in "one-of-a-kind" and include it
        assert!(text.chars().nth(result.head) == Some('o'));
        
        // Test 3: Till forward (t) - should find character but stop before it
        let range = Range::new(0, 0); // Start at beginning
        let result = till_next_char(&text, range, 'o', 1);
        
        // Should stop before the 'o' in "one-of-a-kind" (exclusive)
        assert!(result.head > result.anchor);
        assert!(text.chars().nth(result.head + 1) == Some('o'));
        
        // Test 4: Till backward (T) - should find character but stop after it
        let range = Range::new(20, 20); // Start at middle of text
        let result = till_prev_char(&text, range, 'o', 1);
        
        // Should stop after the 'o' in "one-of-a-kind" (exclusive)
        assert!(text.chars().nth(result.head - 1) == Some('o'));
        
        // Test 5: Find character that doesn't exist
        let range = Range::new(0, 0);
        let result = find_next_char(&text, range, 'z', 1);
        
        // Should return original range when character not found
        assert_eq!(result, range);
        
        // Test 6: Find with count > 1
        let range = Range::new(0, 0);
        let result = find_next_char(&text, range, 'a', 2); // Find second 'a'
        
        // Should find the second 'a' in the text
        assert!(result.head > result.anchor);
        assert!(text.chars().nth(result.head) == Some('a'));
    }

    #[test]
    fn test_find_character_edge_cases() {
        // Test edge cases for find character functions
        
        // Test 1: Empty text
        let empty_text = Rope::from("");
        let range = Range::new(0, 0);
        let result = find_next_char(&empty_text, range, 'a', 1);
        assert_eq!(result, range);
        
        // Test 2: Single character text
        let single_text = Rope::from("a");
        let range = Range::new(0, 0);
        let result = find_next_char(&single_text, range, 'a', 1);
        
        assert!(result.head > result.anchor);
        
        // Test 3: Find at end of text
        let text = Rope::from("hello");
        let range = Range::new(4, 4); // At 'o'
        let result = find_next_char(&text, range, 'x', 1);
        assert_eq!(result, range);
        
        // Test 4: Find at beginning of text
        let text = Rope::from("hello");
        let range = Range::new(0, 0);
        let result = find_prev_char(&text, range, 'x', 1);
        assert_eq!(result, range);
        
        // Test 5: Multibyte characters
        let text = Rope::from("héllo wörld");
        let range = Range::new(0, 0);
        let result = find_next_char(&text, range, 'ö', 1);
        assert!(result.head > result.anchor);
    }

    #[test]
    fn test_find_character_debug() {
        // Debug the find character implementation
        let text = Rope::from("Helix is a one-of-a-kind \"modal\" text editor");
        
        // Debug character positions
        for (i, ch) in text.chars().enumerate() {
            if i <= 15 {
                println!("Position {}: '{}'", i, ch);
            }
        }
        
        // Test find_next_char step by step
        let range = Range::new(0, 0);
        
        // Test the search start position calculation
        let search_start_pos = if range.anchor < range.head {
            range.head.saturating_sub(1)
        } else {
            range.head
        };
        
        // Test find_next_char_impl directly
        let found_pos = find_next_char_impl(&text, 'o', search_start_pos, 1, true);
        
        if let Some(pos) = found_pos {
            println!("Character at found position: '{}'", text.chars().nth(pos).unwrap_or('?'));
        }
        
        // Test the full function
        let result = find_next_char(&text, range, 'o', 1);
        
        // Test cursor position
        let cursor_pos = range.cursor(&text);
        
        // Test what should happen
        println!("\nExpected behavior:");
        println!("- Should start search from position 1 (after current position 0)");
        println!("- Should find 'o' at position 11 in 'one-of-a-kind'");
        println!("- Should create range from cursor (0) to found position (11)");
        
        // Manual search for 'o'
        for (i, ch) in text.chars().enumerate() {
            if ch == 'o' {
                println!("Found 'o' at position {}", i);
                break;
            }
        }
    }

    #[test]
    fn test_user_reported_issues() {
        // Test the specific issues reported by the user
        let text = Rope::from("Helix is a one-of-a-kind \"modal\" text editor");
        
        // Issue 1: w,e,b should stop at special characters like - _
        let mut range = Range::new(11, 11); // Start at 'o' in "one-of-a-kind"
        
        // Test successive w movements
        for i in 1..=7 {
            range = move_next_word_start(&text, range, 1);
        }
        
        // Verify that w stops at each punctuation mark
        let range1 = move_next_word_start(&text, Range::new(11, 11), 1); // "one"
        let range2 = move_next_word_start(&text, range1, 1); // "-"
        let range3 = move_next_word_start(&text, range2, 1); // "of"
        let range4 = move_next_word_start(&text, range3, 1); // "-"
        
        assert!(range2.head - range2.anchor == 1);
        assert!(range4.head - range4.anchor == 1);
        
        // Issue 2: W,E,B should treat punctuation as part of word
        let range_w = move_next_long_word_start(&text, Range::new(11, 11), 1);
        let selected_w = text.chars().skip(range_w.anchor).take(range_w.head - range_w.anchor).collect::<String>();
        
        // W should skip over the entire "one-of-a-kind " as one WORD
        assert!(range_w.head > 20);
        
        // Issue 3: Test f,F,t,T find character movements
        
        // Test f (find forward, inclusive)
        let f_result = find_next_char(&text, Range::new(0, 0), 'o', 1);
        assert!(text.chars().nth(f_result.head) == Some('o'));
        
        // Test F (find backward, inclusive)  
        let f_back_result = find_prev_char(&text, Range::new(20, 20), 'o', 1);
        assert!(text.chars().nth(f_back_result.head) == Some('o'));
        
        // Test t (till forward, exclusive)
        let t_result = till_next_char(&text, Range::new(0, 0), 'o', 1);
        assert!(text.chars().nth(t_result.head + 1) == Some('o'));
        
        // Test T (till backward, exclusive)
        let t_back_result = till_prev_char(&text, Range::new(20, 20), 'o', 1);
        assert!(text.chars().nth(t_back_result.head - 1) == Some('o'));
        
        // Issue 4: Test that f is not "one character off"
        let precise_f = find_next_char(&text, Range::new(10, 10), 'o', 1); // From space before "one"
        assert_eq!(precise_f.head, 11);
        assert!(text.chars().nth(precise_f.head) == Some('o'));
        
        // Test multiple finds to ensure consistency
        let f1 = find_next_char(&text, Range::new(0, 0), 'o', 1);   // First 'o' at 11
        let f2 = find_next_char(&text, Range::new(12, 12), 'o', 1); // Second 'o' at 15
        let f3 = find_next_char(&text, Range::new(16, 16), 'o', 1); // Third 'o' at 27
        
        assert_eq!(f1.head, 11);
        assert_eq!(f2.head, 15);
        assert_eq!(f3.head, 27);
    }

    #[test]
    fn test_find_backward_f_command_debug() {
        // Debug the specific F command issue reported by user
        let text = Rope::from("hello world test");
        
        // Test F command from different positions
        println!("Text: '{}'", text.chars().collect::<String>());
        for (i, ch) in text.chars().enumerate() {
            println!("Position {}: '{}'", i, ch);
        }
        
        // Test 1: Find 'o' backward from position 10 (should find 'o' at position 7 - nearest backward)
        let range = Range::new(10, 10);
        let result = find_prev_char(&text, range, 'o', 1);
        
        println!("\nTest 1: F command from position 10 looking for 'o'");
        println!("Input range: {:?}", range);
        println!("Result range: {:?}", result);
        println!("Character at result.head: '{}'", text.chars().nth(result.head).unwrap_or('?'));
        println!("Expected: 'o' at position 7 (nearest backward)");
        
        // Verify the result - should find the nearest 'o' backward (at position 7 in "world")
        assert_eq!(text.chars().nth(result.head), Some('o'));
        assert_eq!(result.head, 7); // Should be at position 7 (the 'o' in "world")
        
        // Test 2: Find 'l' backward from position 10 (should find 'l' at position 9 - nearest backward)
        let result2 = find_prev_char(&text, range, 'l', 1);
        
        println!("\nTest 2: F command from position 10 looking for 'l'");
        println!("Result range: {:?}", result2);
        println!("Character at result.head: '{}'", text.chars().nth(result2.head).unwrap_or('?'));
        println!("Expected: 'l' at position 9 (nearest backward)");
        
        assert_eq!(text.chars().nth(result2.head), Some('l'));
        assert_eq!(result2.head, 9); // Should be at position 9 (the 'l' in "world")
        
        // Test 3: Find 'e' backward from position 10 (should find 'e' at position 1)
        let result3 = find_prev_char(&text, range, 'e', 1);
        
        println!("\nTest 3: F command from position 10 looking for 'e'");
        println!("Result range: {:?}", result3);
        println!("Character at result.head: '{}'", text.chars().nth(result3.head).unwrap_or('?'));
        println!("Expected: 'e' at position 1");
        
        assert_eq!(text.chars().nth(result3.head), Some('e'));
        assert_eq!(result3.head, 1); // Should be at position 1 (the 'e' in "hello")
        
        // Test 4: Find 'o' backward from position 6 (should find 'o' at position 4 in "hello")
        let range4 = Range::new(6, 6);
        let result4 = find_prev_char(&text, range4, 'o', 1);
        
        println!("\nTest 4: F command from position 6 looking for 'o'");
        println!("Result range: {:?}", result4);
        println!("Character at result.head: '{}'", text.chars().nth(result4.head).unwrap_or('?'));
        println!("Expected: 'o' at position 4 (in 'hello')");
        
        assert_eq!(text.chars().nth(result4.head), Some('o'));
        assert_eq!(result4.head, 4); // Should be at position 4 (the 'o' in "hello")
    }

    #[test]
    fn test_unicode_arrow_character_debug() {
        // Test the specific issue with arrow characters from the tutor
        let text = Rope::from("          ↑\n          k       * h is on the left\n      ← h   l →   * l is on the right");
        
        println!("=== UNICODE ARROW CHARACTER DEBUG ===");
        println!("Text: '{}'", text.chars().collect::<String>());
        
        // Debug character positions and byte lengths
        for (i, ch) in text.chars().enumerate() {
            if i <= 20 {
                println!("Position {}: '{}' (Unicode: U+{:04X}, byte len: {})", 
                    i, ch, ch as u32, ch.len_utf8());
            }
        }
        
        // Test word movement around the arrow character
        let range_before_arrow = Range::new(9, 9); // Position before ↑
        let range_at_arrow = Range::new(10, 10);   // Position at ↑
        let range_after_arrow = Range::new(11, 11); // Position after ↑
        
        println!("\n=== WORD MOVEMENT TESTS ===");
        
        // Test w movement from before arrow
        let result1 = move_next_word_start(&text, range_before_arrow, 1);
        println!("w from pos 9: {:?} -> {:?}", range_before_arrow, result1);
        
        // Test w movement from at arrow
        let result2 = move_next_word_start(&text, range_at_arrow, 1);
        println!("w from pos 10 (at ↑): {:?} -> {:?}", range_at_arrow, result2);
        
        // Test w movement from after arrow
        let result3 = move_next_word_start(&text, range_after_arrow, 1);
        println!("w from pos 11: {:?} -> {:?}", range_after_arrow, result3);
        
        // Test character categorization for arrow
        let arrow_char = '↑';
        let arrow_category = categorize_char(arrow_char);
        println!("\nArrow character '↑' categorized as: {:?}", arrow_category);
        
        // Test grapheme boundaries around arrow
        println!("\n=== GRAPHEME BOUNDARY TESTS ===");
        for pos in 8..=13 {
            if pos <= text.len() {
                let prev = prev_grapheme_boundary(&text, pos);
                let next = next_grapheme_boundary(&text, pos);
                let ch_at_pos = text.chars().nth(pos).unwrap_or('?');
                println!("pos {}: '{}' -> prev={}, next={}", pos, ch_at_pos, prev, next);
            }
        }
        
        // Test if the issue is with getting stuck
        println!("\n=== SUCCESSIVE MOVEMENT TEST ===");
        let mut current_range = range_at_arrow;
        for i in 1..=5 {
            let new_range = move_next_word_start(&text, current_range, 1);
            println!("Movement {}: {:?} -> {:?}", i, current_range, new_range);
            
            if new_range == current_range {
                println!("STUCK! Movement {} didn't advance", i);
                break;
            }
            current_range = new_range;
        }
    }

    #[test]
    fn test_coordinate_conversion_debug() {
        // Test coordinate conversion with multi-byte characters
        let text = Rope::from("hello ↑ world");
        
        println!("=== COORDINATE CONVERSION DEBUG ===");
        println!("Text: '{}'", text.chars().collect::<String>());
        
        // Debug character positions
        for (i, ch) in text.chars().enumerate() {
            println!("Char index {}: '{}' (Unicode: U+{:04X}, byte len: {})", 
                i, ch, ch as u32, ch.len_utf8());
        }
        
        // Test word movement from before the arrow
        let range_before = Range::new(5, 5); // At space before ↑
        let range_at_arrow = Range::new(6, 6); // At ↑
        let range_after = Range::new(7, 7); // At space after ↑
        
        println!("\n=== WORD MOVEMENT RESULTS ===");
        
        let result1 = move_next_word_start(&text, range_before, 1);
        println!("w from pos 5 (space): {:?} -> {:?}", range_before, result1);
        
        let result2 = move_next_word_start(&text, range_at_arrow, 1);
        println!("w from pos 6 (↑): {:?} -> {:?}", range_at_arrow, result2);
        
        let result3 = move_next_word_start(&text, range_after, 1);
        println!("w from pos 7 (space): {:?} -> {:?}", range_after, result3);
        
        // Test what characters are at the result positions
        println!("\n=== CHARACTER VERIFICATION ===");
        for range in [result1, result2, result3] {
            let anchor_char = text.chars().nth(range.anchor).unwrap_or('?');
            let head_char = text.chars().nth(range.head).unwrap_or('?');
            println!("Range {:?}: anchor='{}', head='{}'", range, anchor_char, head_char);
        }
        
        // Test the cursor positioning
        println!("\n=== CURSOR POSITIONING ===");
        for range in [result1, result2, result3] {
            let cursor_pos = range.cursor(&text);
            let cursor_char = text.chars().nth(cursor_pos).unwrap_or('?');
            println!("Range {:?}: cursor at pos {} ('{}')", range, cursor_pos, cursor_char);
        }
        
        // Test successive movements to see if they get stuck
        println!("\n=== SUCCESSIVE MOVEMENT TEST ===");
        let mut current = range_at_arrow;
        for i in 1..=3 {
            let next = move_next_word_start(&text, current, 1);
            println!("Step {}: {:?} -> {:?}", i, current, next);
            if next == current {
                println!("STUCK at step {}!", i);
                break;
            }
            current = next;
        }
    }

    #[test]
    fn test_tutor_section_1_1_unicode_misalignment() {
        // Reproduce the exact issue from tutor section 1.1 with Unicode arrows
        let tutor_text = Rope::from("=================================================================\n=                  1.1 BASIC CURSOR MOVEMENT                    =\n=================================================================\n\n          ↑\n          k       * h is on the left\n      ← h   l →   * l is on the right\n          j       * j looks like a down arrow\n          ↓\n\n The cursor can be moved using the h, j, k, l keys, as shown");
        
        println!("=== TUTOR SECTION 1.1 UNICODE MISALIGNMENT TEST ===");
        
        // Find the line with the up arrow
        let text_string = tutor_text.to_string();
        let lines: Vec<&str> = text_string.lines().collect();
        for (line_num, line) in lines.iter().enumerate() {
            if line.contains('↑') {
                println!("Line {}: '{}'", line_num, line);
                
                // Test character positions in this line
                for (char_pos, ch) in line.chars().enumerate() {
                    println!("  Char {}: '{}' (U+{:04X}, {} bytes)", 
                        char_pos, ch, ch as u32, ch.len_utf8());
                }
                
                // Calculate character offset to start of this line in the rope
                let mut char_offset = 0;
                for i in 0..line_num {
                    char_offset += lines[i].chars().count() + 1; // +1 for newline
                }
                
                // Find the arrow character position within the line
                let arrow_char_pos = line.chars().position(|c| c == '↑').unwrap();
                let global_char_pos = char_offset + arrow_char_pos;
                
                println!("Arrow '↑' at line char pos {}, global char pos {}", arrow_char_pos, global_char_pos);
                
                // Test movement from the arrow position
                let range_at_arrow = Range::new(global_char_pos, global_char_pos);
                let result = move_next_word_start(&tutor_text, range_at_arrow, 1);
                
                println!("Word movement from arrow: {:?} -> {:?}", range_at_arrow, result);
                
                // Test if we can get the character at the result position
                if let Some(result_char) = tutor_text.chars().nth(result.head) {
                    println!("Character at result.head: '{}'", result_char);
                } else {
                    println!("ERROR: No character at result.head position {}", result.head);
                }
                
                break;
            }
        }
        
        // Test successive movements to see where misalignment occurs
        println!("\n=== SUCCESSIVE MOVEMENT TEST ===");
        let arrow_byte_pos = text_string.find('↑').unwrap();
        let char_pos = text_string.chars().take_while(|_| {
            text_string.char_indices().take_while(|(i, _)| *i < arrow_byte_pos).count() > 0
        }).count();
        
        // Simpler approach: find character index directly
        let char_pos = tutor_text.chars().position(|c| c == '↑').unwrap();
        
        let mut current_range = Range::new(char_pos, char_pos);
        for i in 1..=5 {
            let new_range = move_next_word_start(&tutor_text, current_range, 1);
            println!("Movement {}: pos {} -> pos {}", i, current_range.head, new_range.head);
            
            // Check what character we landed on
            if let Some(ch) = tutor_text.chars().nth(new_range.head) {
                println!("  Landed on: '{}' (U+{:04X})", ch, ch as u32);
            }
            
            if new_range == current_range {
                println!("  STUCK!");
                break;
            }
            current_range = new_range;
        }
    }

    #[test]
    fn test_byte_vs_char_offset_issue() {
        // Test the specific issue: our functions return character indices but Zed expects byte offsets
        let text = Rope::from("hello ↑ world");
        
        println!("=== BYTE VS CHARACTER OFFSET DEBUG ===");
        println!("Text: '{}'", text.chars().collect::<String>());
        
        // Show the difference between byte and character positions
        let mut byte_pos = 0;
        for (char_idx, ch) in text.chars().enumerate() {
            println!("Char {}: '{}' at byte {}, {} bytes long", 
                char_idx, ch, byte_pos, ch.len_utf8());
            byte_pos += ch.len_utf8();
        }
        
        // Test our word movement function
        let range = Range::new(6, 6); // At the arrow character (character index 6)
        let result = move_next_word_start(&text, range, 1);
        
        println!("\nWord movement result:");
        println!("Input range: {:?} (character indices)", range);
        println!("Output range: {:?} (character indices)", result);
        
        // Show what characters are at these positions
        if let Some(anchor_char) = text.chars().nth(result.anchor) {
            println!("Character at anchor {}: '{}'", result.anchor, anchor_char);
        }
        if let Some(head_char) = text.chars().nth(result.head) {
            println!("Character at head {}: '{}'", result.head, head_char);
        }
        
        // Show what the byte offsets would be
        let anchor_byte_offset = text.chars().take(result.anchor).map(|c| c.len_utf8()).sum::<usize>();
        let head_byte_offset = text.chars().take(result.head).map(|c| c.len_utf8()).sum::<usize>();
        
        println!("\nCorresponding byte offsets:");
        println!("Anchor char {} -> byte {}", result.anchor, anchor_byte_offset);
        println!("Head char {} -> byte {}", result.head, head_byte_offset);
        
        // The issue: character indices != byte offsets for Unicode text
        // In this case, head character 8 corresponds to byte 10 (due to 3-byte ↑ character)
        println!("\nThe issue: character {} != byte {}", result.head, head_byte_offset);
        assert_eq!(result.head, 8);      // Character index
        assert_eq!(head_byte_offset, 10); // Byte offset
        
        // This demonstrates the coordinate conversion problem:
        // Our Helix functions return character indices, but Zed's snapshot.offset_to_point() expects byte offsets
    }

    #[test]
    fn test_coordinate_conversion_fix() {
        // Test that the coordinate conversion fix resolves Unicode issues
        let text = Rope::from("hello ↑ world");
        
        println!("=== COORDINATE CONVERSION FIX TEST ===");
        
        // Test the conversion functions
        let char_index = 8; // Character 'w' in "world"
        let byte_offset = char_index_to_byte_offset(&text, char_index);
        let back_to_char = byte_offset_to_char_index(&text, byte_offset);
        
        println!("Character {} -> byte {} -> character {}", char_index, byte_offset, back_to_char);
        assert_eq!(char_index, back_to_char);
        
        // Test with the arrow character
        let arrow_char_index = 6; // The ↑ character
        let arrow_byte_offset = char_index_to_byte_offset(&text, arrow_char_index);
        let arrow_back_to_char = byte_offset_to_char_index(&text, arrow_byte_offset);
        
        println!("Arrow character {} -> byte {} -> character {}", arrow_char_index, arrow_byte_offset, arrow_back_to_char);
        assert_eq!(arrow_char_index, arrow_back_to_char);
        
        // Verify the actual byte positions
        assert_eq!(byte_offset, 10); // 'w' is at byte 10
        assert_eq!(arrow_byte_offset, 6); // '↑' starts at byte 6
        
        println!("Coordinate conversion functions working correctly!");
    }
}