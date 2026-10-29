# 4. The pull-down menu (F9)

Five titles on the first line, as NC 3, VC and Far have them. F9 with the
left panel active:

```text
  Left  Files  Commands  Options  Right
 ┌──────────────────────┐ │    Name    ║║    Name    │  Size   │  Date  │ Time ║
 │√Brief         Ctrl+1 │ │readme   txt║║..          │►UP--DIR◄│28-10-26│ 09:12║
 │ Medium        Ctrl+2 │ │tools    txt║║build    bat│      156│24-10-26│ 15:34║
 │ Full          Ctrl+3 │ │            ║║build    txt│      211│28-10-26│ 09:15║
 │ Wide          Ctrl+4 │ │            ║║go       bat│       38│24-10-26│ 15:34║
 │ Detailed      Ctrl+5 │ │            ║║hello    asm│    2 345│28-10-26│ 09:14║
 ├──────────────────────┤ │            ║║hello    com│      412│28-10-26│ 09:15║
 │ Info panel    Ctrl+L │ │            ║║            │         │        │      ║
 │ Tree panel    Ctrl+T │ │            ║║            │         │        │      ║
 │ Quick view    Ctrl+Q │ │            ║║            │         │        │      ║
 │ On/off       Ctrl+F1 │ │            ║║            │         │        │      ║
 ├──────────────────────┤ │            ║║            │         │        │      ║
 │√Name         Ctrl+F3 │ │            ║║            │         │        │      ║
 │ eXtension    Ctrl+F4 │ │            ║║            │         │        │      ║
 │ tiMe         Ctrl+F5 │ │            ║║            │         │        │      ║
 │ Size         Ctrl+F6 │ │            ║║            │         │        │      ║
 │ Unsorted     Ctrl+F7 │ │            ║║            │         │        │      ║
 ├──────────────────────┤ │            ║║            │         │        │      ║
 │ Re-read       Ctrl+R │ │            ║║            │         │        │      ║
 │ fiLter...     Ctrl+F │ ┴────────────╢╟────────────┴─────────┴────────┴──────╢
 │ Hidden files  Ctrl+H │ ted files    ║║hello    asm     2 345 28-10-26 09:14 ║
 │ Drive...      Alt+F1 │ ═════════════╝╚══════════════════════════════════════╝
 └──────────────────────┘
F1 Help F2 Menu F3 View F4 Edit F5 Copy F6 RenMvF7 MkdirF8 DeletF9 Pull F10 Quit
```

*Lit:* OWLOSUI's menu (D15): the bar and the menu black on grey, each
hotkey letter red; `Left` and the current item, `Brief`, black on green. `√` ticks
the panel's mode and its sort order.

F9 opens the menu of the panel that is active: Left for the left panel,
Right for the right. Left and Right arrows go from title to title, and
past Right back to Left. A letter opens the title or chooses the item
whose letter is lit. Every item shows its key on the right, so the menu
is also where the keys are learned. An item that cannot be done now is
grey, not missing.

The capital letter in each item below is its hotkey.

## Left / Right

```
  Brief                 Ctrl+1
  Medium                Ctrl+2
  Full                  Ctrl+3
  Wide                  Ctrl+4
  detAiled              Ctrl+5
  ──────────────────────────────
  Info panel            Ctrl+L
  Tree panel            Ctrl+T
  Quick view            Ctrl+Q
  On/off                Ctrl+F1 / Ctrl+F2
  ──────────────────────────────
  Name                  Ctrl+F3
  eXtension             Ctrl+F4
  tiMe                  Ctrl+F5
  Size                  Ctrl+F6
  Unsorted              Ctrl+F7
  ──────────────────────────────
  Re-read               Ctrl+R
  fiLter...             Ctrl+F
  Hidden files          Ctrl+H
  Drive...              Alt+F1 / Alt+F2
```

The current mode and the current sort order have a tick.

## Files

```
  Help                  F1
  User menu             F2
  View                  F3
  Edit                  F4
  Copy                  F5
  Rename or move        F6
  Make folder           F7
  Delete                F8
  ──────────────────────────────
  File Attributes       Ctrl+A
  ──────────────────────────────
  select Group...       Gray +
  uNselect group...     Gray -
  Invert selection      Gray *
  reStore selection     Ctrl+M
  ──────────────────────────────
  Quit                  F10
```

