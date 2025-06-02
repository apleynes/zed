//! Core Helix movement and selection functions
//! 
//! This module implements pure functions for Helix-style text navigation and selection,
//! mirroring the structure and behavior of helix-core/src/movement.rs.
//! 
//! The key insight is that Helix movements are selection operations that span entire
//! text ranges including whitespace, not cursor movements to word boundaries.

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
    // Long words (ignore punctuation - equivalent to W, E, B commands)
    NextLongWordStart,
    NextLongWordEnd,
    PrevLongWordStart, 
    PrevLongWordEnd,
    // Sub words (camelCase/snake_case boundaries)
    NextSubWordStart,
    NextSubWordEnd,
    PrevSubWordStart,
    PrevSubWordEnd,
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
    pub fn put_cursor(&self, text: &Rope, pos: usize, extend: bool) -> Self {
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

// Core word movement implementation

/// Internal word movement function that handles all word motion targets
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
    if (is_prev && range.head == 0) || (!is_prev && range.head >= text.len()) {
        return range;
    }

    // Prepare the range appropriately based on movement direction
    // This handles block-cursor semantics and anchor positioning
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

    // Perform the movement iterations
    let mut result_range = start_range;
    for _ in 0..count {
        let next_range = find_word_boundary(text, result_range, target);
        if result_range == next_range {
            break;
        }
        result_range = next_range;
    }
    
    result_range
}

/// Find the next word boundary based on the target motion
fn find_word_boundary(text: &Rope, range: Range, target: WordMotionTarget) -> Range {
    let pos = range.head;
    
    match target {
        WordMotionTarget::NextWordStart => find_next_word_start(text, range, false),
        WordMotionTarget::NextWordEnd => find_next_word_end(text, range, false),
        WordMotionTarget::PrevWordStart => find_prev_word_start(text, range, false),
        WordMotionTarget::PrevWordEnd => find_prev_word_end(text, range, false),
        WordMotionTarget::NextLongWordStart => find_next_word_start(text, range, true),
        WordMotionTarget::NextLongWordEnd => find_next_word_end(text, range, true),
        WordMotionTarget::PrevLongWordStart => find_prev_word_start(text, range, true),
        WordMotionTarget::PrevLongWordEnd => find_prev_word_end(text, range, true),
        WordMotionTarget::NextSubWordStart => find_next_sub_word_start(text, range),
        WordMotionTarget::NextSubWordEnd => find_next_sub_word_end(text, range),
        WordMotionTarget::PrevSubWordStart => find_prev_sub_word_start(text, range),
        WordMotionTarget::PrevSubWordEnd => find_prev_sub_word_end(text, range),
    }
}

// Word boundary finding functions

fn find_next_word_start(text: &Rope, range: Range, ignore_punctuation: bool) -> Range {
    let start_pos = range.head;
    let mut pos = start_pos;
    
    // Skip current word if we're in one
    while pos < text.len() && is_word_char(text.char(pos), ignore_punctuation) {
        pos += 1;
    }
    
    // Skip whitespace
    while pos < text.len() && text.char(pos).is_whitespace() {
        pos += 1;
    }
    
    // We're now at the start of the next word, but Helix selections include
    // the entire word plus trailing whitespace
    let word_start = pos;
    
    // Find end of this word
    while pos < text.len() && is_word_char(text.char(pos), ignore_punctuation) {
        pos += 1;
    }
    
    // Include trailing whitespace in selection
    while pos < text.len() && text.char(pos).is_whitespace() && text.char(pos) != '\n' {
        pos += 1;
    }
    
    Range::new(range.anchor, pos)
}

