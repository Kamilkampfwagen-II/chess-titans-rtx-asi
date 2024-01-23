mod patches;
use patches::patches::{Patch, CONSTANT_TICK, FOV};

use std::ffi::c_void;
use std::mem::size_of;

use windows::Win32::Foundation::{BOOL, HANDLE};
use windows::Win32::System::Console::AllocConsole;
use windows::Win32::System::Memory::{PAGE_PROTECTION_FLAGS, VirtualProtect, PAGE_EXECUTE_READWRITE};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::SystemServices::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH, DLL_THREAD_ATTACH, DLL_THREAD_DETACH};


fn patch_game(patch_set: &[Patch])
{
    let h_parent_module = unsafe { GetModuleHandleW(None).unwrap() };

    for patch in patch_set.iter() {        
        let patch_address = (h_parent_module.0 + patch.offset as isize) as *mut u8;

        let mut old_protect: PAGE_PROTECTION_FLAGS = Default::default();
        unsafe {
            // Disable virtual page protection
            VirtualProtect(patch_address as *const c_void, size_of::<u8>(), PAGE_EXECUTE_READWRITE, &mut old_protect).expect("Failed to disable page protection!");

            // Write the individual bytes
            *patch_address = patch.new;

            // Restore virtual page protection
            VirtualProtect(patch_address as *const c_void, size_of::<u8>(), old_protect, &mut old_protect).expect("Failed to restore page protection!");
        };
    }
}

fn main() {
    // Attach a console so we can print stuff
    unsafe { AllocConsole().expect("Failed to allocate console!") }

    println!("Welcome to Chess Titans RTX");

    println!("Patching: CONSTANT_TICK..");
    patch_game(&CONSTANT_TICK);

    println!("Patching: FOV..");
    patch_game(&FOV);
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