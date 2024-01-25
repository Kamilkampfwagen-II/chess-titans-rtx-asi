pub mod patches {
    use crate::patch::patch::*;

    // by https://github.com/adamplayer
    pub const CONSTANT_TICK: [Patch; 6] = [
        Patch {offset: 0x0003FA0E, org: 0x75, new: 0x90},
        Patch {offset: 0x0003FA0F, org: 0x0A, new: 0x90},
        Patch {offset: 0x0003FA14, org: 0x75, new: 0x90},
        Patch {offset: 0x0003FA15, org: 0x04, new: 0x90},
        Patch {offset: 0x0003FA18, org: 0x75, new: 0xEB},
        Patch {offset: 0x0003FB09, org: 0x75, new: 0xEB},
    ];

    // by https://github.com/adamplayer
    pub const FOV: [Patch; 3] = [
        Patch {offset: 0x0013100A, org: 0xBE, new: 0x20},
        Patch {offset: 0x0013100E, org: 0xF0, new: 0xB4},
        Patch {offset: 0x0013100F, org: 0x41, new: 0x42},
    ];

    pub const GRAPHICS_LEVEL_3: [Patch; 1] = [
        Patch {offset: 0x131134, org: 0x0, new: 0x2},
    ];
}