# release.ps1 - the downloads, into dist\, from what is built:
#
#     kc-dos.zip           KC.COM, KC.BIN, CWSDPMI.EXE and its notice
#     kc-windows-x64.zip   kc.exe
#
# Build first: dos\build.cmd, and cargo build -p kc-term --release. A binary
# older than the Rust it is built from is refused, so a release cannot ship
# yesterday's commander. Then put the two zips on a GitHub release.
$ErrorActionPreference = "Stop"
Add-Type -AssemblyName System.IO.Compression
Add-Type -AssemblyName System.IO.Compression.FileSystem
$root = $PSScriptRoot
$dist = Join-Path $root "dist"

$newest = Get-ChildItem -Recurse -Include *.rs, Cargo.toml (Join-Path $root "kc\src"), (Join-Path $root "dos\src"), (Join-Path $root "term\src") |
          Sort-Object LastWriteTime -Descending | Select-Object -First 1
function Built([string]$path) {
  $f = Get-Item (Join-Path $root $path) -ErrorAction SilentlyContinue
  if (-not $f) { throw "No $path - build it first." }
  if ($newest.LastWriteTime -gt $f.LastWriteTime) { throw "$path is older than $($newest.FullName) - build it again." }
  $f
}

# A text file as DOS and Notepad show it: CRLF.
function Text([string]$s) { [Text.Encoding]::ASCII.GetBytes(($s -replace "`r?`n", "`r`n")) }

function Zip([string]$name, [hashtable]$entries) {
  $path = Join-Path $dist $name
  if (Test-Path $path) { Remove-Item $path }
  $zip = [IO.Compression.ZipFile]::Open($path, 'Create')
  try {
    foreach ($k in ($entries.Keys | Sort-Object)) {
      $v = $entries[$k]
      $data = if ($v -is [IO.FileInfo]) { [IO.File]::ReadAllBytes($v.FullName) } else { $v }
      $e = $zip.CreateEntry($k, 'Optimal')
      $s = $e.Open(); $s.Write($data, 0, $data.Length); $s.Close()
    }
  } finally { $zip.Dispose() }
  "{0,-20} {1,9} bytes" -f $name, (Get-Item $path).Length
}

$licence = [IO.File]::ReadAllText((Join-Path $root "LICENSE"))
New-Item -ItemType Directory -Force $dist | Out-Null

Zip "kc-dos.zip" @{
  "KC.COM"      = Built "dos\KC.COM"
  "KC.BIN"      = Built "dos\KC.BIN"
  "CWSDPMI.EXE" = Get-Item (Join-Path $root "dos\CWSDPMI.EXE")
  "CWSDPMI.DOC" = Get-Item (Join-Path $root "dos\cwsdpmi.doc")
  "LICENSE.TXT" = Text $licence
  "README.TXT"  = Text @"
Kirin Commander for DOS - for a 386 or later, with extended memory.

    CWSDPMI
    KC

CWSDPMI is the DPMI host the commander runs under; with no arguments it
stays for exactly one program. KC.COM loads KC.BIN into extended memory.
F1 shows the keys, F9 the menu, F10 leaves. Shift+F9 saves the setup into
KC.INI beside KC.COM.

To give a program run from the commander all of DOS's memory, start it
from a batch file with /STEP - it steps out to run the program and is
started again where it was:

    :again
    CWSDPMI
    KC /STEP
    if not exist RUNPROG.BAT goto done
    call RUNPROG.BAT
    del RUNPROG.BAT
    goto again
    :done

https://github.com/KirinDenis/Kirin_Commander
"@
}

Zip "kc-windows-x64.zip" @{
  "kc.exe"      = Built "target\release\kc.exe"
  "LICENSE.txt" = Text $licence
  "README.txt"  = Text @"
Kirin Commander for Windows - run kc.exe in a console window (Windows
Terminal, or cmd). F1 shows the keys, F9 the menu, F10 leaves. Save setup
(Shift+F9) writes %APPDATA%\KC\kc.ini.

https://github.com/KirinDenis/Kirin_Commander
"@
}