## Commands

```
  Find file             Alt+F7
  History               Alt+F8
  Folder Tree           Alt+F10
  Directory sizes       Alt+F6
  ──────────────────────────────
  Swap panels           Ctrl+U
  Panels on/off         Ctrl+O
  Compare folders       Ctrl+C
  ──────────────────────────────
  Volume label          (DOS)
  System Info           (memory, drives)
  ──────────────────────────────
  Menu file edit
  eXtension file edit
  Folder shortcuts
```

## Options

```
  Configuration...
  Panel settings...
  Colours...            (Color / Black & White / Laptop, and each element)
  Confirmations...
  Viewer and editor...
  lanGuages...
  ──────────────────────────────
  Menu bar always visible
  Path prompt
  Key bar               Ctrl+B
  Mini status
  Clock
  Full screen panels
  Auto Directory sizes
  ──────────────────────────────
  Save setup            Shift+F9
```

Each "..." item is a grey dialog of switches, `[x]` on and `[ ]` off:
Up and Down move between them, Space or a click turns one, Enter is OK
and Esc leaves them as they were. Each switch is named after what it
does, not after a program that had it, and each one changes what the
program does - there is no switch that waits for a feature.

```
╔═══════════ Confirmations [modal] ════════════╗
║                                              ║
║  Ask before                                  ║
║                                              ║
║  [X] Copy                                    ║
║  [X] Rename or move                          ║
║  [X] Overwrite                               ║
║  [X] Delete                                  ║
║  [X] Delete a folder with files in it        ║
║  [X] Delete or overwrite a read-only file    ║
║  [X] Quit                                    ║
║                                              ║
║                         ► OK ◄▄   Cancel  ▄  ║
║                          ▀▀▀▀▀▀  ▀▀▀▀▀▀▀▀▀▀  ║
╚══════════════════════════════════════════════╝
```

*Lit:* the focus - `[x]` of Copy, black on cyan; the box black on grey.

| Dialog | Switches (default) |
|---|---|
| **Configuration** | Insert moves the cursor down (on); letters without Alt start quick search (off, 2.7); Esc on an empty command line hides the panels, and brings them back (off, 03-keys.md) |
| **Panel settings** | Show hidden and system files (on on DOS, off elsewhere; Ctrl+H); Gray + - * select folders too (off); folders in upper case, files in lower case (on; DOS only - a long name is shown as it is); column titles (on); mini status (on) |
| **Confirmations** | Copy; rename or move; overwrite; delete; delete a folder with files in it; delete or overwrite a read-only file; quit - all on. Copy or move off: F5 / F6 go to the other panel at once (Shift+F5 / Shift+F6 still ask, they need the new name). Overwrite off: an existing file is overwritten |
| **Viewer and editor** | Wrap long lines (off); colour code by its language (on); line numbers (off); WordStar keys, Ctrl+Y deletes a line (off); offer words while typing code (on) - how a file opens; F2 and the editor's menu still change them for that file |

Without column titles the names start right under the top edge; without
the mini status the column rules run down to the bottom edge (`╧`). Both
give the names their rows.

**Menu bar always visible** keeps the bar on the first line and moves
the panels one line down; **Key bar** and **Clock** are switches the
menu turns at once, ticked while on. **Languages** (its hotkey G, as
Clock has L) is below.

**Save setup** (Shift+F9) writes the switches, the colour scheme and
each panel's view and order to the setup file, which the next start
reads: `KC.INI` beside `KC.COM` on DOS, as NC kept `NC.INI`;
`%APPDATA%\KC\kc.ini` on Windows and `~/.config/kc/kc.ini` on Linux
(D11). It is lines a person can read and change:

```text
; Kirin Commander setup - Save setup (Shift+F9) writes it
[options]
insert_moves_down=1
confirm_delete=1
...
[screen]
colours=color
language=KCDEU.LNG
[left]
mode=brief
sort=name
reverse=0
[shortcuts]
3=C:\GAMES
```

