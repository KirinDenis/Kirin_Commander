@echo off
rem Kirin Commander for DOS: KC.BIN (the commander, a flat 32-bit binary)
rem from Rust nightly with build-std, and its loader KC.COM from FASM.
rem Needs, once:
rem     rustup toolchain install nightly --component rust-src
rem and a FASM for Windows: set FASM to it, or put FASM.EXE on the PATH.
setlocal
cd /d %~dp0
if "%FASM%"=="" set FASM=fasm.exe
if not exist "%FASM%" if exist C:\DOSFiles\wire-city-2\TOOLS\FASM\WIN\FASM.EXE set FASM=C:\DOSFiles\wire-city-2\TOOLS\FASM\WIN\FASM.EXE
cargo +nightly build --release || exit /b 1
copy /y target\i386-owlos-none\release\kc KC.BIN >nul || exit /b 1
"%FASM%" KC.ASM KC.COM || exit /b 1
for %%F in (KC.BIN) do echo KC.BIN: %%~zF bytes
echo Run RUN.BAT under DOS (DOSBox-X or a real machine), or RUNWIN.BAT here.
