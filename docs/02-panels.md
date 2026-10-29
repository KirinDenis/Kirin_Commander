# 2. A panel

## 2.1 View modes

| Mode | Key | Columns |
|---|---|---|
| Brief | Ctrl+1 | Names only, **three** columns in a 40-column panel (a wider panel: as many as fit at the name width) |
| Medium | Ctrl+2 | Names only, two columns - for long names |
| Full | Ctrl+3 | Name, Size, Date, Time |
| Wide | Ctrl+4 | Name and Size - for long names |
| Detailed | Ctrl+5 | Name, Size, Date, Time, Attributes - the whole screen width |

Brief and Full are the classic pair, and Brief is the default on DOS: an
8.3 name is twelve cells, and three of them fit in a 40-column panel. The
mode is the panel's own: the left panel can be Brief while the right is Full.

## 2.2 How a name is shown

**On DOS (8.3 names):**

- The name and the extension in two sub-columns, the **dot not shown**:
  `README   TXT`, `CITY     COM`. Eight cells, a space, three cells.
- Folders in **upper case**, files in **lower case** (an option, on by
  default, as NC and VC drew them). The name on the disk is not changed.
- A hidden or system file: `░` in place of the space between name and
  extension.

**On a desktop and in a browser (long names):**

- The name as it is, with its case. The extension is not split off,
  except in Brief and Full where an option aligns extensions in a column
  (Far's "Align file extensions").
- A name longer than its column: cut, `}` in the last cell. The mini
  status shows it whole, and Alt+Left / Alt+Right scroll the names sideways.

**Everywhere:**

- `..` first, then folders, then files - each group in the sort order.
- A root folder has no `..`, as in NC and VC: there is nothing above it,
  and Backspace or Ctrl+PgUp there does nothing. Another drive is Alt+F1 /
  Alt+F2.
- The size column: `►UP--DIR◄` for `..`, `►SUB-DIR◄` for a folder whose
  size has not been counted (Alt+F6 counts it, then the number is shown).
- Sizes with a space between thousands: `1 234 567`. A size that does not
  fit is shown in K, then M, then G.
- Dates `dd-mm-yy` and times `hh:mm`, 24-hour - an option gives the
  system's own format.

## 2.3 Sorting

Ctrl+F3 name, Ctrl+F4 extension, Ctrl+F5 time, Ctrl+F6 size, Ctrl+F7
unsorted (the order on the disk). The same key again reverses the order.
The letter of the current order is in the panel's top left corner (`n`,
`e`, `t`, `s`, `u`; upper case when reversed). Folders are always before
files, and sorted the same way among themselves. Names compare without
case, and with numbers as numbers on a desktop (`file2` before `file10`)
- an option; off on DOS, where it is not what anyone expects.

## 2.4 The cursor

The rules people notice at once when one is broken.

