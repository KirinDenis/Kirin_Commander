# 1. The screen

The reference screen is 80 by 25, as Norton Commander and Volkov Commander
drew it. A bigger screen - a window, a browser - gives the same picture with
more rows and columns, never a different one. A phone is a small screen
(see [09-decisions.md](09-decisions.md), D10).

## 1.1 What is where

The screens in these documents are the real thing, 80 by 25, cell by
cell. Text cannot carry colour, so under each one a line says what is lit:
the cursor bar, the selected names, the active panel's title.

```text
╔════════════╤ C:\LESSONS ╤════════════╗╔═══════════ C:\LESSONS\L07 ════╤═12:34╗
║    Name    │    Name    │    Name    ║║    Name    │  Size   │  Date  │ Time ║
║..          │L18         │readme   txt║║..          │►UP--DIR◄│28-10-26│ 09:12║
║L01         │L19         │tools    txt║║build    bat│      156│24-10-26│ 15:34║
║L02         │L20         │            ║║build    txt│      211│28-10-26│ 09:15║
║L03         │L21         │            ║║go       bat│       38│24-10-26│ 15:34║
║L04         │L22         │            ║║hello    asm│    2 345│28-10-26│ 09:14║
║L05         │L23         │            ║║hello    com│      412│28-10-26│ 09:15║
║L06         │L24         │            ║║            │         │        │      ║
║L07         │L25         │            ║║            │         │        │      ║
║L08         │L26         │            ║║            │         │        │      ║
║L09         │L27         │            ║║            │         │        │      ║
║L10         │L28         │            ║║            │         │        │      ║
║L11         │L29         │            ║║            │         │        │      ║
║L12         │L30         │            ║║            │         │        │      ║
║L13         │L31         │            ║║            │         │        │      ║
║L14         │course   txt│            ║║            │         │        │      ║
║L15         │go       bat│            ║║            │         │        │      ║
║L16         │lessons  ini│            ║║            │         │        │      ║
║L17         │make     bat│            ║║            │         │        │      ║
╟────────────┴────────────┴────────────╢╟────────────┴─────────┴────────┴──────╢
║   3 914 bytes in 2 selected files    ║║hello    asm     2 345 28-10-26 09:14 ║
╚══════════════════════════════════════╝╚══════════════════════════════════════╝
C:\LESSONS>
F1 Help F2 Menu F3 View F4 Edit F5 Copy F6 RenMvF7 MkdirF8 DeletF9 Pull F10 Quit
```

*Lit:* the left panel is active - its title `C:\LESSONS` black on cyan,
the cursor bar on `L07` (the panel just came out of it). `go.bat` and
`readme.txt` are selected (yellow), so its mini status counts them. The
right panel is not active: no bar; its mini status shows `hello.asm`,
where its cursor stays. Names are in their colours (1.5): the folders white,
in the right panel `build.bat` and `go.bat` light green, the rest light cyan.

| Line | What |
|---|---|
| 1 | Both panels' top edges; their titles; the clock in the top right corner. The menu bar takes this line while it is open ([04-menus.md](04-menus.md)) |
| 2 | Column titles: `Name` three times in Brief; `Name Size Date Time` in Full |
| 3-20 | Names: in Brief flowing down three columns of twelve cells (eight, a space, three - no dot); in Full one per row |
| 21 | The line over the mini status, the column rules meeting it |
| 22 | The mini status: the item under the cursor, or what is selected |
| 23 | The panels' bottom edges |
| 24 | The command line, the active panel's folder as its prompt |
| 25 | The key bar |

- **Two panels**, the left one 40 columns wide and the right one 40, each
  framed with a double line. They are **one window**, not two: they are not
  moved or sized one by one with the mouse, and there is no gap, title bar
  or shadow between them. This is the commander's screen, not a desktop
  with two windows on it.
- **The panel's title** is in the middle of its top edge: the folder's path,
  shortened from the left with `...` when it does not fit. The active
  panel's title is lit.
- **Column titles** under the top edge: `Name` (three times in Brief),
  `Name Size Date Time` in Full. They can be turned off.
- **The mini status** is the panel's last line inside the frame, under a
  single-line separator. It says what the cursor is on: name, size (or
  `►SUB-DIR◄` / `►UP--DIR◄`), date, time. While files are selected it says
  instead how many and how big: `12 345 bytes in 3 selected files`.
- **The command line** is the line under the panels: the current folder
  as a prompt, `C:\GAMES>`, and a caret. Whatever is typed goes there while
  the panels are on the screen - see [07-command-line.md](07-command-line.md).
- **The key bar** is the last line, OWLOSUI's status line (D15): ten
  labels, each its key (`F1`..`F10`) and a word of at most six letters. While Shift, Ctrl or Alt is held it shows what the
  keys do with it (section 1.3). Ctrl+B hides and shows it.
- **The menu bar** is the first line, OWLOSUI's (D15), hidden until F9, a click on the
  first line, or (an option) Alt pressed alone. "Menu bar always visible"
  is an option. While hidden, the first line belongs to the panels.
- **The clock** is in the top right corner, an option, on by default.

Ctrl+O takes both panels away and shows what is behind them - the
terminal, with the command line at its bottom and the key bar under it.
See [07-command-line.md](07-command-line.md).

```text
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
C:\GAMES>dir *.bat                                                              
                                                                                
 Volume in drive C is WIRECITY                                                  
 Directory of C:\GAMES                                                          
                                                                                
COMMANDR BAT       112 18-10-26  10:02                                          
GO       BAT        38 24-10-26  15:34                                          
        2 file(s)        150 bytes                                              
                   1 234 567 bytes free                                         
                                                                                
C:\GAMES>_                                                                      
F1 Help F2 Menu F3 View F4 Edit F5 Copy F6 RenMvF7 MkdirF8 DeletF9 Pull F10 Quit
```

