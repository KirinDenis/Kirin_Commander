@echo off
rem The web parts of OWLOSUI this page draws with, copied - not linked -
rem as WIRE CITY's pull-owlosui.ps1 does (docs/09-decisions.md, D12): the
rem canvas Screen and Frame (owlosui.js, touch.js), and the browser's
rem storage (files/browser.js). Run it again after OWLOSUI changes them,
rem and read the difference before keeping it.
setlocal
set FROM=%~dp0..\..\OWLOSUI\lib\js
set TO=%~dp0owlosui
if not exist "%TO%\files" mkdir "%TO%\files"
copy /y "%FROM%\owlosui.js" "%TO%\" >nul || exit /b 1
copy /y "%FROM%\touch.js" "%TO%\" >nul || exit /b 1
copy /y "%FROM%\files\browser.js" "%TO%\files\" >nul || exit /b 1
copy /y "%~dp0..\..\OWLOSUI\LICENSE" "%TO%\LICENSE.txt" >nul || exit /b 1
echo OWLOSUI's web parts copied into web\owlosui.
