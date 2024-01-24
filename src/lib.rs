mod patch;
use patch::patch::*;

mod patches;
use patches::patches::*;

use std::ffi::c_void;
use std::mem::size_of;
use std::thread;
use std::time::Duration;
use std::error::Error;

use windows::Win32::Foundation::{BOOL, HANDLE};
use windows::Win32::System::Console::AllocConsole;
use windows::Win32::System::Memory::{PAGE_PROTECTION_FLAGS, VirtualProtect, PAGE_EXECUTE_READWRITE};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::SystemServices::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH, DLL_THREAD_ATTACH, DLL_THREAD_DETACH};


fn apply_patch(patch_set: &[Patch], verify: bool) -> Result<(), Box<dyn Error>> {
    let h_parent_module = unsafe { GetModuleHandleW(None)? };

    if verify {
        for patch in patch_set.iter() {        
            let patch_address = (h_parent_module.0 + patch.offset as isize) as *mut u8;
            let target_byte = unsafe { *patch_address };

            if !verify || target_byte== patch.org { continue; }

            return Err( Box::new(PatchError::ByteMismatch(patch.offset, patch.org, target_byte)) );
        }
    }

    for patch in patch_set.iter() {        
        let patch_address = (h_parent_module.0 + patch.offset as isize) as *mut u8;

        let mut old_protect: PAGE_PROTECTION_FLAGS = Default::default();
        unsafe {
            // Disable virtual page protection
            VirtualProtect(patch_address as *const c_void, size_of::<u8>(), PAGE_EXECUTE_READWRITE, &mut old_protect)?;

            // Write the individual bytes
            *patch_address = patch.new;

            // Restore virtual page protection
            VirtualProtect(patch_address as *const c_void, size_of::<u8>(), old_protect, &mut old_protect)?;
        };
    }

    Ok(())
}


fn settings_watcher() {
    let h_parent_module = unsafe { GetModuleHandleW(None).unwrap() };
    let patch_address = (h_parent_module.0 + GRAPHICS_LEVEL_3.get(0).unwrap().offset as isize) as *mut u8;
    loop {
        if unsafe { *patch_address } != GRAPHICS_LEVEL_3.get(0).unwrap().new {
            let result = apply_patch(&GRAPHICS_LEVEL_3, false);
            match result {
                Ok(()) => println!("[OK] - Revert graphics level to 3"),
                Err(error) => println!("[FAIL] - {}", error),
            }
        }
        thread::sleep(Duration::from_millis(1));
    }
}


fn main() {
    // Attach a console so we can print stuff
    unsafe { AllocConsole().expect("Failed to allocate console!") }

    println!("Welcome to Chess Titans RTX");

    let result = apply_patch(&GRAPHICS_LEVEL_3, false);
    match result {
        Ok(()) => println!("[OK] - Applied patch: GRAPHICS_LEVEL_3"),
        Err(error) => println!("[FAIL] - {}", error),
    }

    let result = apply_patch(&CONSTANT_TICK, true);
    match result {
        Ok(()) => println!("[OK] - Applied patch: CONSTANT_TICK - by AdamPlayer"),
        Err(error) => println!("[FAIL] - {}", error),
    }

    let result = apply_patch(&FOV, true);
    match result {
        Ok(()) => println!("[OK] - Applied patch: FOV - by AdamPlayer"),
        Err(error) => println!("[FAIL] - {}", error),
    }

    // Spawn a new thread to let the game continue running
    thread::spawn(settings_watcher);
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