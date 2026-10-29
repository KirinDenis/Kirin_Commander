# 9. Decisions, and what OWLOSUI must gain

Status: **accepted** by the author on 2026-10-28, as proposed - with one
change that building found: on DOS the program is `KC.COM`, OWLOSUI's
loader, and `KC.BIN` beside it, the commander in extended memory (D11).

## 9.1 Decisions

| # | Question | Proposed | Why |
|---|---|---|---|
| D1 | One program or three | **One**, in Rust, linking OWLOSUI's core: built for DOS (OWLOSUI's `lib/dos` target, the core linked in as its Rust DOS example does), for Windows and Linux (its native window and its terminal backends), for the browser (WebAssembly, as its wire is) | Today there are three commanders (Pascal, C#, JavaScript) that already disagree on F2, F6, Enter, the tree and the drive menu. One program cannot disagree with itself. The platform differences are `Fs` (section 6) and the terminal (section 7), nothing else |
| D2 | Which program is the reference | **Volkov Commander 4.05** for the look on DOS - its source has the exact strings, labels and colours; **NC** wording where VC differs; **Far** for long names and for keys NC never had | VC is a faithful NC 3 copy whose source we can read; Far is what NC became |
| D3 | Ctrl+F3..F7 | **Sort** (Name, Extension, Time, Size, Unsorted), as NC and Far | VC put label and single-file copy there; NC and Far users press Ctrl+F3 to sort by name |
| D4 | Ctrl+Enter | **The name onto the command line**, as NC and Far; Ctrl+I (VC) too | The OFM standard and Far agree |
| D5 | The menu's titles | **Left, Files, Commands, Options, Right** | NC 3, VC and Far have these five; NC 5's Disk menu was about floppies |
| D6 | Panel text colour | **Light cyan on blue** (VC 1Bh) | NC and VC; MC's light grey is MC's |
| D7 | The right mouse button in a panel | **Selects**, as in Far; the context menu on a hold (touch) and on Shift+F10-style menu key | Far's habit; an option for OWLOSUI's own context menu |
| D8 | Enter on a file that is not a program | **The extension table**; without an entry, **view** (an option: nothing, as NC) | Nothing happening on Enter reads as broken today |
| D9 | Delete on a desktop | **To the Recycle Bin / Trash**, Shift+Del for good | What the systems' own file managers do |
| D10 | A phone in portrait (about 46 columns) | **One panel at a time**, the whole width, Tab switching which one; two panels side by side in landscape | Two 23-column panels show nothing |
| D11 | Names | "Kirin Commander"; the program `KC.COM` on DOS (the loader; `KC.BIN` beside it), `kc` on a desktop; its folder of settings `KC` | A flat 32-bit binary under DPMI is how OWLOSUI's Rust runs on DOS: a small COM loads it |
| D12 | How OWLOSUI comes in | Rust: a git dependency pinned to an OWLOSUI commit; the web parts: copied, as WIRE CITY's `pull-owlosui.ps1` does | Reproducible builds, and OWLOSUI changes reviewed before they are taken |
| D13 | The three commanders in OWLOSUI | Become **one small example per platform** of a file panel, as an example should be; the site's `commander` command opens Kirin Commander | OWLOSUI's examples are kept thin |
| D14 | The drive menu on DOS | **A column**: the letter, the size, the free bytes and the free share, for each fixed disk; `removable` and `remote` for the drives not asked | NC and VC showed only the letters in a row; the size and free space were in the Info panel, one keystroke further. The space is asked once, as the menu opens, and never of a floppy drive |
| D15 | Whose menus, key bar and dialogs | **OWLOSUI's**, and the commander is an OWLOSUI window with no frame of its own - the panels' frames are its frame; its menus and keys are the window's, put on the desktop's bars while it is active: its menu bar and menus (F9), its status line as the key bar, its grey dialogs with buttons, the drive menu a list in one. The file panels, the command line and the terminal behind them stay the classic ones, drawn by the commander | The pilot, 2026-10-28: the panels are the classic part; what is around and over them is OWLOSUI. One look for every OWLOSUI program, and the toolkit's menus, buttons and keys are its own, tested there. Done 2026-10-28: the menu, the key bar (the panels' and the viewer's and editor's), the drive menu, every dialog (copy, move, delete, the questions, errors, Options, the editor's) and the progress window. A dialog stays data (`Dialog`); `chrome::open_form` shows it as OWLOSUI's window and reads the answer back |
| D16 | The key bar's words | **`F1 Help`** in ten even cells, a tenth of the width each, no gap; a word too long for its cell shortened (`RenMv`, `Delet`, `Pull`) | The pilot, 2026-10-29: people who never saw NC do not guess that a red `1` is F1. Ten labels of `F1 Help` with gaps take 88 columns, and DOS has 80 - so the classic even grid, and OWLOSUI's status line learnt cells (`StatusLine::cell`) |
| D17 | Every menu item, and languages | **Each item does what it says** - Help, the user menu, File Attributes, Find file, History, Folder Tree, Volume label, System Info, Menu file edit, Extension file edit, Folder shortcuts, Languages; grey only where the machine cannot (attributes in a browser, labels off DOS). **Languages are files** - `KC*.LNG` beside the setup, English `=` the language, `{}` for what changes - not built in | The pilot, 2026-10-29: "доделаем все пункты меню"; a language is a file a person writes, so the commander does not wait for translations to have the item |
| D18 | Stepping out on DOS | **`/STEP`**: the command lines go into `RUNPROG.BAT`, where the commander was into `KC.SAV`, in the folder it started in; the batch file that started it runs them and starts it again, which reads `KC.SAV` once. Folders on the command line are the panels'. Programs that need OWLOSUI's resident are not told apart: they run as any other | WIRE CITY's course disk and site start into KC (2026-10-29), and OWL FLY III needs all of DOS's memory, which a DPMI commander in memory does not leave; the course's Pascal commander did the same |

## 9.2 What OWLOSUI's core must gain

Kirin Commander is the first program that needs these; every program on
OWLOSUI gets them.

1. **Two panels in one window**: a view that lays out two file panels side
   by side in one frame, the division movable from the keyboard; not two
   windows.
2. **Panel view modes**: Brief (three columns in 40 cells), Medium, Full,
   Wide, Detailed; name and extension in sub-columns on DOS; folders upper
   and files lower case; `►SUB-DIR◄` / `►UP--DIR◄`; the `░` mark; `}` for a
   cut name; the mini status in its two forms (the item; the selection's
   total).
3. **The cursor rules** of [02-panels.md](02-panels.md) section 2.4. Today
   going up lands on the first name (`cursor_to_first_name` in the wire's
   SET_FILES) and a removed name sends the cursor to the top.
4. **Modifier keys as events**: the key bar changes while Shift, Ctrl or
   Alt is *held*, so the core must hear a modifier go down and up on its
   own - today only keys with modifiers reach it.
5. **A menu bar that is hidden** until F9 / a click on line 1 / Alt alone,
   and gives the line back when closed.
6. **A key bar** as the status line already is, with the digit and label
   colours, and its rows per modifier.
7. **Quick search**: a small input under the panel, opened by Alt+letter.
8. **Full-screen viewer and editor**: the text view without a frame, a
   status line on top, and the menu hidden as in 5. *Done for the
   commander: `Window::pinned` (a window the work area never nudges) lets
   its frame sit just off the screen.*
9. **A terminal view**: OWLOSUI's console view fed by a pseudo-terminal
   (desktop) or by the site's console (browser), with a cursor and keys
   passed through while it has the screen.
10. **DOS**: an INT 24h handler in the resident, so critical errors come
    back as errors; INT 21h 59h read after a failure.
11. **The VC colour scheme** as a named palette, with Black & White and
    Laptop.

## 9.3 Where the commanders are today

From reading OWLOSUI's three commanders (web `lib/js/apps/commander.js`,
DOS `Examples/DOS/Commandr/COMMANDR.PAS`, desktop
`Examples/Desktop/CSharp/03-Commander/Program.cs`), against this spec:

