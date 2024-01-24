@echo off
@REM cargo build --release --target=x86_64-pc-windows-msvc
cargo build --release --target=i686-pc-windows-msvc
copy /Y target\i686-pc-windows-msvc\release\chess_titans_rtx.dll target\i686-pc-windows-msvc\release\chess_titans_rtx.asi > nul