## 1.2 Panels that are not file lists

Either panel can show, instead of files:

| Mode | Key | What it shows |
|---|---|---|
| Info | Ctrl+L | The drive (label, total, free), the folder (files and bytes), memory on DOS, the folder's description file |
| Tree | Alt+F10 in a panel, or the menu | The drive's folders as a tree; the other panel follows the tree's cursor only on Enter |
| Quick view | Ctrl+Q | The file under the other panel's cursor, as the viewer shows it |

## 1.3 The key bar

The bar is OWLOSUI's status line: grey, each label the key, a space, the
word, the key red - as OWLOSUI lights a key on its status line. The ten
labels stand in ten even cells, a tenth of the width each, with no gap
between them, as the file managers' key bars had them: on eighty columns
a cell is eight, `F6 ` leaves five letters, and a longer word is
shortened (`RenMv`, `Delet`, `Pull`, `Ext`, `Unsrt`, `DirSz`, `Hist`).
The table gives the whole words, which a wider screen shows; a
narrower one drops the F (`1 Help`), and a phone's the space too
(`1Help`), so the words keep their letters. A label is
clicked as its key is pressed.

| Held | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 |
|---|---|---|---|---|---|---|---|---|---|---|
| nothing | Help | Menu | View | Edit | Copy | RenMov | Mkdir | Delete | PullDn | Quit |
| Shift | | Main | View.. | Edit.. | Copy | RenMov | | Delete | SavSet | PullDn |
| Ctrl | Left | Right | Name | Extens | Time | Size | Unsort | | | |
| Alt | Left | Right | View.. | Edit.. | | DirSiz | Find | Histry | Lines | Tree |

- `F2 Menu` is the user menu, not the pull-down one; `F9 Pull` is the pull-down.
- Ctrl+F3..F7 sort (Name, Extension, Time, Size, Unsorted), as NC and Far
  do. Volkov Commander put labels and copying on Ctrl+F4..F8 instead; this
  is decision D3 in [09-decisions.md](09-decisions.md).
- A key that does nothing in the current state has its label empty and
  grey.
- On a touch screen, and with a mouse, a label is clicked like the key.

## 1.4 Frames, separators, marks

- Panel frames: double line `╔═╗║╚╝`. The column separators are single
  `│`, met by the separator line `╟─╢` above the mini status.
- Dialogs, the drive menu and the menus are OWLOSUI's (D15): its grey
  windows with `[modal]` after the title and its buttons, its grey
  menus with a single frame, each with OWLOSUI's shadow. The panels
  under a shadow keep their characters, darkened.
- A hidden or system file: in Brief and Full, a `░` in place of the space
  between the name and the extension.
- A name too long for its column (a long name on a desktop or in a
  browser): cut, with `}` in the last cell, as Far marks it.

## 1.5 Colours

The default scheme is the one every user of NC and VC has seen. The table
gives foreground on background in the 16 IBM PC colours; the hex is the
attribute byte, from the Volkov Commander 4.05 source.

| Element | Colour | Attr |
|---|---|---|
| Panel: frame and normal file | light cyan on blue | 1Bh |
| Panel: column titles | yellow on blue | 1Eh |
| Panel: selected file | yellow on blue | 1Eh |
| Panel: cursor bar | black on cyan | 30h |
| Panel: cursor on a selected file | yellow on cyan | 3Eh |
| Panel: active title | black on cyan | 30h |
| Menu bar, menus (OWLOSUI's, D15) | black on light grey | 70h |
| Menu: hotkey letter | red on light grey | 74h |
| Menu: an item that cannot be done now | dark grey on light grey | 78h |
| Menu: current title, current item | black on green | 20h |
| Menu: their hotkey | red on green | 24h |
| Dialogs, errors among them, the progress window | OWLOSUI's grey window (D15), its buttons, input lines, check boxes and bars in its palette (OWLOSUI lib/core/src/palette.rs) | |
| Key bar (OWLOSUI's status line, D15) | black on light grey | 70h |
| Key bar: the key (`F1`) | red on light grey | 74h |
| Clock | black on cyan | 30h |
| Command line | light grey on black | 07h |
| Viewer and editor text | light cyan on blue | 1Bh |

Two more schemes, as Configuration offers them in NC and VC:

- **Black & White** - for a monochrome adapter: 07h normal, 70h inverse,
  0Fh bright. The cursor bar is inverse, selected files bright.
- **Laptop** - for a screen that shows colour poorly: as Black & White,
  and selected files also marked with a character (`√`) before the name,
  since brightness alone does not show on such a screen.

Names are coloured by what they are, as OWLOSUI's file panels colour them
(and Far, in this order: hidden and system first, so a hidden folder reads
as hidden):

| Name | Colour | Attr |
|---|---|---|
| Hidden or system | dark grey on blue | 18h |
| Folder | white on blue | 1Fh |
| Program: `.COM .EXE .BAT .BTM .CMD` | light green on blue | 1Ah |
| Archive: `.ZIP .RAR .ARJ .LZH .LHA .7Z .GZ .TAR` | light magenta on blue | 1Dh |
| Scratch: `.TMP .BAK .OLD .$$$ .SYD .CHK` | brown on blue | 16h |
| Anything else | light cyan on blue | 1Bh |

Selected names are yellow and the cursor bar black on cyan whatever the
name is. In Black & White folders and programs are bright, the rest normal.
