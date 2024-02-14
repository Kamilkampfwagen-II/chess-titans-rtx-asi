use std::error;
use std::ffi::{c_void, OsString};
use std::mem::size_of;
use std::os::windows::ffi::OsStringExt;

use crate::patch::*;
use windows::core;
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Memory::{
    VirtualProtect, PAGE_EXECUTE_READWRITE, PAGE_PROTECTION_FLAGS,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetClassNameW, GetWindowLongA, SetWindowLongA, SetWindowPos, ShowWindow, GWL_STYLE,
    SWP_NOZORDER, SW_RESTORE, WS_CAPTION, WS_MAXIMIZE, WS_MAXIMIZEBOX, WS_MINIMIZE, WS_SYSMENU,
    WS_THICKFRAME,
};

pub unsafe fn write_to<T>(address: u32, value: T) -> core::Result<()>
where
    T: Copy,
{
    let region = address as *mut T;

    let mut old_protect: PAGE_PROTECTION_FLAGS = Default::default();
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

pub fn set_fov(fov: f32) {
    // by https://github.com/adamplayer
    const FOV_OFFSET: u32 = 0x13100c;

    let _ = unsafe { write_to(get_address_by_offset(FOV_OFFSET), fov) };
}

pub fn set_altitude(altitude: f32) {
    // by https://github.com/adamplayer
    const ALT_OFFSET: u32 = 0x131008;

    let _ = unsafe { write_to(get_address_by_offset(ALT_OFFSET), altitude) };
}

pub fn get_window_class_name(hwnd: HWND) -> Option<OsString> {
    let mut class_name: Vec<u16> = vec![0; 256];
    let result = unsafe { GetClassNameW(hwnd, &mut class_name) };
    if result == 0 {
        return None;
    }

    let length = result as usize;
    class_name.truncate(length);
    Some(OsString::from_wide(&class_name))
}

pub fn disable_maximize(hwnd: HWND) {
    // Un-maximize the window
    unsafe { ShowWindow(hwnd, SW_RESTORE) };

    let mut l_style = unsafe { GetWindowLongA(hwnd, GWL_STYLE) };
    l_style &= !(WS_MAXIMIZEBOX).0 as i32;
    unsafe { SetWindowLongA(hwnd, GWL_STYLE, l_style) };
}

pub fn make_borderless_fullscreen(hwnd: HWND, res: [u32; 2]) -> core::Result<()> {
    let mut l_style = unsafe { GetWindowLongA(hwnd, GWL_STYLE) };

    // Make borderless
    l_style &= !(WS_CAPTION | WS_THICKFRAME | WS_MINIMIZE | WS_MAXIMIZE | WS_SYSMENU).0 as i32;
    unsafe { SetWindowLongA(hwnd, GWL_STYLE, l_style) };

    // Move to top-left, resize to the given resolution
    unsafe {
        SetWindowPos(
            hwnd,
            HWND(0),
            0,
            0,
            res[0] as i32,
            res[1] as i32,
            SWP_NOZORDER,
        )
    }
}

pub fn get_window_monitor_res(hwnd: HWND) -> Option<[u32; 2]> {
    let monitor = unsafe { MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST) };
    let mut monitor_info = MONITORINFO {
        cbSize: size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };

    let result = unsafe { GetMonitorInfoW(monitor, &mut monitor_info) };
    if !result.as_bool() {
        return None;
    }

    let width = monitor_info.rcMonitor.right - monitor_info.rcMonitor.left;
    let height = monitor_info.rcMonitor.bottom - monitor_info.rcMonitor.top;

    Some([width as u32, height as u32])
}