fn find_next_word_end(text: &Rope, range: Range, ignore_punctuation: bool) -> Range {
    let mut pos = range.head;
    
    // Move to end of current word if we're in one
    while pos < text.len() && is_word_char(text.char(pos), ignore_punctuation) {
        pos += 1;
    }
    
    // Skip whitespace to next word
    while pos < text.len() && text.char(pos).is_whitespace() {
        pos += 1;
    }
    
    // Move to end of the next word
    while pos < text.len() && is_word_char(text.char(pos), ignore_punctuation) {
        pos += 1;
    }
    
    // Back up one to be ON the last character, not after it
    if pos > 0 {
        pos -= 1;
    }
    
    Range::new(range.anchor, pos)
}

fn find_prev_word_start(text: &Rope, range: Range, ignore_punctuation: bool) -> Range {
    let mut pos = range.head;
    
    if pos == 0 {
        return range;
    }
    
    // Move backward to find previous word start
    pos -= 1;
    
    // Skip whitespace
    while pos > 0 && text.char(pos).is_whitespace() {
        pos -= 1;
    }
    
    // Skip to start of current word
    while pos > 0 && is_word_char(text.char(pos), ignore_punctuation) {
        pos -= 1;
    }
    
    // Move forward one to be at start of word
    if !is_word_char(text.char(pos), ignore_punctuation) {
        pos += 1;
    }
    
    Range::new(range.anchor, pos)
}

fn find_prev_word_end(text: &Rope, range: Range, ignore_punctuation: bool) -> Range {
    let mut pos = range.head;
    
    if pos == 0 {
        return range;
    }
    
    pos -= 1;
    
    // Skip whitespace
    while pos > 0 && text.char(pos).is_whitespace() {
        pos -= 1;
    }
    
    Range::new(range.anchor, pos)
}

// Sub-word movement stubs (for camelCase/snake_case)
fn find_next_sub_word_start(text: &Rope, range: Range) -> Range {
    // TODO: Implement sub-word movement
    find_next_word_start(text, range, false)
}

fn find_next_sub_word_end(text: &Rope, range: Range) -> Range {
    // TODO: Implement sub-word movement  
    find_next_word_end(text, range, false)
}

fn find_prev_sub_word_start(text: &Rope, range: Range) -> Range {
    // TODO: Implement sub-word movement
    find_prev_word_start(text, range, false)
}

fn find_prev_sub_word_end(text: &Rope, range: Range) -> Range {
    // TODO: Implement sub-word movement
    find_prev_word_end(text, range, false)
}

// Character classification helpers

fn is_word_char(ch: char, ignore_punctuation: bool) -> bool {
    if ignore_punctuation {
        // Long word: only whitespace separates words
        !ch.is_whitespace()
    } else {
        // Regular word: alphanumeric and underscore
        ch.is_alphanumeric() || ch == '_'
    }
}

fn is_word_boundary(left: char, right: char, ignore_punctuation: bool) -> bool {
    let left_word = is_word_char(left, ignore_punctuation);
    let right_word = is_word_char(right, ignore_punctuation);
    left_word != right_word
}

// Grapheme boundary helpers (simplified for now)

fn prev_grapheme_boundary(text: &Rope, pos: usize) -> usize {
    if pos == 0 {
        0
    } else {
        pos.saturating_sub(1)
    }
}

fn next_grapheme_boundary(text: &Rope, pos: usize) -> usize {
    std::cmp::min(pos + 1, text.len())
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
    fn test_long_word_movement() {
        let tests = [
            ("alphanumeric.!,and.?=punctuation are not treated any differently than alphanumerics",
                vec![(1, Range::new(0, 0), Range::new(0, 32))]),
            (".._.._ punctuation is joined by underscores into a single word",
                vec![(1, Range::new(0, 0), Range::new(0, 7))]),
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
    fn test_range_operations() {
        let range = Range::new(5, 10);
        assert_eq!(range.from(), 5);
        assert_eq!(range.to(), 10);
        assert_eq!(range.len(), 5);
        assert_eq!(range.direction(), Direction::Forward);
        
        let flipped = range.flip();
        assert_eq!(flipped.anchor, 10);
        assert_eq!(flipped.head, 5);
        assert_eq!(flipped.direction(), Direction::Backward);
    }
}