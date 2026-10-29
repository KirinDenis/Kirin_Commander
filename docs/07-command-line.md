# 7. The command line and the terminal behind the panels

The panels lie over a terminal - DOS's screen, the system's shell, the
browser's console - and the command line under them is that terminal's
own prompt. This is what made NC a shell and not a file browser: what
is typed runs, and what it printed is behind the panels, one Ctrl+O away.

## 7.1 Rules, the same everywhere

1. **The command line is always there**, on the line above the key bar,
   showing the active panel's folder as the prompt (`C:\GAMES>`). Only an
   option ("Path prompt" off) shortens the prompt; nothing hides the line
   while the panels are up.
2. **Typing goes to it** while the panels are shown - every letter that is
   not a key of the panels. Alt+letters are quick search (section 2.7).
3. **Enter with text runs it.** Enter with the line empty acts on the
   panel: into a folder, run a program.
4. **While a command runs, the panels go away**, and the terminal shows
   what it prints, full screen, as if the commander were not there. When
   it ends the panels come back by themselves and are re-read - the
   command may have changed the folder.
5. **`cd` typed changes the active panel's folder**, and a drive letter
   alone (`D:`) its drive: the command line and the panel are one place.
   The other way round too: moving in the panel changes the prompt, and
   the next command runs there.
6. **Ctrl+O takes the panels away** and leaves the terminal: what the
   commands printed, the command line at its bottom, the key bar under it
   (Ctrl+B hides that too). Ctrl+O again brings them back. **With the
   panels off, the terminal is a terminal**: Tab completes names, Up and
   Down walk the history, Ctrl+C stops what runs, Enter on an empty line
   is an empty line - nothing of the panels' is in the way. F-keys keep
   their meaning, on the file under the hidden panel's cursor, as in NC.
7. **The history** is the terminal's: Ctrl+E / Ctrl+X step through it on
   the line, Alt+F8 shows it as a list (Enter runs, Shift+Enter puts it on
   the line, Del forgets one). Kept between runs.
8. **Names onto the line**: Ctrl+Enter the name under the cursor, Ctrl+[
   and Ctrl+] a panel's path (section 3.3) - so a command about a file is
   built without typing its name.
9. **Ctrl+Up and Ctrl+Down** (Far) make the panels shorter or taller, so
   the last lines the terminal printed show under them.

## 7.2 DOS

- The terminal is DOS's own screen. Kirin Commander keeps a copy of what
  was on it when the panels came up, and puts it back on Ctrl+O and before
  a program runs, so the program's output continues where the last one
  stopped, as in NC.
- A command runs through `COMMAND.COM /C` - so internal commands (`DIR`,
  `COPY`, `SET`), batch files and paths all work as typed - and `%COMSPEC%`
  decides which shell.
- **Memory**: a program run from the commander gets all the memory DOS
  can give. Started with `/STEP` - by a batch file that says it will run
  what the commander leaves - Kirin Commander steps out of memory to run
  it and comes back after (kc/src/step.rs, D18): it writes the command
  lines into `RUNPROG.BAT` and where it was into `KC.SAV`, both in the
  folder it was started in, and leaves; the batch file runs `RUNPROG.BAT`
  and starts it again, and it reads `KC.SAV` once - the same folders, the
  same names under the cursors, the same panel active, the same history
  and switches. What the program printed is the terminal behind the
  panels, as after a run in memory. Selection is not kept.

  ```text
  :again
  CWSDPMI.EXE
  KC.COM /STEP A:\ C:\
  if not exist RUNPROG.BAT goto done
  call RUNPROG.BAT
  del RUNPROG.BAT
  goto again
  :done
  ```

  Without `/STEP` the command runs from inside, in what memory the
  commander leaves. Folders after the switches (`KC.COM A:\ C:\`) are the
  panels' at the start.
- `cd` and `D:` typed are done by the commander itself, without running
  COMMAND.COM - they must change *its* folder, which a child shell cannot.
- The environment (`SET`, `PATH`) of a command run this way is the
  commander's; a `SET` typed is done by the commander too, as NC did.

## 7.3 Windows and Linux

- The terminal is a real one: a pseudo-terminal (ConPTY on Windows, a pty
  on Linux) running the person's shell - `cmd.exe` or PowerShell (an
  option), `$SHELL` on Linux - drawn in an OWLOSUI console view behind the
  panels, with its colours and its cursor.
- The command line is handed to that shell as typed. The shell's own
  folder follows the panel: before each command the commander tells it
  `cd` to the panel's folder; after it, the shell's folder is read back
  (`$PWD`, or the prompt's report) and becomes the panel's.
- A full-screen program (an editor, `top`, `mc` itself) gets the whole
  screen while it runs, and the panels come back after.

## 7.4 The browser: WIRE CITY's console

In a browser there is no system shell; there is the site's console, and
it is a good one. Kirin Commander's terminal **is** that console:

- **The same shell**: its grammar (`;`, `&&`, `||`, pipes, `>`), its
  commands - `ls`, `cd`, `cat`, `grep`, `cp`, `mv`, `rm`, `mkdir`, `du`,
  `df`, `mount`, `umount`, `relabel`, `run`, `build`, `play`, `edit`,
  `dos`, `zip`, `help`, `man` - its history, its Tab completion of paths,
  drive letters, games and lessons.
- **The same paths**: `C:\GAMES` and `/mnt/c/GAMES` both, the drive letters
  of the site's drive table, case-blind over case-sensitive storage.
- **The same drives**: what `mount` adds - a storage folder, a folder of
  the computer (`--computer`), a ZIP, an address - appears in the drive
  menu (Alt+F1/F2) at once, and `umount` takes it away. The menu's own
  "Mount..." item opens the same dialog the command would.
- **The same confirmations**: a command whose effect deletes, sets up or
  shares asks the person Yes/No when it comes from a model or a page, and
  runs as typed when the person typed it - the console's rule, kept.
- **The commander's operations are console commands**: F5 on a file sends
  `cp` the same way the menu does, so they show in the history, a model
  can do them over the site's MCP, and the effect rules apply to them.
  The dialogs still ask what [05-file-operations.md](05-file-operations.md)
  says they ask - for the person at the keyboard the dialog is the
  confirmation.
- **`run`** starts a DOS program on the site's DOS PC; the DOS PC's window
  is the "terminal" for it, and the panels come back when it is closed.
- **Ctrl+O** shows the console full screen; the console's own Ctrl+`
  drop-down stays what it is for the rest of the site.

## 7.5 Things the command line never does

- Run a command the person did not type or choose: Enter on a panel item
  with an empty line opens it, it does not run something with its name.
- Lose what was typed: a panel key pressed with text on the line (Tab,
  F5) leaves the text where it is.
- Swallow an error: a command that failed says so in the terminal, and
  its exit code is in `%ERRORLEVEL%` / `$?` for the next one.
