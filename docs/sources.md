# Sources

Facts in these documents come from the sources below; nothing here is
copied from them. Where they disagree, [09-decisions.md](09-decisions.md)
says what is chosen.

## The programs themselves

- **Volkov Commander 4.05**, its assembler source - menus (`VCMENU.INC`),
  key bar and its Shift/Ctrl/Alt rows (`VCSUB2.INC`), colour tables
  (`VC.ASM`, `Palets`), dialogs (`VCCOPY.INC`, `VCF1.INC`, `VCFIND.INC`),
  viewer and editor (`VCVIEW.INC`, `VCEDIT.INC`), panels and info
  (`VCSUB3.INC`), the key table (`VC.ASM`, `FncTab`):
  https://github.com/ddanila/vc
- **Far Manager 3**, its English help, `FarEng.hlf` (installed with Far):
  topics Panels, PanelViewModes, PanelCmd, FuncCmd, CmdLineCmd, DriveDlg,
  SelectFiles, FastFind, CopyFiles, CopyAskOverwrite, MakeFolder,
  DeleteFile, Menus, LeftRightMenu, FilesMenu, CmdMenu, OptMenu, Viewer,
  Editor, ColorThemes, InterfSettings, PanelSettings.
- **Midnight Commander**, its manual and source - menus
  (`src/filemanager/filemanager.c`), listing formats and the size column
  (`panel.c`), the default skin (`misc/skins/default.ini`):
  https://source.midnight-commander.org/man/mc.html,
  https://github.com/MidnightCommander/mc
- **Norton Commander** - no digital manual of NC 4 or 5 was found; from
  descriptions and screenshots:
  https://knowware.dk/dos56/norton-commander-1.shtml,
  https://knowware.dk/dos56/norton-commander-2.shtml,
  https://ilyabirman.net/meanwhile/all/ui-museum-norton-commander-5-0/,
  https://www.edu.yar.ru/russian/cources/comp/nort.html,
  https://en.wikipedia.org/wiki/Norton_Commander

## On the standard

- Nikolai Bezroukov, the orthodox file manager standard:
  https://softpanorama.org/OFM/Standards/ofm_standard1999.shtml,
  and on Norton Commander's history:
  https://softpanorama.org/OFM/Paradigm/Ch03/norton_commander.shtml

## Under it

- **OWLOSUI**, the toolkit: https://github.com/KirinDenis/OWLOSUI - its
  file panel (`lib/core/src/files.rs`), its three commanders, its file
  sources for the browser (`lib/js/files/`), its DOS resident.
- **WIRE CITY**'s console and drive table - the browser's terminal, its
  commands, its mounts and confirmations.
- DOS's INT 21h and INT 24h error codes; Win32 system error codes; POSIX
  `errno`; the File System and OPFS exceptions of the browsers.

## Still wanted

- Screenshots of NC 5 / VC 4 / Far for checking cell by cell: Brief and
  Full, the drive menu, F9's menus, the copy dialog, the editor.
- A text of NC 4/5's own menus, which the sources above disagree about.
