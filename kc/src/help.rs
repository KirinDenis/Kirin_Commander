//! F1: the keys, in the viewer, as the docs give them (docs/03-keys.md) -
//! short enough to read on one machine without the other.

pub const TEXT: &str = "\
Kirin Commander - the keys

Esc leaves this, as it leaves every window.

The panels
  Tab             the other panel
  Enter           into a folder; run a program; a file: what KC.EXT says,
                  else the viewer
  Backspace       out to the parent folder (the command line empty)
  Ctrl+\\          to the root
  Insert          select the file, and down
  Shift+arrows    select along the way
  Gray + - *      select by mask, unselect by mask, invert
  Ctrl+M          the last selection back
  Alt+letters     quick search
  Ctrl+1..Ctrl+5  Brief, Medium, Full, Wide, Detailed
  Ctrl+F3..F7     sort by name, extension, time, size; unsorted
  Ctrl+L          the Info panel in the other panel
  Ctrl+Q          Quick view in the other panel
  Ctrl+T, Alt+F10 the folder tree in this panel
  Ctrl+O          the panels away: what ran behind them
  Ctrl+U          the panels swapped
  Ctrl+R          read again
  Ctrl+H          hidden files shown or not
  Ctrl+F          the filter
  Ctrl+A          the file's attributes
  Ctrl+B          the key bar on or off
  Ctrl+Shift+0..9 this folder as shortcut 0..9 (F9, Commands, Folder
                  shortcuts goes to one)
  Alt+F1, Alt+F2  the drive of the left, the right panel

The function keys
  F1  this help              Shift+F2  the main user menu
  F2  the user menu (KC.MNU) Shift+F4  edit a file by its name
  F3  view (a folder: size)  Shift+F5  copy in place, a new name
  F4  edit                   Shift+F6  rename in place
  F5  copy                   Shift+F8  delete the file under the cursor
  F6  rename or move         Shift+F9  save the setup
  F7  make a folder          Alt+F6    count the folders' sizes
  F8  delete                 Alt+F7    find files
  F9  the menu               Alt+F8    the command history
  F10 quit                   Alt+F10   the folder tree

The command line
  Enter           run it (cd moves the panel)
  Ctrl+Enter      the name under the cursor onto the line
  Ctrl+[  Ctrl+]  the left, the right panel's folder onto the line
  Ctrl+E  Ctrl+X  the previous, the next command
  Up  Down        the same, with the panels away (Ctrl+O)
  Esc             the line cleared

In a dialog
  Tab, Shift+Tab  the next, the previous control
  Enter           the button that is lit
  Esc             Cancel
  a button's letter, or Alt and it, presses it

The user menu, KC.MNU
  An item is a line at the left edge: its key, a colon, its words. The
  commands under it start with a space. In a command, !.! is the file
  under the cursor, ! its name without the extension, !: its drive and
  !\\ its folder.

    A: Assemble the file
        fasm !.! !.com

  F2 opens the menu of the folder the panel is in, or, without one, the
  main menu beside the setup file. F9, Commands, Menu file edit makes it.

The extension table, KC.EXT
  An extension (or several), a colon and the command Enter runs for a
  file that has it:

    asm inc: edit !.!
";
