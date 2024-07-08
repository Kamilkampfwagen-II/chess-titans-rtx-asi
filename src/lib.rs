mod patch;

mod patches;
use patches::*;

#[macro_use]
mod helper;
use helper::*;

mod config;
use config::conf::{self, Config, Unwrap};

use std::path::Path;
use std::ptr::null;
use std::sync::Arc;
use std::thread;

use windows::core::PCWSTR;
use windows::Win32::Foundation::{BOOL, HANDLE};
use windows::Win32::System::Console::AllocConsole;
use windows::Win32::System::SystemServices::{
    DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH, DLL_THREAD_ATTACH, DLL_THREAD_DETACH,
};
use windows::Win32::UI::WindowsAndMessaging::FindWindowW;

fn window_watcher(config: &Config) {
    let class_name = to_c_utf16("ChessWindowClass");
    let lpclassname = PCWSTR(class_name.as_ptr());
    let lpwindowname = PCWSTR(null());
    let hwnd = {
        loop {
            if let Ok(v) = unsafe { FindWindowW(lpclassname, lpwindowname) } {
                break v;
            }
            sleep_ms!(1000);
        }
    };

    println!(
        "[INFO] - Found Chess Titans window with handle {}",
        hwnd.0 as isize
    );

    disable_maximize(hwnd);
    println!("[INFO] - Disabled maximize button"); // Also un-maximizes the window

    if config["fullscreen"].unwrap() {
        let mut width: u32 = config["width"].unwrap();
        let mut height: u32 = config["height"].unwrap();
        if width == 0 || height == 0 {
            let res = get_window_monitor_res(hwnd)
                .expect("Failed to get display resolution, don't you have a monitor??");
            width = res[0];
            height = res[1];
        }

        make_borderless_fullscreen(hwnd, [width, height])
            .expect("Failed to enable fullscreen windowed. Whats going on here?");

        println!("[INFO] - Enabled borderless window");
    }
}

fn settings_watcher() {
    let patch = GRAPHICS_LEVEL_3.first().unwrap();
    let patch_address = get_address_by_offset(patch.offset);

    loop {
        if unsafe { read_from::<u8>(patch_address) } != patch.new {
            apply_and_report(&GRAPHICS_LEVEL_3, false, "Revert graphics level to 3")
        }
        sleep_ms!(1000);
    }
}

fn main() {
    // Read the config and clone references for new threads
    let config_0 = Arc::new(conf::read(Path::new("chess_titans_rtx.conf")));
    let config_1 = Arc::clone(&config_0);

    // Attach a console so we can print stuff
    if config_0["console"].unwrap() {
        let _ = unsafe { AllocConsole() };
    }
    println!("Welcome to Chess Titans RTX");

    // Thank you Adam :)
    if config_0["constant_tick_patch"].unwrap() {
        apply_and_report(&CONSTANT_TICK, true, "Constant Tick - by AdamPlayer");
    }
    set_fov(config_0["fov"].unwrap());
    set_altitude(config_0["altitude"].unwrap());

    // Continue with new threads to unblock the main thread
    if config_0["settings_override"].unwrap() {
        thread::spawn(settings_watcher);
    }
    thread::spawn(move || window_watcher(&config_1));
}

#[allow(unused_variables)]
#[no_mangle]
extern "system" fn DllMain(dll_module: HANDLE, call_reason: u32, lpv_reserverd: &u32) -> BOOL {
    match call_reason {
        DLL_PROCESS_ATTACH => {
            main();
            BOOL(1)
        }

        DLL_PROCESS_DETACH => BOOL(1),

        DLL_THREAD_ATTACH => BOOL(1),

        DLL_THREAD_DETACH => BOOL(1),

        _ => BOOL(1),
    }
}
