pub mod helper {
    use std::error;
    use std::ffi::c_void;
    use std::mem::size_of;
    use std::ptr::null;
    
    use windows::core::{self, PCWSTR};
    use windows::Win32::Foundation::HWND;
    use windows::Win32::Graphics::Gdi::{DISPLAY_DEVICEW, EnumDisplayDevicesW, GetDeviceCaps, CreateDCW, HORZRES, VERTRES};
    use windows::Win32::System::Memory::{PAGE_PROTECTION_FLAGS, VirtualProtect, PAGE_EXECUTE_READWRITE};
    use windows::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows::Win32::UI::WindowsAndMessaging::{SET_WINDOW_POS_FLAGS, GetClassNameA, GetWindowLongA, GetWindowTextLengthW, SetWindowLongA, ShowWindow, SetWindowPos, WS_BORDER, WS_CAPTION, WS_THICKFRAME, WS_MINIMIZE, WS_MAXIMIZE, WS_SYSMENU, WS_MAXIMIZEBOX, HWND_NOTOPMOST, HWND_BOTTOM, GWL_STYLE, SW_RESTORE};
    use crate::patch::patch::*;


    pub unsafe fn write_to<T>(address: u32, value: T) -> Result<(), core::Error>
    where
        T: Copy,
    {
        let region = address as *mut T;

        let mut old_protect: PAGE_PROTECTION_FLAGS = Default::default();
        unsafe {
            // Disable virtual page protection
            VirtualProtect(region as *const c_void, size_of::<T>(), PAGE_EXECUTE_READWRITE, &mut old_protect)?;
    
            // Write
            *region = value;
    
            // Restore virtual page protection
            VirtualProtect(region as *const c_void, size_of::<T>(), old_protect, &mut old_protect)?;
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
        let h_parent_module = unsafe { GetModuleHandleW(None).unwrap() }; // I don't see a reason for this to fail

        h_parent_module.0 as u32 + offset
    }


    pub fn apply_patch(patch_set: &[Patch], verify: bool) -> Result<(), Box<dyn error::Error>> {
        if verify {
            for patch in patch_set.iter() {        
                let patch_address = get_address_by_offset(patch.offset) as *mut u8;
                let target_byte = unsafe { *patch_address };

                if !verify || target_byte== patch.org { continue; }

                return Err( Box::new(PatchError::ByteMismatch(patch.offset, patch.org, target_byte)) );
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


    pub fn get_window_class_name(hwnd: HWND) -> String {
        let length = unsafe { GetWindowTextLengthW(hwnd) + 1 };
        let mut lp_string: Vec<u8> = vec![0x8; length as usize];
        unsafe { GetClassNameA(hwnd, &mut lp_string) };
    
        String::from_utf8(lp_string).unwrap_or_else(|_| String::from("")).trim_end_matches('\0').to_string()
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
        let mut device = DISPLAY_DEVICEW::default();
        device.cb = size_of::<DISPLAY_DEVICEW>() as u32;

        let result = unsafe { EnumDisplayDevicesW(PCWSTR(null()), 0, &mut device, 0) };
        if !result.as_bool() { return None; }

        let device_name: Vec<u16> = device.DeviceName.iter().cloned().collect();
        let device_context = unsafe { CreateDCW(PCWSTR(null()), PCWSTR(device_name.as_ptr()), PCWSTR(null()), None) };

        let width = unsafe { GetDeviceCaps(device_context, HORZRES) } as u32;
        let height = unsafe { GetDeviceCaps(device_context, VERTRES) } as u32;

        Some([width, height])
    }


    pub fn move_window(hwnd: HWND) -> Result<(), core::Error> {
        unsafe { SetWindowPos(hwnd, HWND_NOTOPMOST, 0, 0, 0, 0, SET_WINDOW_POS_FLAGS(HWND_BOTTOM.0 as u32) ) }
    }

}