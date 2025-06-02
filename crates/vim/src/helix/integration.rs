//! Integration layer between Helix core functions and Zed editor
//!
//! This module provides the bridge between the pure Helix movement functions
//! and Zed's editor infrastructure, converting between different coordinate
//! systems and selection representations.

use super::core::{self, Range as HelixRange, WordMotionTarget};
use crate::{Vim, Mode};
use editor::{Editor, movement, scroll::Autoscroll, SelectionGoal, DisplayPoint};
use gpui::{Window, Context};
use rope::Rope;

impl Vim {
    /// Convert Zed editor text to Rope for Helix functions
    fn editor_to_rope(&self, editor: &Editor) -> Rope {
        let buffer = editor.buffer().read(editor.cx());
        let snapshot = buffer.snapshot();
        Rope::from(snapshot.text())
    }

    /// Convert Zed selection to Helix range
    fn zed_selection_to_helix_range(&self, editor: &Editor, selection: &editor::Selection) -> HelixRange {
        let buffer = editor.buffer().read(editor.cx());
        let snapshot = buffer.snapshot();
        
        let anchor_offset = selection.tail().to_offset(&snapshot, editor::Bias::Left);
        let head_offset = selection.head().to_offset(&snapshot, editor::Bias::Left);
        
        HelixRange::new(anchor_offset, head_offset)
    }

    /// Convert Helix range back to Zed selection
    fn helix_range_to_zed_selection(
        &self,
        editor: &Editor,
        helix_range: HelixRange,
        goal: SelectionGoal,
    ) -> editor::Selection {
        let buffer = editor.buffer().read(editor.cx());
        let snapshot = buffer.snapshot();
        
        let anchor_point = snapshot.offset_to_point(helix_range.anchor);
        let head_point = snapshot.offset_to_point(helix_range.head);
        
        let anchor_display = editor.display_point_for_anchor(
            &snapshot.anchor_at(anchor_point, editor::Bias::Left),
            &editor.display_map.read(),
        );
        let head_display = editor.display_point_for_anchor(
            &snapshot.anchor_at(head_point, editor::Bias::Left),
            &editor.display_map.read(),
        );
        
        editor::Selection::new(anchor_display, head_display, goal)
    }

    /// Perform Helix word movement using core functions
    pub fn helix_word_move_core(
        &mut self,
        target: WordMotionTarget,
        count: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.is_helix_select_mode() {
            // In select mode, extend existing selections
            self.update_editor(window, cx, |_, editor, window, cx| {
                let rope = Self::editor_to_rope_static(editor);
                editor.change_selections(Some(Autoscroll::fit()), window, cx, |s| {
                    s.move_with(|_map, selection| {
                        let helix_range = Self::zed_selection_to_helix_range_static(editor, selection);
                        let new_range = match target {
                            WordMotionTarget::NextWordStart => {
                                core::move_next_word_start(&rope, helix_range, count)
                            },
                            WordMotionTarget::NextWordEnd => {
                                core::move_next_word_end(&rope, helix_range, count)
                            },
                            WordMotionTarget::PrevWordStart => {
                                core::move_prev_word_start(&rope, helix_range, count)
                            },
                            WordMotionTarget::PrevWordEnd => {
                                core::move_prev_word_end(&rope, helix_range, count)
                            },
                            WordMotionTarget::NextLongWordStart => {
                                core::move_next_long_word_start(&rope, helix_range, count)
                            },
                            WordMotionTarget::NextLongWordEnd => {
                                core::move_next_long_word_end(&rope, helix_range, count)
                            },
                            WordMotionTarget::PrevLongWordStart => {
                                core::move_prev_long_word_start(&rope, helix_range, count)
                            },
                            WordMotionTarget::PrevLongWordEnd => {
                                core::move_prev_long_word_end(&rope, helix_range, count)
                            },
                            _ => helix_range, // Sub-word movements not implemented yet
                        };
                        
                        // Only move the head in select mode
                        let buffer = editor.buffer().read(cx);
                        let snapshot = buffer.snapshot();
                        let new_head_point = snapshot.offset_to_point(new_range.head);
                        let new_head_display = editor.display_point_for_anchor(
                            &snapshot.anchor_at(new_head_point, editor::Bias::Left),
                            &editor.display_map.read(),
                        );
                        
                        selection.set_head(new_head_display, selection.goal);
                    });
                });
            });
        } else {
            // In normal mode, create new selections
            self.update_editor(window, cx, |_, editor, window, cx| {
                let rope = Self::editor_to_rope_static(editor);
                editor.change_selections(Some(Autoscroll::fit()), window, cx, |s| {
                    s.move_with(|_map, selection| {
                        let start_offset = selection.head().to_offset(&editor.buffer().read(cx).snapshot(), editor::Bias::Left);
                        let start_range = HelixRange::point(start_offset);
                        
                        let new_range = match target {
                            WordMotionTarget::NextWordStart => {
                                core::move_next_word_start(&rope, start_range, count)
                            },
                            WordMotionTarget::NextWordEnd => {
                                core::move_next_word_end(&rope, start_range, count)
                            },
                            WordMotionTarget::PrevWordStart => {
                                core::move_prev_word_start(&rope, start_range, count)
                            },
                            WordMotionTarget::PrevWordEnd => {
                                core::move_prev_word_end(&rope, start_range, count)
                            },
                            WordMotionTarget::NextLongWordStart => {
                                core::move_next_long_word_start(&rope, start_range, count)
                            },
                            WordMotionTarget::NextLongWordEnd => {
                                core::move_next_long_word_end(&rope, start_range, count)
                            },
                            WordMotionTarget::PrevLongWordStart => {
                                core::move_prev_long_word_start(&rope, start_range, count)
                            },
                            WordMotionTarget::PrevLongWordEnd => {
                                core::move_prev_long_word_end(&rope, start_range, count)
                            },
                            _ => start_range, // Sub-word movements not implemented yet
                        };
                        
                        // Set both anchor and head for new selection in normal mode
                        let buffer = editor.buffer().read(cx);
                        let snapshot = buffer.snapshot();
                        
                        let anchor_point = snapshot.offset_to_point(new_range.anchor);
                        let head_point = snapshot.offset_to_point(new_range.head);
                        
                        let anchor_display = editor.display_point_for_anchor(
                            &snapshot.anchor_at(anchor_point, editor::Bias::Left),
                            &editor.display_map.read(),
                        );
                        let head_display = editor.display_point_for_anchor(
                            &snapshot.anchor_at(head_point, editor::Bias::Left),
                            &editor.display_map.read(),
                        );
                        
                        selection.set_tail(anchor_display, selection.goal);
                        selection.set_head(head_display, selection.goal);
                    });
                });
            });
        }
    }

