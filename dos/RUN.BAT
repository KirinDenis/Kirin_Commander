@echo off
rem Kirin Commander on DOS. CWSDPMI with no arguments stays for exactly one
rem program and unloads itself after it.
rem
rem     RUN          the commander
rem     RUN SHOT     one frame into SHOT.BIN and SHOT.TXT, then out
rem     RUN KEYS     DOSBox-X only: AUTOTYPE presses keys, F12 keeps a frame
rem                  in SHOTS.TXT after each, F10 and Yes leave
rem     RUN MENUS    the same through Help, System Info and Find file
rem     RUN LABEL    the same: Volume label KCTEST on C:, then System Info
rem     RUN STEP     the commander as a page's DOS PC starts it: /STEP, it
rem                  leaves RUNPROG.BAT and is started again; `ver` typed,
rem                  then the history, a frame of each in SHOTS.TXT
rem     RUN x EXIT   ...and close the emulator (for RUNWIN.BAT)
rem
rem KC.OUT says what the batch saw: the code the commander left with.
if "%OS%"=="Windows_NT" goto windows
echo before %1 > KC.OUT
if "%1"=="STEP" goto step
CWSDPMI.EXE
if "%1"=="SHOT" KC.COM /SHOT >> KC.OUT
if "%1"=="KEYS" AUTOTYPE -w 3 -p 0.5 f12 f9 f12 esc f7 t m p enter f12 f8 enter f12 f10 f12 y
if "%1"=="KEYS" KC.COM >> KC.OUT
if "%1"=="MENUS" AUTOTYPE -w 3 -p 0.5 f12 f1 f12 esc f9 right right i f12 enter f9 right right f k c enter f12 esc f12 f10 y
if "%1"=="MENUS" KC.COM >> KC.OUT
if "%1"=="LABEL" AUTOTYPE -w 3 -p 0.5 f9 right right v f12 k c t e s t f12 enter f12 f9 right right i f12 enter f10 y
if "%1"=="LABEL" KC.COM >> KC.OUT
if "%1"=="" KC.COM
if errorlevel 1 echo errorlevel 1 or more >> KC.OUT
echo after >> KC.OUT
if "%2"=="EXIT" exit
goto end
:step
rem Not into KC.OUT here: the commander shows the command line through DOS.
AUTOTYPE -w 3 -p 0.5 v e r enter , , , , , , , , f12 f9 right right h f12 esc f9 right right p f12 f9 right right p f10 y
:again
CWSDPMI.EXE
KC.COM /STEP C:\ C:\DOS
if not exist RUNPROG.BAT goto stepped
echo stepped out >> KC.OUT
call RUNPROG.BAT
del RUNPROG.BAT
goto again
:stepped
if exist KC.SAV echo KC.SAV left behind >> KC.OUT
echo after >> KC.OUT
if "%2"=="EXIT" exit
goto end
:windows
call "%~dp0RUNWIN.BAT" %1 %2
:end
