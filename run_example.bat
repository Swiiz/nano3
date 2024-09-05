@echo off

echo Compiling example with nightly toolchain and wasm compiler params...
cargo +nightly build --target wasm32-unknown-unknown --release -p example*
if %errorlevel% neq 0 exit /b %errorlevel%
echo ====================================================
echo Moving target/[...]/example.wasm to modules/example/.wasm...
move .\target\wasm32-unknown-unknown\release\example.wasm ./modules/example
if %errorlevel% neq 0 exit /b %errorlevel%
del .\modules\example\.wasm
ren .\modules\example\example.wasm .wasm
if %errorlevel% neq 0 exit /b %errorlevel%
echo ====================================================
echo Launching engine...
cargo run -p runner
if %errorlevel% neq 0 exit /b %errorlevel%