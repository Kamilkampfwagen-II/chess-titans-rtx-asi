use std::error;
use std::ffi::{c_void, OsString};
use std::mem::size_of;
use std::os::windows::ffi::OsStringExt;
use std::ptr::null;

use crate::patch::*;
use windows::core::{self, PCWSTR};
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Gdi::{
    CreateDCW, EnumDisplayDevicesW, GetDeviceCaps, DISPLAY_DEVICEW, HORZRES, VERTRES,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Memory::{
    VirtualProtect, PAGE_EXECUTE_READWRITE, PAGE_PROTECTION_FLAGS,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetClassNameW, GetWindowLongA, SetWindowLongA,
    SetWindowPos, ShowWindow, GWL_STYLE, HWND_BOTTOM, HWND_NOTOPMOST, SET_WINDOW_POS_FLAGS,
    SW_RESTORE, WS_BORDER, WS_CAPTION, WS_MAXIMIZE, WS_MAXIMIZEBOX, WS_MINIMIZE, WS_SYSMENU,
    WS_THICKFRAME,
};

pub unsafe fn write_to<T>(address: u32, value: T) -> Result<(), core::Error>
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

pub fn make_borderless(hwnd: HWND) {
    let mut l_style = unsafe { GetWindowLongA(hwnd, GWL_STYLE) };

    if l_style & WS_BORDER.0 as i32 != 0 {
        l_style &= !(WS_CAPTION | WS_THICKFRAME | WS_MINIMIZE | WS_MAXIMIZE | WS_SYSMENU).0 as i32;
        unsafe { SetWindowLongA(hwnd, GWL_STYLE, l_style) };
    }
}

pub fn get_display_res() -> Option<[u32; 2]> {
    let mut device = DISPLAY_DEVICEW {
        cb: size_of::<DISPLAY_DEVICEW>() as u32,
        ..Default::default()
    };

    let result = unsafe { EnumDisplayDevicesW(PCWSTR(null()), 0, &mut device, 0) };
    if !result.as_bool() {
        return None;
    }

    let device_name: Vec<u16> = device.DeviceName.to_vec();
    let device_context = unsafe {
        CreateDCW(
            PCWSTR(null()),
            PCWSTR(device_name.as_ptr()),
            PCWSTR(null()),
            None,
        )
    };

    let width = unsafe { GetDeviceCaps(device_context, HORZRES) } as u32;
    let height = unsafe { GetDeviceCaps(device_context, VERTRES) } as u32;

    Some([width, height])
}

pub fn move_window(hwnd: HWND) -> Result<(), core::Error> {
    unsafe {
        SetWindowPos(
            hwnd,
            HWND_NOTOPMOST,
            0,
            0,
            0,
            0,
            SET_WINDOW_POS_FLAGS(HWND_BOTTOM.0 as u32),
        )
    }
}
