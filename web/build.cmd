@echo off
rem Kirin Commander for the browser: web\kc.wasm beside kc.js. Needs the
rem wasm32 target and Binaryen's wasm-opt (for asyncify), once:
rem     rustup target add wasm32-unknown-unknown
rem     cargo install wasm-opt
rem asyncify lets the module sleep inside an Fs call until the browser's
rem storage answers its promise (src/lib.rs): only kc.host may sleep.
setlocal
cd /d %~dp0..
cargo build -p kc-web --target wasm32-unknown-unknown --release || exit /b 1
wasm-opt -O2 --all-features --asyncify --pass-arg=asyncify-imports@kc.host target\wasm32-unknown-unknown\release\kc_web.wasm -o web\kc.wasm || exit /b 1
for %%F in (web\kc.wasm) do echo web\kc.wasm: %%~zF bytes
