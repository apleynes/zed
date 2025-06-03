# Helix to Zed Keymap Implementation Tracking

This document tracks the implementation status of all Helix keymaps in Zed, following the exact structure and groupings from the [official Helix keymap documentation](https://docs.helix-editor.com/keymap.html).

## Legend

- ✅ **Implemented** - Fully working with tests
- 🚧 **Partial** - Basic implementation, may need refinement
- ❌ **Not Implemented** - Not yet implemented
- 🔄 **In Progress** - Currently being worked on
- 📝 **Planned** - Planned for implementation

---

## Normal Mode

Normal mode is the default mode when you launch helix. You can return to it from other modes by pressing the `Escape` key.

### Movement

> NOTE: Unlike Vim, `f`, `F`, `t` and `T` are not confined to the current line.

| Key                   | Description                                        | Status | Notes |
| -----                 | -----------                                        | ------ | ----- |
| `h`, `Left`           | Move left                                          | ✅ | Uses vim infrastructure |
| `j`, `Down`           | Move down                                          | ✅ | Uses vim infrastructure |
| `k`, `Up`             | Move up                                            | ✅ | Uses vim infrastructure |
| `l`, `Right`          | Move right                                         | ✅ | Uses vim infrastructure |
| `w`                   | Move next word start                               | ✅ | Full Helix implementation with tests |
| `b`                   | Move previous word start                           | ✅ | Full Helix implementation with tests |
| `e`                   | Move next word end                                 | ✅ | Full Helix implementation with tests |
| `W`                   | Move next WORD start                               | ✅ | Full Helix implementation with tests |
| `B`                   | Move previous WORD start                           | ✅ | Full Helix implementation with tests |
| `E`                   | Move next WORD end                                 | ✅ | Full Helix implementation with tests |
| `t`                   | Find till next char                                | ✅ | Full Helix implementation with tests |
| `f`                   | Find next char                                     | ✅ | Full Helix implementation with tests |
| `T`                   | Find till previous char                            | ✅ | Full Helix implementation with tests |
| `F`                   | Find previous char                                 | ✅ | Full Helix implementation with tests |
| `G`                   | Go to line number `<n>`                            | ✅ | Full Helix implementation |
| `Alt-.`               | Repeat last motion (`f`, `t`, `m`, `[` or `]`)     | ❌ | Not implemented |
| `Home`                | Move to the start of the line                      | ✅ | Uses vim infrastructure |
| `End`                 | Move to the end of the line                        | ✅ | Uses vim infrastructure |
| `Ctrl-b`, `PageUp`    | Move page up                                       | ✅ | Uses vim infrastructure |
| `Ctrl-f`, `PageDown`  | Move page down                                     | ✅ | Uses vim infrastructure |
| `Ctrl-u`              | Move cursor and page half page up                  | ✅ | Uses vim infrastructure |
| `Ctrl-d`              | Move cursor and page half page down                | ✅ | Uses vim infrastructure |
| `Ctrl-i`              | Jump forward on the jumplist                       | ✅ | Uses vim infrastructure |
| `Ctrl-o`              | Jump backward on the jumplist                      | ✅ | Uses vim infrastructure |
| `Ctrl-s`              | Save the current selection to the jumplist         | ❌ | Not implemented |

### Changes

| Key         | Description                                                          | Status | Notes |
| -----       | -----------                                                          | ------ | ----- |
| `r`         | Replace with a character                                             | ✅ | Uses vim infrastructure |
| `R`         | Replace with yanked text                                             | ❌ | Not implemented |
| `~`         | Switch case of the selected text                                     | ❌ | Not implemented |
| `` ` ``     | Set the selected text to lower case                                  | ❌ | Not implemented |
| `` Alt-` `` | Set the selected text to upper case                                  | ❌ | Not implemented |
| `i`         | Insert before selection                                              | ✅ | Uses vim infrastructure |
| `a`         | Insert after selection (append)                                      | ✅ | Uses vim infrastructure |
| `I`         | Insert at the start of the line                                      | ✅ | Uses vim infrastructure |
| `A`         | Insert at the end of the line                                        | ✅ | Uses vim infrastructure |
| `o`         | Open new line below selection                                        | ✅ | Uses vim infrastructure |
| `O`         | Open new line above selection                                        | ✅ | Uses vim infrastructure |
| `.`         | Repeat last insert                                                   | ✅ | Uses vim infrastructure |
| `u`         | Undo change                                                          | ✅ | Uses vim infrastructure |
| `U`         | Redo change                                                          | ✅ | Uses vim infrastructure |
| `Alt-u`     | Move backward in history                                             | ❌ | Not implemented |
| `Alt-U`     | Move forward in history                                              | ❌ | Not implemented |
| `y`         | Yank selection                                                       | ✅ | Uses vim infrastructure |
| `p`         | Paste after selection                                                | ✅ | Uses vim infrastructure |
| `P`         | Paste before selection                                               | ✅ | Uses vim infrastructure |
| `"` `<reg>` | Select a register to yank to or paste from                           | ✅ | Uses vim infrastructure |
| `>`         | Indent selection                                                     | ✅ | Uses vim infrastructure |
| `<`         | Unindent selection                                                   | ✅ | Uses vim infrastructure |
| `=`         | Format selection (**LSP**)                                           | ✅ | Uses vim infrastructure |
| `d`         | Delete selection                                                     | ✅ | Uses vim infrastructure |
| `Alt-d`     | Delete selection, without yanking                                    | ❌ | Not implemented |
| `c`         | Change selection (delete and enter insert mode)                      | ✅ | Uses vim infrastructure |
| `Alt-c`     | Change selection (delete and enter insert mode, without yanking)     | ❌ | Not implemented |
| `Ctrl-a`    | Increment object (number) under cursor                               | ✅ | Uses vim infrastructure |
| `Ctrl-x`    | Decrement object (number) under cursor                               | ✅ | Uses vim infrastructure |
| `Q`         | Start/stop macro recording to the selected register (experimental)   | ✅ | Uses vim infrastructure |
| `q`         | Play back a recorded macro from the selected register (experimental) | ✅ | Uses vim infrastructure |

#### Shell

| Key     | Description                                                                      | Status | Notes |
| ------  | -----------                                                                      | ------ | ----- |
| <code>&#124;</code>     | Pipe each selection through shell command, replacing with output                 | ❌ | Not implemented |
| <code>Alt-&#124;</code> | Pipe each selection into shell command, ignoring output                          | ❌ | Not implemented |
| `!`     | Run shell command, inserting output before each selection                        | ❌ | Not implemented |
| `Alt-!` | Run shell command, appending output after each selection                         | ❌ | Not implemented |
| `$`     | Pipe each selection into shell command, keep selections where command returned 0 | ❌ | Not implemented |

### Selection manipulation

| Key                      | Description                                                       | Status | Notes |
| -----                    | -----------                                                       | ------ | ----- |
| `s`                      | Select all regex matches inside selections                        | ✅ | Interactive prompt with real-time preview |
| `S`                      | Split selection into sub selections on regex matches              | ✅ | Interactive prompt with real-time preview |
| `Alt-s`                  | Split selection on newlines                                       | ❌ | Not implemented |
| `Alt-minus`              | Merge selections                                                  | ✅ | Full implementation with tests |
| `Alt-_`                  | Merge consecutive selections                                      | ✅ | Full implementation with tests |
| `&`                      | Align selection in columns                                        | ✅ | Full implementation with tests |
| `_`                      | Trim whitespace from the selection                                | ✅ | Full implementation with tests |
| `;`                      | Collapse selection onto a single cursor                           | ✅ | Full implementation with tests |
| `Alt-;`                  | Flip selection cursor and anchor                                  | ✅ | Full implementation with tests |
| `Alt-:`                  | Ensures the selection is in forward direction                     | ❌ | Not implemented |
| `,`                      | Keep only the primary selection                                   | ✅ | Full implementation with tests |
| `Alt-,`                  | Remove the primary selection                                      | ✅ | Full implementation with tests |
| `C`                      | Copy selection onto the next line (Add cursor below)              | ✅ | Full implementation with tests |
| `Alt-C`                  | Copy selection onto the previous line (Add cursor above)          | ✅ | Full implementation with tests |
| `(`                      | Rotate main selection backward                                    | ✅ | Full implementation with tests |
| `)`                      | Rotate main selection forward                                     | ✅ | Full implementation with tests |
| `Alt-(`                  | Rotate selection contents backward                                | ✅ | Full implementation with tests |
| `Alt-)`                  | Rotate selection contents forward                                 | ✅ | Full implementation with tests |
| `%`                      | Select entire file                                                | ✅ | Full implementation |
| `x`                      | Select current line, if already selected, extend to next line     | ✅ | Uses vim infrastructure |
| `X`                      | Extend selection to line bounds (line-wise selection)             | ❌ | Not implemented |
| `Alt-x`                  | Shrink selection to line bounds (line-wise selection)             | ❌ | Not implemented |
| `J`                      | Join lines inside selection                                       | ❌ | Not implemented |
| `Alt-J`                  | Join lines inside selection and select the inserted space         | ❌ | Not implemented |
| `K`                      | Keep selections matching the regex                                | ✅ | Interactive prompt with real-time preview |
| `Alt-K`                  | Remove selections matching the regex                              | ✅ | Interactive prompt with real-time preview |
| `Ctrl-c`                 | Comment/uncomment the selections                                  | ✅ | Uses vim infrastructure |
| `Alt-o`, `Alt-up`        | Expand selection to parent syntax node (**TS**)                   | ❌ | Not implemented |
| `Alt-i`, `Alt-down`      | Shrink syntax tree object selection (**TS**)                      | ❌ | Not implemented |
| `Alt-p`, `Alt-left`      | Select previous sibling node in syntax tree (**TS**)              | ❌ | Not implemented |
| `Alt-n`, `Alt-right`     | Select next sibling node in syntax tree (**TS**)                  | ❌ | Not implemented |
| `Alt-a`                  | Select all sibling nodes in syntax tree (**TS**)                  | ❌ | Not implemented |
| `Alt-I`, `Alt-Shift-down`| Select all children nodes in syntax tree (**TS**)                 | ❌ | Not implemented |
| `Alt-e`                  | Move to end of parent node in syntax tree (**TS**)                | ❌ | Not implemented |
| `Alt-b`                  | Move to start of parent node in syntax tree (**TS**)              | ❌ | Not implemented |

### Search

Search commands all operate on the `/` register by default. To use a different register, use `"<char>`.

| Key   | Description                                 | Status | Notes |
| ----- | -----------                                 | ------ | ----- |
| `/`   | Search for regex pattern                    | ✅ | Uses vim infrastructure |
| `?`   | Search for previous pattern                 | ✅ | Uses vim infrastructure |
| `n`   | Select next search match                    | ✅ | Uses vim infrastructure |
| `N`   | Select previous search match                | ✅ | Uses vim infrastructure |
| `*`   | Use current selection as the search pattern, automatically wrapping with `\b` on word boundaries | ❌ | Not implemented |
| `Alt-*` | Use current selection as the search pattern | ❌ | Not implemented |

### Minor modes

These sub-modes are accessible from normal mode and typically switch back to normal mode after a command.

| Key      | Description                                        | Status | Notes |
| -----    | -----------                                        | ------ | ----- |
| `v`      | Enter [select (extend) mode](#select--extend-mode) | ✅ | Full implementation |
| `g`      | Enter [goto mode](#goto-mode)                      | ❌ | Not implemented |
| `m`      | Enter [match mode](#match-mode)                    | ❌ | Not implemented |
| `:`      | Enter command mode                                 | ✅ | Uses vim infrastructure |
| `z`      | Enter [view mode](#view-mode)                      | ❌ | Not implemented |
| `Z`      | Enter sticky [view mode](#view-mode)               | ❌ | Not implemented |
| `Ctrl-w` | Enter [window mode](#window-mode)                  | ✅ | Uses vim infrastructure |
| `Space`  | Enter [space mode](#space-mode)                    | ❌ | Not implemented |

#### View mode

Accessed by typing `z` in [normal mode](#normal-mode).

| Key                  | Description                                               | Status | Notes |
| -----                | -----------                                               | ------ | ----- |
| `z`, `c`             | Vertically center the line                                | ❌ | Not implemented |
| `t`                  | Align the line to the top of the screen                   | ❌ | Not implemented |
| `b`                  | Align the line to the bottom of the screen                | ❌ | Not implemented |
| `m`                  | Align the line to the middle of the screen (horizontally) | ❌ | Not implemented |
| `j`, `down`          | Scroll the view downwards                                 | ❌ | Not implemented |
| `k`, `up`            | Scroll the view upwards                                   | ❌ | Not implemented |
| `Ctrl-f`, `PageDown` | Move page down                                            | ❌ | Not implemented |
| `Ctrl-b`, `PageUp`   | Move page up                                              | ❌ | Not implemented |
| `Ctrl-u`             | Move cursor and page half page up                         | ❌ | Not implemented |
| `Ctrl-d`             | Move cursor and page half page down                       | ❌ | Not implemented |

#### Goto mode

Accessed by typing `g` in [normal mode](#normal-mode).

| Key   | Description                                      | Status | Notes |
| ----- | -----------                                      | ------ | ----- |
| `g`   | Go to line number `<n>` else start of file       | ❌ | Not implemented |
| <code>&#124;</code>  | Go to column number `<n>` else start of line     | ❌ | Not implemented |
| `e`   | Go to the end of the file                        | ❌ | Not implemented |
| `f`   | Go to files in the selections                    | ❌ | Not implemented |
| `h`   | Go to the start of the line                      | ❌ | Not implemented |
| `l`   | Go to the end of the line                        | ❌ | Not implemented |
| `s`   | Go to first non-whitespace character of the line | ❌ | Not implemented |
| `t`   | Go to the top of the screen                      | ❌ | Not implemented |
| `c`   | Go to the middle of the screen                   | ❌ | Not implemented |
| `b`   | Go to the bottom of the screen                   | ❌ | Not implemented |
| `d`   | Go to definition (**LSP**)                       | ❌ | Not implemented |
| `y`   | Go to type definition (**LSP**)                  | ❌ | Not implemented |
| `r`   | Go to references (**LSP**)                       | ❌ | Not implemented |
| `i`   | Go to implementation (**LSP**)                   | ❌ | Not implemented |
| `a`   | Go to the last accessed/alternate file           | ❌ | Not implemented |
| `m`   | Go to the last modified/alternate file           | ❌ | Not implemented |
| `n`   | Go to next buffer                                | ❌ | Not implemented |
| `p`   | Go to previous buffer                            | ❌ | Not implemented |
| `.`   | Go to last modification in current file          | ❌ | Not implemented |
| `j`   | Move down textual (instead of visual) line       | ❌ | Not implemented |
| `k`   | Move up textual (instead of visual) line         | ❌ | Not implemented |
| `w`   | Show labels at each word and select the word that belongs to the entered labels | ❌ | Not implemented |

#### Match mode

Accessed by typing `m` in [normal mode](#normal-mode).

| Key              | Description                                     | Status | Notes |
| -----            | -----------                                     | ------ | ----- |
| `m`              | Goto matching bracket (**TS**)                  | ❌ | Not implemented |
| `s` `<char>`     | Surround current selection with `<char>`        | ❌ | Not implemented |
| `r` `<from><to>` | Replace surround character `<from>` with `<to>` | ❌ | Not implemented |
| `d` `<char>`     | Delete surround character `<char>`              | ❌ | Not implemented |
| `a` `<object>`   | Select around textobject                        | ❌ | Not implemented |
| `i` `<object>`   | Select inside textobject                        | ❌ | Not implemented |

#### Window mode

Accessed by typing `Ctrl-w` in [normal mode](#normal-mode).

| Key                    | Description                                          | Status | Notes |
| -----                  | -------------                                        | ------ | ----- |
| `w`, `Ctrl-w`          | Switch to next window                                | ✅ | Uses vim infrastructure |
| `v`, `Ctrl-v`          | Vertical right split                                 | ✅ | Uses vim infrastructure |
| `s`, `Ctrl-s`          | Horizontal bottom split                              | ✅ | Uses vim infrastructure |
| `f`                    | Go to files in the selections in horizontal splits   | ❌ | Not implemented |
| `F`                    | Go to files in the selections in vertical splits     | ❌ | Not implemented |
| `h`, `Ctrl-h`, `Left`  | Move to left split                                   | ✅ | Uses vim infrastructure |
| `j`, `Ctrl-j`, `Down`  | Move to split below                                  | ✅ | Uses vim infrastructure |
| `k`, `Ctrl-k`, `Up`    | Move to split above                                  | ✅ | Uses vim infrastructure |
| `l`, `Ctrl-l`, `Right` | Move to right split                                  | ✅ | Uses vim infrastructure |
| `q`, `Ctrl-q`          | Close current window                                 | ✅ | Uses vim infrastructure |
| `o`, `Ctrl-o`          | Only keep the current window, closing all the others | ✅ | Uses vim infrastructure |
| `H`                    | Swap window to the left                              | ❌ | Not implemented |
| `J`                    | Swap window downwards                                | ❌ | Not implemented |
| `K`                    | Swap window upwards                                  | ❌ | Not implemented |
| `L`                    | Swap window to the right                             | ❌ | Not implemented |

#### Space mode

Accessed by typing `Space` in [normal mode](#normal-mode).

| Key     | Description                                                             | Status | Notes |
| -----   | -----------                                                             | ------ | ----- |
| `f`     | Open file picker at LSP workspace root                                  | ❌ | Not implemented |
| `F`     | Open file picker at current working directory                           | ❌ | Not implemented |
| `b`     | Open buffer picker                                                      | ❌ | Not implemented |
| `j`     | Open jumplist picker                                                    | ❌ | Not implemented |
| `g`     | Open changed file picker                                                | ❌ | Not implemented |
| `G`     | Debug (experimental)                                                    | ❌ | Not implemented |
| `k`     | Show documentation for item under cursor in a [popup](#popup) (**LSP**) | ❌ | Not implemented |
| `s`     | Open document symbol picker (**LSP**)                                   | ❌ | Not implemented |
| `S`     | Open workspace symbol picker (**LSP**)                                  | ❌ | Not implemented |
| `d`     | Open document diagnostics picker (**LSP**)                              | ❌ | Not implemented |
| `D`     | Open workspace diagnostics picker (**LSP**)                             | ❌ | Not implemented |
| `r`     | Rename symbol (**LSP**)                                                 | ❌ | Not implemented |
| `a`     | Apply code action (**LSP**)                                             | ❌ | Not implemented |
| `h`     | Select symbol references (**LSP**)                                      | ❌ | Not implemented |
| `'`     | Open last fuzzy picker                                                  | ❌ | Not implemented |
| `w`     | Enter [window mode](#window-mode)                                       | ❌ | Not implemented |
| `c`     | Comment/uncomment selections                                            | ❌ | Not implemented |
| `C`     | Block comment/uncomment selections                                      | ❌ | Not implemented |
| `Alt-c` | Line comment/uncomment selections                                       | ❌ | Not implemented |
| `p`     | Paste system clipboard after selections                                 | ❌ | Not implemented |
| `P`     | Paste system clipboard before selections                                | ❌ | Not implemented |
| `y`     | Yank selections to clipboard                                            | ❌ | Not implemented |
| `Y`     | Yank main selection to clipboard                                        | ❌ | Not implemented |
| `R`     | Replace selections by clipboard contents                                | ❌ | Not implemented |
| `/`     | Global search in workspace folder                                       | ❌ | Not implemented |
| `?`     | Open command palette                                                    | ❌ | Not implemented |

#### Unimpaired

These mappings are in the style of [vim-unimpaired](https://github.com/tpope/vim-unimpaired).

| Key      | Description                                  | Status | Notes |
| -----    | -----------                                  | ------ | ----- |
| `]d`     | Go to next diagnostic (**LSP**)              | ❌ | Not implemented |
| `[d`     | Go to previous diagnostic (**LSP**)          | ❌ | Not implemented |
| `]D`     | Go to last diagnostic in document (**LSP**)  | ❌ | Not implemented |
| `[D`     | Go to first diagnostic in document (**LSP**) | ❌ | Not implemented |
| `]f`     | Go to next function (**TS**)                 | ❌ | Not implemented |
| `[f`     | Go to previous function (**TS**)             | ❌ | Not implemented |
| `]t`     | Go to next type definition (**TS**)          | ❌ | Not implemented |
| `[t`     | Go to previous type definition (**TS**)      | ❌ | Not implemented |
| `]a`     | Go to next argument/parameter (**TS**)       | ❌ | Not implemented |
| `[a`     | Go to previous argument/parameter (**TS**)   | ❌ | Not implemented |
| `]c`     | Go to next comment (**TS**)                  | ❌ | Not implemented |
| `[c`     | Go to previous comment (**TS**)              | ❌ | Not implemented |
| `]T`     | Go to next test (**TS**)                     | ❌ | Not implemented |
| `[T`     | Go to previous test (**TS**)                 | ❌ | Not implemented |
| `]p`     | Go to next paragraph                         | ❌ | Not implemented |
| `[p`     | Go to previous paragraph                     | ❌ | Not implemented |
| `]g`     | Go to next change                            | ❌ | Not implemented |
| `[g`     | Go to previous change                        | ❌ | Not implemented |
| `]Space` | Add newline below                            | ❌ | Not implemented |
| `[Space` | Add newline above                            | ❌ | Not implemented |

## Insert mode

Accessed by typing `i` in [normal mode](#normal-mode).

| Key                                         | Description                 | Status | Notes |
| -----                                       | -----------                 | ------ | ----- |
| `Escape`                                    | Switch to normal mode       | ✅ | Uses vim infrastructure |
| `Ctrl-s`                                    | Commit undo checkpoint      | ❌ | Not implemented |
| `Ctrl-x`                                    | Autocomplete                | ✅ | Uses vim infrastructure |
| `Ctrl-r`                                    | Insert a register content   | ❌ | Not implemented |
| `Ctrl-w`, `Alt-Backspace`                   | Delete previous word        | ✅ | Uses vim infrastructure |
| `Alt-d`, `Alt-Delete`                       | Delete next word            | ✅ | Uses vim infrastructure |
| `Ctrl-u`                                    | Delete to start of line     | ✅ | Uses vim infrastructure |
| `Ctrl-k`                                    | Delete to end of line       | ✅ | Uses vim infrastructure |
| `Ctrl-h`, `Backspace`, `Shift-Backspace`    | Delete previous char        | ✅ | Uses vim infrastructure |
| `Ctrl-d`, `Delete`                          | Delete next char            | ✅ | Uses vim infrastructure |
| `Ctrl-j`, `Enter`                           | Insert new line             | ✅ | Uses vim infrastructure |

## Select / extend mode

Accessed by typing `v` in [normal mode](#normal-mode).

| Key | Description | Status | Notes |
| --- | ----------- | ------ | ----- |
| All movement keys | Extend selection instead of replacing | ✅ | Full implementation with tests |
| `v` | Exit select mode | ✅ | Full implementation |
| `Escape` | Exit select mode | ✅ | Full implementation |

## Implementation Summary

### ✅ Fully Implemented (Core Functionality)
- **Basic Movement**: h, j, k, l, arrow keys
- **Word Movement**: w, e, b, W, E, B with proper punctuation handling
- **Find Character**: f, F, t, T with precise positioning
- **Selection Operations**: 
  - Collapse (`;`), flip (`Alt-;`), merge (`Alt--`, `Alt-_`)
  - Trim (`_`), align (`&`)
  - Copy to next/prev line (`C`, `Alt-C`)
  - Keep/remove primary (`,`, `Alt-,`)
  - Rotate selections (`(`, `)`) and contents (`Alt-(`, `Alt-)`)
- **Regex Selection Operations**:
  - Select regex matches (`s`) with interactive prompt and real-time preview ✅
  - Split selections on regex (`S`) with interactive prompt and real-time preview ✅
  - Keep selections matching regex (`K`) with interactive prompt, real-time preview, and exact Helix behavior ✅
  - Remove selections matching regex (`Alt-K`) with interactive prompt, real-time preview, and exact Helix behavior ✅
  - **Interactive UI Features**:
    - Real-time preview updates as user types regex pattern ✅
    - Enter key confirms selection and closes modal ✅
    - Escape key cancels operation and restores original selections ✅
    - Graceful handling of invalid regex patterns ✅
    - Empty pattern handling ✅
    - Comprehensive UI integration tests ✅
- **Mode System**: Normal and Select modes with proper switching
- **Line Selection**: x for line selection
- **Basic Editing**: Insert modes, undo/redo, yank/paste, delete/change
- **Window Management**: Basic window operations via Ctrl-w

### 🚧 Partially Implemented
- **Select All**: % command implemented

### ❌ Major Missing Features
- **Minor Mode Systems**: g (goto), m (match), z (view), Space modes
- **Text Objects**: mi, ma commands for inside/around objects
- **Advanced Selection**: Syntax tree operations, shell pipe operations
- **Search Integration**: *, Alt-* for selection-based search
- **Case Operations**: ~, `, Alt-` for case changes
- **Advanced Editing**: R (replace with yanked), Alt-d/Alt-c (no-yank operations)
- **History Navigation**: Alt-u, Alt-U for history
- **Line Operations**: J (join), X/Alt-x (line bounds)
- **Repeat Operations**: Alt-. for motion repeat
- **Register Operations**: Ctrl-r in insert mode
- **Advanced Window**: Window swapping (H, J, K, L)

### 📝 Next Priority Implementation Order

1. **Text Objects and Match Mode** (`mi`, `ma`, `mm`)
2. **Goto Mode** (`g` prefix commands)
3. **Case Operations** (`~`, `` ` ``, `` Alt-` ``)
4. **Advanced Selection Operations** (syntax tree, shell pipe)
5. **Space Mode** (file pickers, LSP operations)
6. **View Mode** (`z` prefix commands)
7. **Search Integration** (`*`, `Alt-*`)
8. **Advanced Editing Operations**

### Test Coverage Status

- ✅ **Movement Tests**: 8+ tests covering all basic and word movements
- ✅ **Selection Tests**: 31+ tests covering all selection operations
- ✅ **Integration Tests**: Keystroke simulation and workflow tests
- ❌ **Minor Mode Tests**: Not yet implemented
- ❌ **Text Object Tests**: Not yet implemented

---

## Notes

- **Vim Infrastructure Reuse**: Many basic operations leverage existing vim infrastructure in Zed
- **Helix-Specific Implementation**: Word movements, find characters, and selection operations have full Helix-specific implementations
- **Primary Selection Tracking**: Implemented global primary selection index tracking for rotate operations
- **Mode System**: Proper Helix Normal/Select mode distinction implemented
- **Test Coverage**: Comprehensive test suite for implemented features
- **Manual Verification**: All implemented features verified to work correctly in practice

This tracking document will be updated as new features are implemented. 

### 🔧 **Recent Fixes Applied**
- **Fixed Interactive Prompts**: Enter/Escape keys now work correctly to confirm/cancel regex dialogs
- **Fixed Keep/Remove Behavior**: Now uses partial matches within selections (e.g., keep "o" on selections "one", "two", "three" keeps "one" and "two")
- **Fixed Key Context**: Updated from "InteractiveRegexPrompt" to "RegexPrompt" to match keymap
- **Enhanced User Experience**: Added helpful regex tips in the prompt dialog 