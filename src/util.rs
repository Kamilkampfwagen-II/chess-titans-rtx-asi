use std::error;
use std::f32::consts::PI;
use std::ffi::c_void;
use std::mem::size_of;

use crate::patch::*;
use windows::core;
use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Memory::{
    VirtualProtect, PAGE_EXECUTE_READWRITE, PAGE_PROTECTION_FLAGS,
};
use windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState;
use windows::Win32::UI::WindowsAndMessaging::{
    GetWindowLongA, GetWindowRect, SetWindowLongA, SetWindowPos, ShowWindow, GWL_STYLE,
    SWP_NOZORDER, SW_RESTORE, WS_CAPTION, WS_MAXIMIZE, WS_MAXIMIZEBOX, WS_MINIMIZE, WS_SYSMENU,
    WS_THICKFRAME,
};

macro_rules! sleep_ms {
    ($ms:expr) => {
        std::thread::sleep(std::time::Duration::from_millis($ms))
    };
}

pub unsafe fn write_to<T>(address: u32, value: T) -> core::Result<()>
where
    T: Copy,
{
    let region = address as *mut T;

    let mut old_protect = PAGE_PROTECTION_FLAGS::default();
    unsafe {
        // Disable virtual page protection
        VirtualProtect(
            region as *const c_void,
            size_of::<T>(),
            PAGE_EXECUTE_READWRITE,
            &mut old_protect,
        )?;

        // Write
        *region = value;

        // Restore virtual page protection
        VirtualProtect(
            region as *const c_void,
            size_of::<T>(),
            old_protect,
            &mut old_protect,
        )?;
    };

    Ok(())
}

pub unsafe fn read_from<T>(address: u32) -> T
where
    T: Copy,
{
    let region = address as *mut T;
    unsafe { *region }
}

pub fn get_address_by_offset(offset: u32) -> u32 {
    unsafe { GetModuleHandleW(None).unwrap() }.0 as u32 + offset // I don't see a reason for this to fail
}

pub fn apply_patch(patch_set: &[Patch], verify: bool) -> Result<(), Box<dyn error::Error>> {
    let base_address = get_address_by_offset(0);

    if verify {
        for patch in patch_set.iter() {
            let target_byte = unsafe { read_from::<u8>(base_address + patch.offset) };
            if target_byte == patch.org {
                continue;
            }

            return Err(Box::new(PatchError::ByteMismatch(
                patch.offset,
                patch.org,
                target_byte,
            )));
        }
    }

    for patch in patch_set.iter() {
        let patch_address = get_address_by_offset(patch.offset);
        unsafe { write_to(patch_address, patch.new) }?;
    }

    Ok(())
}

pub fn apply_and_report(patch_set: &[Patch], verify: bool, ok_msg: &str) {
    let result = apply_patch(patch_set, verify);
    match result {
        Ok(()) => println!("[INFO] - Applied patch: {}", ok_msg),
        Err(error) => println!("[FAIL] - {}", error),
    }
}

pub fn is_pressing(vkey: i32) -> bool {
    (unsafe { GetAsyncKeyState(vkey) } < 0)
}

pub fn to_radians(degrees: f32) -> f32 {
    degrees * PI / 180.0
}

pub fn set_fov(degrees: f32, aspect_ratio: f32) {
    // by https://github.com/adamplayer
    const FOV_OFFSET: u32 = 0x13100c;

    let _ = unsafe { write_to(get_address_by_offset(FOV_OFFSET), degrees) };

    let fov = to_radians(degrees);
    let altitude = ((PI - fov) / 2.0).tan() * aspect_ratio * 10.0 + 28.652; // Don't believe me?
    println!("FOV: {}, Altitude: {}", degrees, altitude);
    set_altitude(altitude);
}

pub fn set_altitude(altitude: f32) {
    // by https://github.com/adamplayer
    const ALT_OFFSET: u32 = 0x131008;

    let _ = unsafe { write_to(get_address_by_offset(ALT_OFFSET), altitude) };
}

pub fn disable_maximize(hwnd: HWND) {
    // Un-maximize the window
    let _ = unsafe { ShowWindow(hwnd, SW_RESTORE) };

    let mut l_style = unsafe { GetWindowLongA(hwnd, GWL_STYLE) };
    l_style &= !(WS_MAXIMIZEBOX).0 as i32;
    unsafe { SetWindowLongA(hwnd, GWL_STYLE, l_style) };
}

pub fn get_window_rect(hwnd: HWND) -> core::Result<RECT> {
    let mut rect = RECT::default();
    unsafe { GetWindowRect(hwnd, &mut rect) }?;

    Ok(rect)
}

pub fn set_window_properties(hwnd: HWND, rect: RECT, style: i32) -> core::Result<()> {
    unsafe { SetWindowLongA(hwnd, GWL_STYLE, style) };

    unsafe {
        SetWindowPos(
            hwnd,
            HWND::default(),
            rect.left,
            rect.top,
            rect.right - rect.left,
            rect.bottom - rect.top,
            SWP_NOZORDER,
        )
    }
}

pub fn get_window_style(hwnd: HWND) -> Result<i32, ()> {
    let style = unsafe { GetWindowLongA(hwnd, GWL_STYLE) };
    if style == 0 {
        Err(())
    } else {
        Ok(style)
    }
}

pub fn get_borderless_style(style: i32) -> i32 {
    style & !(WS_CAPTION | WS_THICKFRAME | WS_MINIMIZE | WS_MAXIMIZE | WS_SYSMENU).0 as i32
}

pub fn get_window_fullscreen_rect(hwnd: HWND) -> Option<RECT> {
    let monitor = unsafe { MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST) };
    let mut monitor_info = MONITORINFO {
        cbSize: size_of::<MONITORINFO>() as u32,
        ..MONITORINFO::default()
    };

    if !unsafe { GetMonitorInfoW(monitor, &mut monitor_info) }.as_bool() {
        return None;
    }

    Some(monitor_info.rcMonitor)
}

pub fn to_c_utf16(str: &str) -> Box<[u16]> {
    str.encode_utf16().chain(Some(0)).collect()
}
