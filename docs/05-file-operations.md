# 5. File operations

What every operation shares, then each one. The file systems underneath
are in [06-file-systems.md](06-file-systems.md); here is what the person
sees.

## 5.1 Rules for all of them

1. **What is acted on**: the selection, if the panel has one; otherwise
   the file under the cursor. Never `..`. The dialog names it: `"hello.asm"`
   for one, `3 files` / `2 folders and 5 files` for more.
2. **Asked first, done after**: every operation that changes the disk opens
   a dialog first, Enter in it is the go-ahead, Esc does nothing. The only
   exceptions are Shift+F5/F6 "in place" with a name typed, which are the
   dialog.
3. **One at a time, the screen alive**: a long operation shows progress
   and can be stopped with Esc, which asks "Stop?" - the file being
   written is finished or removed, never left half-written under its
   real name (section 5.7).
4. **Errors stop and ask**, file by file (section 5.8). They are never
   only written to a log, and never swallowed.
5. **Afterwards**: both panels re-read; the cursor rules of
   [02-panels.md](02-panels.md) section 2.4; only what was done is
   unselected.
6. **Nothing is lost on a name clash**: overwriting is always asked unless
   the person said "All" for this operation.
7. **The same on DOS, a desktop and a browser**: the dialogs, their words
   and their buttons are the same; only what a file system cannot do
   (attributes in a browser, links on DOS) is not offered.

## 5.2 Copy (F5) and Move / Rename (F6)

```text
╔════════════╤ C:\LESSONS ╤════════════╗╔═══════════ C:\LESSONS\L07 ════╤═12:34╗
║    Name    │    Name    │    Name    ║║    Name    │  Size   │  Date  │ Time ║
║..          │L18         │readme   txt║║..          │►UP--DIR◄│28-10-26│ 09:12║
║L01         │L19╔═══════════════ Copy [modal] ═══════════════╗│24-10-26│ 15:34║
║L02         │L20║                                            ║│28-10-26│ 09:15║
║L03         │L21║  Copy 2 files to                           ║│24-10-26│ 15:34║
║L04         │L22║  C:\LESSONS\L07\                           ║│28-10-26│ 09:14║
║L05         │L23║                                            ║│28-10-26│ 09:15║
║L06         │L24║  Already existing files:                   ║│        │      ║
║L07         │L25║  (•) Ask                                   ║│        │      ║
║L08         │L26║  ( ) Overwrite                             ║│        │      ║
║L09         │L27║  ( ) Skip                                  ║│        │      ║
║L10         │L28║  ( ) Overwrite older                       ║│        │      ║
║L11         │L29║  ( ) Rename the new                        ║│        │      ║
║L12         │L30║  [X] Copy attributes and dates             ║│        │      ║
║L13         │L31║                                            ║│        │      ║
║L14         │cou║                     ► Copy ◄▄   Cancel  ▄  ║│        │      ║
║L15         │go ║                      ▀▀▀▀▀▀▀▀  ▀▀▀▀▀▀▀▀▀▀  ║│        │      ║
║L16         │les╚════════════════════════════════════════════╝│        │      ║
║L17         │make     bat│            ║║            │         │        │      ║
╟────────────┴────────────┴────────────╢╟────────────┴─────────┴────────┴──────╢
║   3 914 bytes in 2 selected files    ║║hello    asm     2 345 28-10-26 09:14 ║
╚══════════════════════════════════════╝╚══════════════════════════════════════╝
C:\LESSONS>

```

*Lit:* OWLOSUI's grey dialog (D15): the target line's text all selected, so typing replaces it; the choice for existing files as radio buttons; `Copy`, the default, between `► ◄`.