A line the commander does not know is passed over, and a switch the
file does not name keeps its default. A test (`--shot`, `/SHOT`) reads
no setup file.

The items of the list above that the menu does not have yet - Path
prompt, Full screen panels, Auto Directory sizes - wait for their
features; the mini status is a switch in Panel settings.

## What the Files and Commands items do

Every item of every menu does something; none waits grey for a feature.
Two are grey where the machine cannot do them: **File Attributes** in a
browser, whose storage keeps no attributes, and **Volume label** where
the drives' labels cannot be written (everywhere but DOS).

- **Help** (F1) - the keys, in the viewer: Esc goes back.
- **User menu** (F2) - the commands a person wrote into `KC.MNU`: the one
  in the panel's folder, or, without one, the main one beside the setup
  file. Shift+F2 opens the main one always. An item is a line at the left
  edge - its key, a colon, its words - and its commands are the lines
  under it that start with a space. Its key, or Enter on it, runs them one
  after another in the panel's folder. In a command `!.!` is the file
  under the cursor, `!` its name without the extension, `!:` its drive,
  `!\` its folder (with the `\`), `!!` a `!` - NC's words:

  ```text
  A: Assemble the file
      fasm !.! !.com
  R: Run it
      !.com
  ```

- **File Attributes** (Ctrl+A) - Read-only, Hidden, System and Archive of
  the selected files, or of the one under the cursor; for one file its
  date and time too, typed as the panel shows them (`29-10-26 08:50`).
  Set writes them all; a date that did not change is not written.
- **Find file** (Alt+F7) - a mask (2.6), in the panel's folder and every
  folder under it, or on the whole drive. What it finds is a list - at
  most 1000 - whose **Go to** puts the panel on the file's folder with the
  cursor on it, and whose **View** opens it as well.
- **History** (Alt+F8) - the command lines run, the newest at the bottom:
  **Run** runs one again, **Put on the line** puts it there to change.
- **Folder Tree** (Alt+F10) - the tree in this panel, as Ctrl+T.
- **Volume label** - the label of the panel's drive, written; empty takes
  it away. On DOS it has at most 11 letters and is upper case. A drive
  that keeps its old label all the same - a folder an emulator mounted
  as a drive, which has no label of its own to write - is said so.
- **System Info** - what the machine is (DOS and its version, the
  conventional memory left; the system elsewhere), and every drive with
  what it is, its label, its size and what is free.
- **Menu file edit** - the main user menu or this folder's (Main / Local),
  in the editor; a new file if it is not there yet.
- **Extension file edit** - `KC.EXT`, beside the setup file, in the
  editor. Its lines are an extension (or several), a colon, and the
  command Enter runs for a file that has it, with the same `!` words:

  ```text
  asm inc: edit !.!
  zip: unzip -l !.!
  ```

  A file whose extension the table does not name opens in the viewer.
- **Folder shortcuts** - the ten, 0 to 9: **Go** to one (or its digit),
  **Set here** (or Ctrl+Shift and the digit in the panels), **Clear**.
  Save setup keeps them.

## Languages

**Languages** lists English and every language file beside the setup
file - `KC*.LNG`, `KCDEU.LNG` for one - by the name the file gives itself.
The one chosen speaks at once: the menus, the key bar, the dialogs, the
lists, the panels' own words. Save setup remembers it.

A language file is lines a person writes, English on the left of `=`,
the language on the right. `{}` stands for what changes - a name, a
number - and the translation puts it where its grammar wants it; a
hotkey is the letter between `~` on each side, so it can be another
letter:

```text
; Kirin Commander language file
[language]
name=Deutsch
[text]
~C~opy=~K~opieren
Copy {} to=Kopiere {} nach
{} bytes free={} Bytes frei
```

Words the file does not name stay English, so a translation half done
is still a commander that works. On DOS the file is in the machine's
own code page - each byte the letter the screen shows for it - and
elsewhere UTF-8. The help (F1) stays English.

## Behind the menu

Each item is a command with a name, so a page, a script or a model can
send it the way the browser console does (`kc view`, `kc copy --to D:\`)
- see [07-command-line.md](07-command-line.md). The menu is one way to
send them, the keys another.
