mod patch;

mod patches;
use patches::patches::*;

mod helper;
use helper::helper::*;

mod config;
use config::conf;
use config::conf::Unwrap;

use std::collections::HashMap;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use windows::Win32::Foundation::{BOOL, HANDLE, HWND, LPARAM};
use windows::Win32::System::Console::AllocConsole;
use windows::Win32::System::SystemServices::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH, DLL_THREAD_ATTACH, DLL_THREAD_DETACH};
use windows::Win32::System::Threading::GetCurrentProcessId;
use windows::Win32::UI::WindowsAndMessaging::{GetWindowThreadProcessId, EnumWindows};


#[no_mangle]
extern "system" fn enum_windows_proc(hwnd: HWND, l_param: LPARAM) -> BOOL {
    let mut pid: u32 = 0;
    unsafe { GetWindowThreadProcessId(hwnd, Some(&mut pid as *mut u32)) };
    if pid != unsafe { GetCurrentProcessId() } { return BOOL(1); }

    let wclass_name = get_window_class_name(hwnd);
    if wclass_name != "ChessWindowC" { return  BOOL(1); }

    // This is the window we're looking for
    unsafe { *(l_param.0 as *mut HWND) = hwnd };
    return BOOL(0);
}


fn window_watcher(config: &HashMap<String, conf::Value>) {
    let mut hwnd: HWND = Default::default();
    loop {
        let _ = unsafe { EnumWindows(Some(enum_windows_proc),  LPARAM(&mut hwnd as *mut HWND as isize)) }; // This always returns Err() for some reason, so we ignore it
        if hwnd != Default::default() { break; }
    }
    println!("[OK] - Found Chess Titans window with handle {}", hwnd.0);

    disable_maximize(hwnd);
    println!("[OK] - Disabled maximize button"); // Also un-maximizes the window

    if config.get("fullscreen").unwrap().unwrap() {
        make_borderless(hwnd);
        let _ = move_window(hwnd);
        println!("[OK] - Enabled borderless window");
    }
}


fn settings_watcher() {
    let patch_address = get_address_by_offset(GRAPHICS_LEVEL_3.get(0).unwrap().offset);

    loop {
        if unsafe { read_from::<u8>(patch_address) } != GRAPHICS_LEVEL_3.get(0).unwrap().new {
            apply_and_report(&GRAPHICS_LEVEL_3, false, "Revert graphics level to 3")
        }
        thread::sleep(Duration::from_millis(1));
    }
}


fn res_watcher(config: &HashMap<String, conf::Value>) { // This is incredibly stupid, but I have no other solution for now
    const WIDTH_OFFSET: u32 = 0x131154;
    const HEIGHT_OFFSET: u32 = 0x131158;
    let width_address = get_address_by_offset(WIDTH_OFFSET);
    let height_address = get_address_by_offset(HEIGHT_OFFSET);

    let width = config.get("width").unwrap().unwrap();
    let height = config.get("height").unwrap().unwrap();

    let mut i = 0;
    loop {
        if unsafe { read_from::<u32>(width_address) } != width {
            i = 0;
            let _ = unsafe { write_to(width_address , width) };
        }

        if unsafe { read_from::<u32>(height_address) } != height {
            i = 0;
            let _ = unsafe { write_to(height_address, height) };
        }

        i += 1;
        if i > 1000 { break; } // No need to continue the loop after the window initialization
        thread::sleep(Duration::from_millis(1));
    }
}


fn main() {
    let config_0 = Arc::new(conf::read());
    let config_1 = Arc::clone(&config_0);
    let config_2 = Arc::clone(&config_0);

    // Attach a console so we can print stuff
    if config_0.get("console").unwrap().unwrap() {
        let _ = unsafe { AllocConsole() };
    }

    println!("Welcome to Chess Titans RTX");

    // Thank you Adam :)
    apply_and_report(&CONSTANT_TICK,    true,   "Constant Tick - by AdamPlayer");

    // Continue with new threads to unblock the main thread
    thread::spawn(settings_watcher);
    thread::spawn(move || res_watcher(&*config_2)); 
    thread::spawn(move || window_watcher(&*config_1));
}


#[allow(unused_variables)]
#[no_mangle]
extern "system" fn DllMain(
    dll_module: HANDLE,
    call_reason: u32,
    lpv_reserverd: &u32,
) -> BOOL {
    match call_reason {
        DLL_PROCESS_ATTACH => {
            main();
            return BOOL(1)
        }

        DLL_PROCESS_DETACH => {
            return BOOL(1)
        }
        
        DLL_THREAD_ATTACH => {
            return BOOL(1)
        }
        
        DLL_THREAD_DETACH => {
            return BOOL(1)
        }
        
        _ => {
            return BOOL(1)
        }
    }
}