# 3. Keys

NC/VC keys first; where Far added one that people now expect, it is here
too, marked (Far). A key in two programs with two meanings is a decision
in [09-decisions.md](09-decisions.md).

## 3.1 Function keys in the panels

| Key | Plain | Shift | Ctrl | Alt |
|---|---|---|---|---|
| F1 | Help | | Left panel on/off | Left drive menu |
| F2 | User menu | Main user menu | Right panel on/off | Right drive menu |
| F3 | View (a folder: count its size) | View a typed name | Sort by name | View with the other viewer |
| F4 | Edit | Edit a typed name, new or not | Sort by extension | Edit with the other editor |
| F5 | Copy | Copy the file under the cursor, in place (a copy with another name) | Sort by time | |
| F6 | Rename or move | Rename the file under the cursor, in place | Sort by size | Count folder sizes |
| F7 | Make folder | | Unsorted | Find file |
| F8 | Delete | Delete the file under the cursor, not the selection | | Command history |
| F9 | Pull-down menu | Save setup | | 25/50 lines (DOS) |
| F10 | Quit | Pull-down menu | | Folder tree |

F10 asks "Do you want to quit Kirin Commander?" Yes / No - an option, on.

## 3.2 Panels

| Key | Does |
|---|---|
| Tab | The other panel |
| Enter | A folder: into it. `..`: out. A program (`.COM .EXE .BAT` on DOS, executable elsewhere): run it. Another file: what its extension says (section 3.5), else nothing |
| Ctrl+PgDn | Into the folder under the cursor |
| Ctrl+PgUp | Out to the parent (in a root: the drive menu) |
| Ctrl+\\ | To the root |
| Backspace | Out to the parent, while the command line is empty (Far) |
| Up Down Left Right PgUp PgDn Home End | The cursor (rule 8 in [02-panels.md](02-panels.md)) |
| Insert | Select / unselect, and down |
| Shift+arrows | Select along the way (Far) |
| Gray + / Gray − / Gray \* | Select by mask / unselect by mask / invert |
| Shift+Gray + / Shift+Gray − | Select all / unselect all (Far) |
| Ctrl+M | Restore the last selection |
| Alt+letters | Quick search |
| Ctrl+1 .. Ctrl+5 | View mode (Brief, Medium, Full, Wide, Detailed) |
| Ctrl+O | Panels off and on: the terminal behind them |
| Ctrl+P | The inactive panel off and on |
| Ctrl+U | Swap the panels |
| Ctrl+L | Info panel in the other panel |
| Ctrl+Q | Quick view in the other panel |
| Ctrl+T | Tree in this panel (Far) |
| Ctrl+R | Re-read |
| Ctrl+H | Hidden files shown / not |
| Ctrl+F | Filter |
| Ctrl+A | File attributes |
| Ctrl+B | Key bar on/off |
| Ctrl+Up / Ctrl+Down | Panels taller / shorter (Far) |
| Ctrl+Left / Ctrl+Right | Move the division between the panels, while the command line is empty (Far) |
| Ctrl+Shift+0..9 | Set a folder shortcut (Far); F9, Commands, Folder shortcuts goes to one - a right Ctrl cannot be told from a left one on DOS |
| Alt+Shift+F9 or F9 → Options | Configuration |

## 3.3 The command line

Typing goes to the command line while the panels are shown (section 2.7
for letters with Alt). Enter runs it.

| Key | Does |
|---|---|
| Ctrl+Enter | The name under the cursor onto the line (NC, Far) |
| Ctrl+Shift+Enter | The name under the other panel's cursor |
| Ctrl+[ / Ctrl+] | The left / right panel's path onto the line |
| Ctrl+F | The full path of the name under the cursor (Far) - Ctrl+F is the filter while the line is empty |
| Ctrl+E / Ctrl+X | The previous / next command from the history |
| Alt+F8 | The history as a list |
| Esc | Clear the line; with it empty, nothing (an option: panels off/on, as VC could) |
| Ctrl+Y | Clear the line (Far) |
| Home End Left Right, Ctrl+Left / Ctrl+Right | Move in the line - Left and Right only once the line has text; in Brief they move the panel cursor (Far's rule) |
| Ctrl+K | Delete to the end of the line |
| Ctrl+Ins / Shift+Ins / Ctrl+V | Copy the line / paste into it. With the line empty, Ctrl+Ins copies the selected names |

## 3.4 In every dialog and menu

| Key | Does |
|---|---|
| Tab / Shift+Tab | Next / previous control |
| Enter | The default button |
| Esc | Cancel |
| Alt+letter, or the letter in a menu | The control or item with that hotkey |
| Up / Down in an input line | Its history (Ctrl+Up/Down in Far; plain Down is OWLOSUI's) |
| F10 in a path field | A folder tree to choose the path from |

These are OWLOSUI's own; Kirin Commander adds nothing to them.

## 3.5 Enter on a file

What a file opens with is decided by its extension, from a table the user
edits (F9 → Commands → Extension file edit), as NC's `NC.EXT`:

```
asm: edit !.!
txt: view !.!
zip: unzip -l !.! | more
```

Without an entry: a program runs, anything else does nothing (NC) - an
option makes it view. The table's language is NC's: `!` the name, `!.!`
name and extension, `!:` the drive, `!\` the path.
