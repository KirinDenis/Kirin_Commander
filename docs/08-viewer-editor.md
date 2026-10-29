# 8. The viewer (F3) and the editor (F4)

Both take the whole screen, as in NC, VC and Far: no window frame of their
own, no buttons. A status line at the top, the text, the key bar at the
bottom, and a **menu bar that is hidden until F9 or Alt**, which holds
everything the keys do - so a person who does not know the keys finds
them there, and one who knows them never sees it.

Both are OWLOSUI's own text view and editor, with what OWLOSUI already
gives them: syntax colours, line numbers, word wrap, hex, find and
replace, undo, the clipboard shared with the system, words offered while
typing in code, touch. Kirin Commander adds the screen around them.

## 8.1 The viewer

```text
 C:\LESSONS\L07\HELLO.ASM                 Col 1    2 345 bytes   100%           
; HELLO.ASM - a line on the screen, through DOS                                 
                                                                                
        org  100h                                                               
                                                                                
start:                                                                          
        mov  ah,9           ; DOS: write the string up to the $                 
        mov  dx,text        ; DS:DX - where it is                               
        int  21h                                                                
        mov  ax,4C00h       ; DOS: end the program, errorlevel 0                
        int  21h                                                                
                                                                                
text    db   'Hello, DOS!',13,10,'$'                                            
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
F1 Help F2 Wrap F3 Quit F4 Hex  F5 Goto F6 Edit F7 Srch F8 Code F9 Menu F10 Quit
```

*Lit:* the status line black on cyan; the key bar OWLOSUI's status line, grey with the keys red (D15), and its dialogs OWLOSUI's grey windows; the text light cyan on blue; mnemonics and registers white, comments grey, as OWLOSUI colours assembler.

| Key | Does |
|---|---|
| F2 | Wrap / unwrap long lines |
| F3, F10, Esc | Back to the panels |
| F4 | Text / hex |
| F5 | Go to a line, a percentage, or (hex) an offset |
| F6 | The editor, on the same place |
| F7, Shift+F7 | Search / search again (text or hex, case, whole words) |
| F8 | Code page (DOS 437/866, Windows 1251, UTF-8) |
| F9 or Alt | The menu |
| Gray + / Gray − | The next / previous file of the panel (Far) |
| Ctrl+F10 | Back to the panels with the cursor on this file (Far) |
| Up Down PgUp PgDn Home End, Left Right | Move; sideways when unwrapped |

The viewer opens any file, of any size, at once: it reads only what is on
the screen. A file that is not text opens in hex by itself.

**Its menu**: File (Edit F6, Next file, Previous file, Back to panels) /
Search (Search F7, Again Shift+F7, Go to F5) / View (Wrap F2, Hex F4,
Code page F8, Line numbers).

## 8.2 The editor

```text
 C:\LESSONS\L07\HELLO.ASM *               Line 9   Col 23   Assembler           
; HELLO.ASM - a line on the screen, through DOS                                 
                                                                                
        org  100h                                                               
                                                                                
start:                                                                          
        mov  ah,9           ; DOS: write the string up to the $                 
        mov  dx,text        ; DS:DX - where it is                               
        int  21h                                                                
        mov  ax,4C00h  _    ; DOS: end the program, errorlevel 0                
        int  21h                                                                
                                                                                
text    db   'Hello, DOS!',13,10,'$'                                            
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
F1 Help F2 Save F3 Mark F4 Repl F5 Copy F6 Move F7 Srch F8 DeletF9 Menu F10 Quit
```

*Lit:* as the viewer, and `*` after the name: the text is changed and not saved. The caret in line 9.

| Key | Does |
|---|---|
| F2 | Save |
| Shift+F2 | Save as - a name, the code page, line ends (CRLF on DOS, as found otherwise) |
| F7, Shift+F7, Ctrl+F7 | Search, again, replace |
| Alt+F8 | Go to line (and column) |
| F9 or Alt | The menu |
| F10, Esc | Back to the panels - "You have made changes since the last save." **Save / Don't save / Continue editing** when the text is changed |
| Shift+F10 | Save and back |
| Ctrl+Z, Ctrl+Y | Undo, redo |
| Shift+arrows, Ctrl+A, Ctrl+C/X/V, Ctrl+Ins/Shift+Del/Shift+Ins | Selection and the clipboard |
| Ctrl+Y (classic keys) | Delete the line - WordStar keys are OWLOSUI's "Classic keys" option |
| Tab, Up, Down, Esc while words are offered | Take one, choose, put away |

**Its menu** - this is what the "Save" buttons become:

```text
  File    Edit    Search    View    Options                                     
┌────────────────────────┐e screen, through DOS                                 
│ Save                F2 │                                                      
│ Save as...    Shift+F2 │                                                      
├────────────────────────┤                                                      
│ Open...       Shift+F4 │                                                      
│ Reload                 │  ; DOS: write the string up to the $                 
├────────────────────────┤  ; DS:DX - where it is                               
│ Close              F10 │                                                      
└────────────────────────┘  ; DOS: end the program, errorlevel 0                
        int  21h                                                                
                                                                                
text    db   'Hello, DOS!',13,10,'$'                                            
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
F1 Help F2 Save F3 Mark F4 Repl F5 Copy F6 Move F7 Srch F8 DeletF9 Menu F10 Quit
```

*Lit:* `File` white on black in the bar; the menu white on cyan with yellow hotkeys; `Save` white on black.

- The menu bar is the screen's first line, hidden while the text is
  being written: F9 or Alt shows it, Esc or a choice hides it again.
- No button anywhere in the editor. A dialog the editor opens (Save as,
  Find) has its buttons, as every dialog does.
- Shift+F4 in the panels opens the editor on a typed name, new or not;
  Saving a new file puts the panel's cursor on it.

## 8.3 What happens when

- **A file that changed on the disk** while it was open: on save, "The file
  has been changed since it was opened. Save over it?" On returning to the
  editor (a background run on a desktop), the status line says so.
- **A file opened twice** (F4 on a file already being edited): the editor
  that has it is brought back, not a second one opened.
- **A file that cannot be saved** (read-only, disk full): the error box of
  [05-file-operations.md](05-file-operations.md) section 5.8, with Retry,
  Save as and Cancel; the text is never lost because the save failed.
- **A big file**: the viewer has no limit; the editor holds what OWLOSUI's
  editor holds, and says so before opening one bigger rather than cutting
  it ("This file is 6 M; the editor holds 2 M. View it instead?").
