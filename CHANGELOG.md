# Changelog

All notable changes to the Zed Helix Mode fork will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.191.0-helix.1] - 2025-06-09

### Added
- **Complete Helix Editor Mode Implementation**
  - Full Helix-style modal editing with normal, insert, and visual modes
  - Comprehensive motion system supporting all standard Helix movements
  - Advanced text object support (word, paragraph, brackets, quotes, etc.)
  - Complete operator system (change, delete, yank) with motion combinations
  - Multi-selection editing with Helix-style selection manipulation
  - Search and replace functionality with Helix keybindings
  - Window and pane management with Helix-style commands
  - Buffer and file operations using Helix navigation patterns

- **Helix Keymap System**
  - Full implementation of Helix default keybindings
  - Mode-aware key handling and command dispatch
  - Proper key sequence handling for complex commands
  - Visual feedback for current mode and selections

- **Selection and Movement**
  - Character, word, line, and paragraph movements
  - Beginning/end of line and document navigation
  - Search-based navigation and selection
  - Bracket and quote matching with selection
  - Advanced selection manipulation (extend, reduce, split)

- **Text Editing Operations**
  - Change, delete, and yank operations with proper motion support
  - Replace functionality with character and regex support
  - Undo/redo system compatible with Helix expectations
  - Line joining and splitting operations
  - Case conversion and text transformation

- **Match Mode Implementation**
  - Bracket matching and navigation
  - Quote mark matching with proper handling
  - Intelligent pair selection and manipulation

### Technical Implementation
- Integrated with Zed's existing editor infrastructure
- Proper state management for modal editing
- Comprehensive test coverage for all Helix operations
- Performance optimizations for multi-selection scenarios

### Documentation
- Complete implementation tracking and testing documentation
- Development workflow and release management processes
- User guides for Helix mode transition

### Notes
This release represents the culmination of extensive development to bring full Helix editor functionality to Zed. The implementation maintains compatibility with Zed's existing features while providing a complete Helix editing experience.

## Previous Versions

This is the first release of the Zed Helix Mode fork. Previous development was based on the upstream Zed project version 0.191.0.