| | Today | The spec |
|---|---|---|
| Panels | Two separate windows, each moved and sized on its own | One screen, two panels |
| Columns | Two name columns in a half-screen panel; no Brief, no Full | Brief = 3, Full = Name Size Date Time |
| Out of a folder | Cursor on the first name | Cursor on the folder just left |
| After delete | Cursor jumps to the top | The next name |
| After mkdir | Cursor stays | On the new folder |
| Drive menu | A dialog with OK/Cancel; the current drive not chosen; titled differently in each | A menu over the panel, the current drive under the cursor, a letter chooses |
| F2 | Upload (web), save (DOS), nothing (desktop) | User menu |
| F6 | Rename only on the web | Rename or move everywhere |
| F10 | Not on the web | Quit, with a question |
| Enter on a file | Opens (web); runs or Properties (DOS, desktop) | Runs a program, else the extension table |
| Tree (Alt+F10) | DOS and desktop only | Everywhere |
| Ctrl+O, Ctrl+P, Ctrl+U, Gray + − \*, Ctrl+F3..F7, Shift+F-keys, Ctrl+PgUp/PgDn | Mostly missing | All |
| Menu bar | Web only, always shown | Hidden, F9 |
| Key bar | Different labels in each; no modifier rows | One bar, with Shift/Ctrl/Alt rows |
| Viewer / editor | Windows with Save/Close buttons (DOS, desktop); Save offered in the read-only viewer (web) | Full screen, no buttons, a hidden menu |
| Command line, Ctrl+O terminal | None | Always there; the terminal behind |
| Overwrite | Copy and move overwrite without asking (web) | Always asked |
| Errors | A box with OK, the operation stops | Retry / Skip / Skip all / Cancel per file |
