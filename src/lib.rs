mod bridge_api;
use bridge_api::{remix_api::*, PfnDrawLightInstance};

mod config;
use config::conf::{self, Config, Unwrap, Value};

mod patch;

mod patches;
use patches::*;

#[macro_use]
mod util;
use util::*;

use std::collections::HashMap;
use std::mem::MaybeUninit;
use std::path::Path;
use std::ptr::null;
use std::sync::Mutex;
use std::thread;

use windows::core::PCWSTR;
use windows::Win32::Foundation::{BOOL, HANDLE, RECT};
use windows::Win32::System::Console::AllocConsole;
use windows::Win32::System::SystemServices::{
    DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH, DLL_THREAD_ATTACH, DLL_THREAD_DETACH,
};
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::WindowsAndMessaging::FindWindowW;

static LIGHT_HANDLES: Mutex<Vec<u64>> = Mutex::new(Vec::new());
static mut DRAW_LIGHT_INSTANCE: Mutex<MaybeUninit<PfnDrawLightInstance>> =
    Mutex::new(MaybeUninit::uninit());

extern "C" fn on_frame_end() {
    let light_handles = LIGHT_HANDLES.lock().unwrap();
    let draw_light_inctance = unsafe { DRAW_LIGHT_INSTANCE.lock().unwrap().assume_init() };

    for handle in light_handles.iter() {
        draw_light_inctance(*handle);
    }
}

fn bridge_api_madness() {
    sleep_ms!(5000);

    let api = bridge_api::initialize().expect("Failed to initialize Bridge API!");

    unsafe {
        let mut draw_light_instance = DRAW_LIGHT_INSTANCE.lock().unwrap();
        draw_light_instance.write(api.DrawLightInstance.assume_init());
    }

    println!("RegisterDevice");
    unsafe { api.RegisterDevice.assume_init()() };

    println!("Register EndFrame callback");
    unsafe { api.RegisterEndSceneCallback.assume_init()(on_frame_end) };

    let mut i: u64 = 0;
    loop {
        if is_pressing(VK_Q.0 as i32) {
            i += 1;

            let light_info = LightInfo {
                s_type: StructType::LightInfo,
                hash: i,
                radiance: Float3D {
                    x: 10.0,
                    y: 10.0,
                    z: 10.0,
                },
            };

            let distant_light_info = LightInfoDistantExt {
                s_type: StructType::LightInfoDistantExt,
                direction: Float3D {
                    x: -0.344255,
                    y: -0.938876,
                    z: 0.0,
                }, // Fallback light with this direction is visible at all times
                angular_diameter_degrees: 1.0,
            };

            let handle =
                unsafe { api.CreateDistantLight.assume_init()(&light_info, &distant_light_info) };
            println!(
                "Created a distant light with handle: {} and hash: {}",
                handle, &light_info.hash
            );

            let mut light_handles = LIGHT_HANDLES.lock().unwrap();
            light_handles.push(handle);

            sleep_ms!(500);
        }

        sleep_ms!(500);
    }
}

