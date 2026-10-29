@echo off
rem Start DOSBox-X and run RUN.BAT inside it, the repository as drive C:.
rem Set DOSBOXX to your dosbox-x.exe, or put it where the default says.
rem
rem     RUNWIN          the commander, interactively
rem     RUNWIN SHOT     one frame into SHOT.BIN and SHOT.TXT, then DOSBox-X closes
rem     RUNWIN KEYS     keys typed by DOSBox-X, a frame after each in SHOTS.TXT
setlocal
cd /d %~dp0
if "%DOSBOXX%"=="" set DOSBOXX=C:\DOSFiles\dosbox-x\bin\x64\Release\dosbox-x.exe
if not exist "%DOSBOXX%" (
  echo dosbox-x.exe not found: set DOSBOXX to it.
  exit /b 1
)
for %%R in ("%~dp0..") do set ROOT=%%~fR
rem The TrueType output's built-in font has no double box lines and draws
rem the panels' frames single; Nouveau_IBM, shipped beside dosbox-x.exe, has
rem them. With a pixel output (surface, openglpp) this setting is unused.
set GO=-noautoexec -set "ttf font=Nouveau_IBM" %DOSBOXX_OPTS% -c "mount c %ROOT%" -c "c:" -c "cd \DOS"
if "%1"=="" goto interactive
del SHOT.BIN SHOT.TXT SHOTS.TXT 2>nul
"%DOSBOXX%" %GO% -c "RUN %1 EXIT" >nul 2>&1
exit /b 0
:interactive
"%DOSBOXX%" %GO% -c "RUN" >nul 2>&1