    // Static helper functions for use in closures
    fn editor_to_rope_static(editor: &Editor) -> Rope {
        let buffer = editor.buffer().read(editor.cx());
        let snapshot = buffer.snapshot();
        Rope::from(snapshot.text())
    }

    fn zed_selection_to_helix_range_static(editor: &Editor, selection: &editor::Selection) -> HelixRange {
        let buffer = editor.buffer().read(editor.cx());
        let snapshot = buffer.snapshot();
        
        let anchor_offset = selection.tail().to_offset(&snapshot, editor::Bias::Left);
        let head_offset = selection.head().to_offset(&snapshot, editor::Bias::Left);
        
        HelixRange::new(anchor_offset, head_offset)
    }
}

// Action implementations using the new core functions

impl Vim {
    /// Move to start of next word using Helix semantics
    pub fn helix_move_next_word_start(
        &mut self,
        window: &mut Window, 
        cx: &mut Context<Self>
    ) {
        self.helix_word_move_core(WordMotionTarget::NextWordStart, 1, window, cx);
    }

    /// Move to end of next word using Helix semantics
    pub fn helix_move_next_word_end(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>
    ) {
        self.helix_word_move_core(WordMotionTarget::NextWordEnd, 1, window, cx);
    }

    /// Move to start of previous word using Helix semantics
    pub fn helix_move_prev_word_start(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>
    ) {
        self.helix_word_move_core(WordMotionTarget::PrevWordStart, 1, window, cx);
    }

    /// Move to end of previous word using Helix semantics
    pub fn helix_move_prev_word_end(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>
    ) {
        self.helix_word_move_core(WordMotionTarget::PrevWordEnd, 1, window, cx);
    }

    /// Move to start of next long word (ignore punctuation) using Helix semantics
    pub fn helix_move_next_long_word_start(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>
    ) {
        self.helix_word_move_core(WordMotionTarget::NextLongWordStart, 1, window, cx);
    }

    /// Move to end of next long word (ignore punctuation) using Helix semantics
    pub fn helix_move_next_long_word_end(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>
    ) {
        self.helix_word_move_core(WordMotionTarget::NextLongWordEnd, 1, window, cx);
    }

    /// Move to start of previous long word (ignore punctuation) using Helix semantics
    pub fn helix_move_prev_long_word_start(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>
    ) {
        self.helix_word_move_core(WordMotionTarget::PrevLongWordStart, 1, window, cx);
    }

    /// Move to end of previous long word (ignore punctuation) using Helix semantics
    pub fn helix_move_prev_long_word_end(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>
    ) {
        self.helix_word_move_core(WordMotionTarget::PrevLongWordEnd, 1, window, cx);
    }
}