fn late_init_window_creation(config: Config) {
    let hwnd = {
        let class_name = to_c_utf16("ChessWindowClass");
        loop {
            if let Ok(v) = unsafe { FindWindowW(PCWSTR(class_name.as_ptr()), PCWSTR(null())) } {
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

    thread::spawn(bridge_api_madness);

    let mut style = 0;
    let mut rect: RECT = RECT::default();
    let mut is_fullscreen = false;
    let vkey_fullscreen: i32 = config["vkey_fullscreen"].unwrap();

    let mut aspect_ratio = {
        let rect = get_window_fullscreen_rect(hwnd)
            .expect("Failed to get display resolution, don't you have a monitor??");
        (rect.right - rect.left) as f32 / (rect.bottom - rect.top) as f32
    };
    let default_fov: f32 = config["fov"].unwrap();
    let mut fov = default_fov;
    let vkey_fov_up: i32 = config["vkey_fov_up"].unwrap();
    let vkey_fov_down: i32 = config["vkey_fov_down"].unwrap();
    let vkey_fov_reset: i32 = config["vkey_fov_reset"].unwrap();
    set_fov(default_fov, aspect_ratio);

    // let default_alt: f32 = config["altitude"].unwrap();
    // let mut alt = default_alt;
    // let vkey_alt_up: i32 = config["vkey_alt_up"].unwrap();
    // let vkey_alt_down: i32 = config["vkey_alt_down"].unwrap();
    // let vkey_alt_reset: i32 = config["vkey_alt_reset"].unwrap();

    const SLEEP_MS: u64 = 50;
    const STEP: f32 = 100.0 * SLEEP_MS as f32 / 1000.0;

    loop {
        // FOV
        if is_pressing(vkey_fov_up) {
            fov = (fov + STEP).min(175.0);
            set_fov(fov, aspect_ratio);
        } else if is_pressing(vkey_fov_down) {
            fov = (fov - STEP).max(5.0);
            set_fov(fov, aspect_ratio);
        } else if is_pressing(vkey_fov_reset) {
            fov = default_fov;
            set_fov(fov, aspect_ratio)
        }

        // // Camera Altitude
        // if is_pressing(vkey_alt_up) {
        //     alt = (alt + STEP).min(1000.0);
        //     set_altitude(alt);
        // } else if is_pressing(vkey_alt_down) {
        //     alt = (alt - STEP).max(10.0);
        //     set_altitude(alt);
        // } else if is_pressing(vkey_alt_reset) {
        //     alt = default_alt;
        //     set_altitude(alt);
        // }

        // Borderless Fullscreen
        if is_pressing(vkey_fullscreen) {
            if is_fullscreen {
                set_window_properties(hwnd, rect, style)
                    .expect("Failed to revert back window properties!");
            } else {
                style = get_window_style(hwnd).expect("Failed to get window properties!");
                rect = get_window_rect(hwnd).expect("Failed to get window size!");
                let fullscreen_rect = get_window_fullscreen_rect(hwnd).unwrap();

                set_window_properties(hwnd, fullscreen_rect, get_borderless_style(style))
                    .expect("Failed to set window properties!");

                aspect_ratio = (fullscreen_rect.right - fullscreen_rect.left) as f32
                    / (fullscreen_rect.bottom - fullscreen_rect.top) as f32;
            }

            is_fullscreen = !is_fullscreen;
        }

        sleep_ms!(SLEEP_MS);
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
    let default_config = HashMap::from([
        ("console", Value::Bool(true)),
        ("fov", Value::F32(30.0)),
        ("constant_tick_patch", Value::Bool(true)),
        ("settings_override", Value::Bool(true)),
        ("vkey_fov_up", Value::I32(VK_F.0 as i32)),
        ("vkey_fov_down", Value::I32(VK_V.0 as i32)),
        ("vkey_fov_reset", Value::I32(VK_R.0 as i32)),
        ("vkey_alt_up", Value::I32(VK_G.0 as i32)),
        ("vkey_alt_down", Value::I32(VK_B.0 as i32)),
        ("vkey_alt_reset", Value::I32(VK_T.0 as i32)),
        ("vkey_fullscreen", Value::I32(VK_RETURN.0 as i32)),
    ]);

    let config = conf::read(Path::new("chess_titans_rtx.conf"), default_config);

    // Attach a console so we can print stuff
    if config["console"].unwrap() {
        let _ = unsafe { AllocConsole() };
    }
    println!("Welcome to Chess Titans RTX");

    // Thank you Adam :)
    if config["constant_tick_patch"].unwrap() {
        apply_and_report(&CONSTANT_TICK, true, "Constant Tick - by AdamPlayer");
    }

    // Continue with new threads to unblock the main thread
    if config["settings_override"].unwrap() {
        thread::spawn(settings_watcher);
    }
    thread::spawn(move || late_init_window_creation(config));
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