- **The target line** comes filled with the other panel's path; the whole
  text is selected, so typing replaces it and an arrow first lets it be
  edited (NC's rule).
- **What the target means**:
  - an existing folder: into it, under the same names;
  - a name ending in `\` (or `/`): a folder, made if it is not there;
  - otherwise, for one file: the new name. For several files and a name
    that is not a folder: a mask - `*.bak` copies each with that extension.
  - a relative name: relative to the source panel's folder. So F6 with
    `hello2.asm` typed is a rename in place.
- **F10 in the line** opens the folder tree to choose the target.
- **Already existing files**: Ask (default), Overwrite, Skip, Overwrite
  only older, Rename the new one (`name (2).ext`; `NAME~1.EXT` on DOS).
- **A folder copied** copies everything in it; into a folder of the same
  name it merges, asking per file as above.
- **Copy onto itself** (same folder, same name) is refused before anything
  starts: "Cannot copy a file onto itself."
- **A folder into itself** (copy `C:\A` to `C:\A\B`) is refused the same way.
- **Move on one volume** is a rename - instant, nothing copied. **Move
  across volumes** is copy, then delete of each source only after its copy
  is verified complete (size compared; on a desktop also a byte count).
  A move that stops half way leaves every file either at the source or at
  the target, never in neither.

## 5.3 The overwrite question

```text
╔═══════════════════ Warning [modal] ════════════════════╗
║                                                        ║
║  The file already exists:                              ║
║  C:\LESSONS\L07\GO.BAT                                 ║
║                                                        ║
║  New:             38  24-10-26  15:34                  ║
║  Existing:        38  24-10-26  15:34                  ║
║                                                        ║
║                                                        ║
║       ► Overwrite ◄▄   All  ▄   Skip  ▄   Skip all  ▄  ║
║        ▀▀▀▀▀▀▀▀▀▀▀▀▀  ▀▀▀▀▀▀▀  ▀▀▀▀▀▀▀▀  ▀▀▀▀▀▀▀▀▀▀▀▀  ║
║                     Append  ▄   Rename  ▄   Cancel  ▄  ║
║                    ▀▀▀▀▀▀▀▀▀▀  ▀▀▀▀▀▀▀▀▀▀  ▀▀▀▀▀▀▀▀▀▀  ║
╚════════════════════════════════════════════════════════╝
```

*Lit:* OWLOSUI's grey dialog (D15); `Overwrite`, the default, between `► ◄`. Seven answers are two rows of buttons.

- Sizes and dates of both, so the choice is an informed one.
- A read-only target: the question says so ("is read-only. Overwrite it
  anyway?") and is asked even after All, unless "Also overwrite read-only"
  was chosen.
- F3 in the question views either file (Far).

## 5.4 Delete (F8)

```text
╔═══════ Delete [modal] ═══════╗
║                              ║
║  Do you wish to delete       ║
║  "HELLO.ASM"                 ║
║                              ║
║                              ║
║     ► Delete ◄▄   Cancel  ▄  ║
║      ▀▀▀▀▀▀▀▀▀▀  ▀▀▀▀▀▀▀▀▀▀  ║
╚══════════════════════════════╝
```

*Lit:* OWLOSUI's grey dialog (D15); `Delete`, the default, between `► ◄`.

```text
╔═════════════════ Delete [modal] ══════════════════╗
║                                                   ║
║  The folder C:\LESSONS\L07 is not empty.          ║
║  Delete it with everything in it?                 ║
║                                                   ║
║                                                   ║
║       ► Delete ◄▄   All  ▄   Skip  ▄   Cancel  ▄  ║
║        ▀▀▀▀▀▀▀▀▀▀  ▀▀▀▀▀▀▀  ▀▀▀▀▀▀▀▀  ▀▀▀▀▀▀▀▀▀▀  ║
╚═══════════════════════════════════════════════════╝
```

*Lit:* the same grey dialog; the second question names the folder and says what is in danger, so it is not answered by habit.

- For several: `Do you wish to delete 3 files`.
- **A folder with files in it**: a second question, naming it: "The folder D:\WORK is not empty. Delete it with everything
  in it?" Delete / All / Skip / Cancel.
- A read-only file: asked again, as for overwrite.
- **On a desktop, to the Recycle Bin / Trash** by default (an option);
  Shift+Delete deletes for good, and its question says "for good".
- On DOS and in the browser there is no bin; the question says "This cannot
  be undone." In the browser, the site's own journal of removals still
  applies (see [06-file-systems.md](06-file-systems.md)).

## 5.5 Make folder (F7)

- One line: `Create the folder` with the name. Several levels at once:
  `A\B\C`.
- On DOS the name is checked as 8.3 before anything is made, and the error
  says what is wrong ("A name has at most 8 letters before the dot and 3
  after").
- The cursor lands on the new folder.

## 5.6 Attributes (Ctrl+A)

On DOS and Windows: Read-only, Hidden, System, Archive, and the date and
time - of the selected files, or the one under the cursor; the date for
one file only, typed as the panel shows it (`29-10-26 08:50`). On Linux:
the permission bits and owner, and the date (not yet: the four switches
are DOS's). In a browser: only what the storage has (nothing, for OPFS) -
the item is grey. See [04-menus.md](04-menus.md), What the Files and
Commands items do.

## 5.7 Progress

```text
╔═════════════════════ Copy ═════════════════════╗
║  Copying the file                              ║
║  C:\LESSONS\L07\DISK.IMG                       ║
║  to                                            ║
║  D:\DISK.IMG                                   ║
║  ███████████████████░░░░░░░░░░░░░░░░░░░░░ 48%  ║
║  Total: 0 of 1 files, 1.4 M of 3.0 M           ║
║  ███████████████████░░░░░░░░░░░░░░░░░░░░░ 48%  ║
╚════════════════════════════════════════════════╝
```

*Lit:* OWLOSUI's grey window and its progress bars (`█` done, `░` to do, the per cent at the right end). Not modal: Esc still asks to stop.

- Shown only if the operation takes longer than half a second.
- The total is counted first (a scan, which can be skipped with Esc: then
  only the file bar is shown).
- A file being copied is written under a temporary name in the target
  folder and renamed when complete, so a stop or a crash never leaves a
  short file looking like a whole one. (Not on DOS, where a temporary name
  costs an 8.3 slot and a rename costs nothing: there the half-written file
  is deleted on a stop.)

## 5.8 Errors

Every error is OWLOSUI's grey dialog, titled Error, with **what was being done**,
**to what**, **why** in the system's words made human, and **what can be
done now**:

```text
╔════════════╤ C:\LESSONS ╤════════════╗╔═══════════ C:\LESSONS\L07 ════╤═12:34╗
║    Name    │    Name    │    Name    ║║    Name    │  Size   │  Date  │ Time ║
║..          │L18         │readme   txt║║..          │►UP--DIR◄│28-10-26│ 09:12║
║L01         │L19         │tools    txt║║build    bat│      156│24-10-26│ 15:34║
║L02         │L20         │            ║║build    txt│      211│28-10-26│ 09:15║
║L03         │L21         │            ║║go       bat│       38│24-10-26│ 15:34║
║L04       ╔════════════════════ Error [modal] ════════════════════╗0-26│ 09:14║
║L05       ║                                                       ║0-26│ 09:15║
║L06       ║  Cannot copy the file                                 ║    │      ║
║L07       ║  C:\LESSONS\L07\README.TXT                            ║    │      ║
║L08       ║  The disk is write-protected.                         ║    │      ║
║L09       ║                                                       ║    │      ║
║L10       ║                                                       ║    │      ║
║L11       ║       ► Retry ◄▄   Skip  ▄   Skip all  ▄   Cancel  ▄  ║    │      ║
║L12       ║        ▀▀▀▀▀▀▀▀▀  ▀▀▀▀▀▀▀▀  ▀▀▀▀▀▀▀▀▀▀▀▀  ▀▀▀▀▀▀▀▀▀▀  ║    │      ║
║L13       ╚═══════════════════════════════════════════════════════╝    │      ║
║L14         │course   txt│            ║║            │         │        │      ║
║L15         │go       bat│            ║║            │         │        │      ║
║L16         │lessons  ini│            ║║            │         │        │      ║
║L17         │make     bat│            ║║            │         │        │      ║
╟────────────┴────────────┴────────────╢╟────────────┴─────────┴────────┴──────╢
║   3 914 bytes in 2 selected files    ║║hello    asm     2 345 28-10-26 09:14 ║
╚══════════════════════════════════════╝╚══════════════════════════════════════╝
C:\LESSONS>

```

*Lit:* OWLOSUI's grey dialog (D15); `Retry`, the default, between `► ◄`; the system's own code, when there is one, at the right end of its line.

- **Retry** after the person fixed it (a disk put in, space made); **Skip**
  this file and go on; **Skip all** for the same kind of error; **Cancel**
  the rest.
- The words come from one table of error kinds - [06-file-systems.md](06-file-systems.md)
  section 6.4 - so "the disk is full" reads the same whether DOS said
  error 0x27, Windows `ERROR_DISK_FULL`, Linux `ENOSPC` or the browser
  `QuotaExceededError`.
- The system's own code is in the box too, small, at the end
  (`DOS error 27h`), for the person who needs it.
- **DOS's critical errors** (INT 24h: drive not ready, write-protected,
  general failure) never reach DOS's own "Abort, Retry, Fail?": Kirin
  Commander takes INT 24h for itself and asks in its own box, with the
  same buttons.
- After the operation, if anything was skipped: one summary box, "3 files
  were not copied", with a list.
