@echo off
cargo build --release --target=i686-pc-windows-msvc
copy /Y target\i686-pc-windows-msvc\release\chess_titans_rtx.dll target\i686-pc-windows-msvc\release\chess_titans_rtx.asi > nul
copy /Y target\i686-pc-windows-msvc\release\chess_titans_rtx.dll "D:\Games\Chess Titans RTX\chess_titans_rtx.asi" > nul