1. **Each panel has its own cursor, its own selection, its own folder and
   its own mode.** Tab moves the keys to the other panel and changes none
   of them. The cursor bar is drawn only in the active panel; the other
   panel keeps a quiet mark of where its cursor is (the name in the
   cursor colour's foreground, no bar).
2. **Into a folder** (Enter, Ctrl+PgDn): the cursor is on `..`, at the
   top.
3. **Out of a folder** (Enter on `..`, Ctrl+PgUp, Backspace on an empty
   command line): the cursor is **on the folder just left**, in the
   middle of the panel if it can be, so the way back is one Enter.
4. **Ctrl+\\** to the root: the cursor on the first-level folder that led
   to where the panel was.
5. **A drive changed** (Alt+F1/F2): the panel shows that drive's current
   folder, as DOS remembers one per drive, cursor on `..` or the first name.
6. **Re-read** (Ctrl+R, and after any operation): the cursor stays on the
   same name. If that name is gone - deleted, moved away - it goes to the
   **next** name, or the previous one if it was the last. Never back to
   the top.
7. **After an operation**:
   - Copy and Move: the source panel keeps its cursor (rule 6); the
     target panel is re-read and its cursor stays where it was.
   - Make folder (F7): the cursor **on the new folder**.
   - Rename in place (Shift+F6): the cursor on the new name.
   - Delete: rule 6, the next name.
   - A file created by the editor (Shift+F4): the cursor on it.
8. **Multi-column movement** (Brief, Medium): Up and Down move within a
   column and on to the next column at its end; Left and Right move a
   whole column, and at the first or last column to the first or last
   name; PgUp/PgDn move a whole panel of names; Home and End go to the
   first (`..`) and the last name.
9. **Scrolling** keeps the cursor where it is on the screen when it can:
   the names move, the bar stays.
10. **Mouse**: a click puts the cursor on a name; a double click is Enter;
    the right button selects (Far's habit) - an option, since a right click
    is the context menu elsewhere in OWLOSUI. The wheel moves the cursor.
11. **Touch**: a tap is the cursor, a double tap is Enter, a hold selects,
    a drag scrolls - OWLOSUI's touch rules.

## 2.5 Selection

- **Insert** selects or unselects the file under the cursor and moves the
  cursor down (an option, on). Shift+arrows select along the way (Far).
- **Gray +** opens "Select" with a mask; **Gray −** "Unselect"; **Gray \***
  inverts the selection of files (Ctrl+Gray \* includes folders).
- **Shift+Gray +** / **Shift+Gray −** select / unselect all.
- **Ctrl+M** brings back the selection from before the last operation.
- `..` is never selected. Folders are selected by Insert; by a mask only
  when "Select folders" is on (off by default).
- The selection belongs to the panel and is kept while the panel's folder
  is the same; changing folder clears it.
- A file operation acts on **the selection, if there is one, else on the
  file under the cursor**, and says which in its dialog ("3 files" or
  `"readme.txt"`).
- After a successful copy, move or delete the selection is cleared; after
  one that failed partly, **only the files that were done** are
  unselected, so what is left selected is what still needs doing.

## 2.6 Masks

A mask is what Select, Unselect, Find file, the filter (Ctrl+F) and the
copy target understand:

- `*` any characters, `?` one character, as DOS has them.
- On DOS, `NAME.EXT` rules: `*.*` matches every name including one with
  no extension; `*` alone matches every name.
- Several masks separated by `,` or `;`: `*.asm,*.inc`.
- An exclusion after `|`: `*.*|*.bak,*.tmp` - everything but backups.
- A mask with no `*` or `?` matches as if `*` were added: `read` is
  `read*`.
- Case is ignored, on every system - Linux included, where it is the
  person's expectation in a file manager, not the system's.
- Masks are remembered (history, Up in the line).

## 2.7 Quick search

Hold **Alt** and type: a small box `Search:` opens at the bottom of the
panel, and the cursor jumps to the first name that starts with what is
typed. `*` and `?` work in it. Ctrl+Enter goes to the next match;
Backspace takes a letter back; Esc, Enter, or any key that is not a
letter closes the box and leaves the cursor where it is. While the
command line is empty, typing a letter without Alt can be set to search
too (an option, off: without Alt, letters go to the command line).

## 2.8 The drive menu (Alt+F1, Alt+F2)

- A small menu over the panel it is for (Alt+F1 the left, Alt+F2 the
  right), titled "Choose left drive" / "Choose right drive".
- **On DOS**: one line per drive DOS has - the letter, the drive's size,
  its free bytes and how much of it is free (D14). NC and VC had only the
  letters in a row; the numbers are what NC kept in its Info panel. Only
  a fixed disk is asked its space (INT 21h 36h), once, as the menu
  opens: asking a floppy drive spins it, and with no disk in it stops at
  "Abort, Retry, Fail". A floppy drive says `removable`; a network drive
  or a CD-ROM, which DOS reaches through a redirector, says `remote`.
- **On a desktop and in a browser**, where a drive is more than a letter,
  as Far: one line per drive, the letter and what it is - `C:  fixed
  SYSTEM  1.2 G free`, a network path, a browser storage name.
- **The cursor starts on the drive the panel is on now.**

```text
╔════════════╤ C:\LESSONS ╤════════════╗╔═══════════ C:\LESSONS\L07 ════╤═12:34╗
║    Name    │    Name    │    Name    ║║    Name    │  Size   │  Date  │ Time ║
║..          │L18         │readme   txt║║..          │►UP--DIR◄│28-10-26│ 09:12║
║L01         │L19         │tools    txt║║build    bat│      156│24-10-26│ 15:34║
║L02         │L20         │            ║║build    txt│      211│28-10-26│ 09:15║
║L03         │L21         │            ║║go       bat│       38│24-10-26│ 15:34║
║L04         │L22         │            ║║hello    asm│    2 345│28-10-26│ 09:14║
║L0╔══ Choose left drive [modal] ══╗   ║║hello    com│      412│28-10-26│ 09:15║
║L0║                               ║   ║║            │         │        │      ║
║L0║         Size     Free         ║   ║║            │         │        │      ║
║L0║  A:  removable                ║   ║║            │         │        │      ║
║L0║  B:  removable                ║   ║║            │         │        │      ║
║L1║  C:  2 048 M  2 047 M  99%    ║   ║║            │         │        │      ║
║L1║  D:    512 M    512 M 100%    ║   ║║            │         │        │      ║
║L1║  E:  4 096 K  3 072 K  75%    ║   ║║            │         │        │      ║
║L1║          ► OK ◄▄   Cancel  ▄  ║   ║║            │         │        │      ║
║L1║           ▀▀▀▀▀▀  ▀▀▀▀▀▀▀▀▀▀  ║   ║║            │         │        │      ║
║L1╚═══════════════════════════════╝   ║║            │         │        │      ║
║L16         │lessons  ini│            ║║            │         │        │      ║
║L17         │make     bat│            ║║            │         │        │      ║
╟────────────┴────────────┴────────────╢╟────────────┴─────────┴────────┴──────╢
║   3 914 bytes in 2 selected files    ║║hello    asm     2 345 28-10-26 09:14 ║
╚══════════════════════════════════════╝╚══════════════════════════════════════╝
C:\LESSONS>

```

*Lit:* OWLOSUI's grey dialog with a list and OK, Cancel (D15); `C:`'s
line - the drive the left panel is on - under the list's cursor, black
on cyan. Each letter is its drive's hotkey. While the dialog is open the status line is
the dialog's, empty: the panels' keys do nothing until it closes.
- A letter key chooses that drive at once; Enter the one under the
  cursor; Up and Down (and Left, Right) move it; Esc or Cancel closes. With more drives than the
  panel has rows, the list scrolls with the cursor.
- A drive that is not ready (no disk in A:) is chosen anyway, and the
  panel says so in its frame - "Drive A: is not ready" with Retry and
  Cancel, as DOS asks - rather than the menu refusing.
- On a desktop the menu also lists the home folder, the places from the
  folder shortcuts (Ctrl+Shift+0..9) and mounted volumes; in a browser,
  what the page has mounted (see [07-command-line.md](07-command-line.md)).

## 2.9 Filters and hidden files

- Ctrl+H shows or hides hidden and system files (shown by default on DOS,
  as NC did; hidden on Linux dot-files, as `ls` does).
- Ctrl+F (Left/Right menu: Filter) sets a mask the panel shows - `*.asm`
  - and the panel's frame says `[filter]` while one is on.
