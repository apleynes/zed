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
        if self.head > self.anchor {
            // In forward selections, cursor is before the head position
            prev_grapheme_boundary(text, self.head)
        } else {
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

// Simplified grapheme boundary helpers for Zed rope
pub fn prev_grapheme_boundary(_text: &Rope, char_idx: usize) -> usize {
    char_idx.saturating_sub(1)
}

pub fn next_grapheme_boundary(text: &Rope, char_idx: usize) -> usize {
    std::cmp::min(char_idx + 1, text.len())
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

    // Prepare the range appropriately based on the target movement direction
    #[allow(clippy::collapsible_else_if)]
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
fn range_to_target(text: &Rope, origin: Range, target: WordMotionTarget) -> Range {
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
    
    // Get characters starting from head
    let chars: Vec<char> = text.chars_at(head).collect();
    let mut char_index = 0;
    
    // Get previous character for boundary detection
    let mut prev_ch = if head > 0 {
        text.chars_at(head.saturating_sub(1)).next()
    } else {
        None
    };

    // Skip any initial newline characters
    let head_start = head;
    while char_index < chars.len() {
        let ch = chars[char_index];
        if char_is_line_ending(ch) {
            prev_ch = Some(ch);
            head += 1;
            char_index += 1;
        } else {
            break;
        }
    }
    
    if prev_ch.map(char_is_line_ending).unwrap_or(false) {
        anchor = head;
    }

    // Find our target position
    while char_index < chars.len() {
        let next_ch = chars[char_index];
        
        if let Some(prev) = prev_ch {
            if reached_target(target, prev, next_ch) {
                if head == head_start {
                    anchor = head;
                } else {
                    break;
                }
            }
        }
        
        prev_ch = Some(next_ch);
        head += 1;
        char_index += 1;
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
    } else if ch.is_ascii_punctuation() {
        CharCategory::Punctuation
    } else {
        CharCategory::Unknown
    }
}

/// Check if character is a line ending
fn char_is_line_ending(ch: char) -> bool {
    matches!(ch, '\n' | '\r')
}

/// Check if there's a word boundary between two characters
fn is_word_boundary(a: char, b: char) -> bool {
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

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_char_categorization() {
        assert_eq!(categorize_char('a'), CharCategory::Word);
        assert_eq!(categorize_char('_'), CharCategory::Word);
        assert_eq!(categorize_char('5'), CharCategory::Word);
        assert_eq!(categorize_char(' '), CharCategory::Whitespace);
        assert_eq!(categorize_char('\n'), CharCategory::Eol);
        assert_eq!(categorize_char('.'), CharCategory::Punctuation);
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
}