// Source of the original C header:
// https://github.com/xoxor4d/bridge-remix/blob/feature/bridge-api/src/api/bridge_api/bridge_c.h

#![allow(dead_code)]

use remix_api::*;
use std::ffi::{c_char, CString};
use std::mem::{self, MaybeUninit};
use windows::core::PCSTR;
use windows::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};

#[repr(C)]
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorCode {
    Success = 0,
    GeneralFailure = 1,
    GetModuleHandleFailure = 2,
    InvalidArguments = 3,
    GetProcAddressFailure = 4,
    AlreadyExists = 5,
    RegisteringNonRemixD3D9Device = 6,
    RemixDeviceWasNotRegistered = 7,
    IncompatibleVersion = 8,
    NotInitialized = 11,
}

pub type DebugPrint = extern "system" fn(*const c_char);
pub type CreateTriangleMesh = extern "system" fn(*const MeshInfo) -> u64;
pub type DrawMeshInstance = extern "system" fn(u64, *const Transform, Bool);

pub type CreateSphereLight =
    extern "system" fn(*const LightInfo, *const LightInfoSphereExt) -> u64;

pub type CreateRectLight = extern "system" fn(*const LightInfo, *const LightInfoRectExt) -> u64;

pub type CreateDiskLight = extern "system" fn(*const LightInfo, *const LightInfoDiskExt) -> u64;

pub type CreateCylinderLight =
    extern "system" fn(*const LightInfo, *const LightInfoCylinderExt) -> u64;

pub type CreateDistantLight =
    extern "system" fn(*const LightInfo, *const LightInfoDistantExt) -> u64;

pub type DestroyLight = extern "system" fn(u64);
pub type DrawLightInstance = extern "system" fn(u64);
pub type SetConfigVariable = extern "system" fn(*const c_char, *const c_char);
pub type RegisterDevice = extern "system" fn();
pub type EndSceneCallback = extern "C" fn();
pub type RegisterEndSceneCallback = extern "C" fn(EndSceneCallback);

#[allow(non_snake_case)]
#[repr(C)]
#[derive(Debug, Clone)]
pub struct Interface {
    pub initialized: bool,
    pub DebugPrint: MaybeUninit<DebugPrint>,
    _pad0: MaybeUninit<unsafe extern "system" fn(u64)>,
    _pad1: MaybeUninit<unsafe extern "system" fn(u64)>,
    pub CreateTriangleMesh: MaybeUninit<CreateTriangleMesh>,
    _pad2: MaybeUninit<unsafe extern "system" fn(u64)>,
    pub DrawMeshInstance: MaybeUninit<DrawMeshInstance>,
    pub CreateSphereLight: MaybeUninit<CreateSphereLight>,
    pub CreateRectLight: MaybeUninit<CreateRectLight>,
    pub CreateDiskLight: MaybeUninit<CreateDiskLight>,
    pub CreateCylinderLight: MaybeUninit<CreateCylinderLight>,
    pub CreateDistantLight: MaybeUninit<CreateDistantLight>,
    pub DestroyLight: MaybeUninit<DestroyLight>,
    pub DrawLightInstance: MaybeUninit<DrawLightInstance>,
    pub SetConfigVariable: MaybeUninit<SetConfigVariable>,
    pub RegisterDevice: MaybeUninit<RegisterDevice>,
    pub RegisterEndSceneCallback: MaybeUninit<RegisterEndSceneCallback>,
}

impl Default for Interface {
    fn default() -> Self {
        Self {
            initialized: false,
            DebugPrint: MaybeUninit::uninit(),
            _pad0: MaybeUninit::uninit(),
            _pad1: MaybeUninit::uninit(),
            CreateTriangleMesh: MaybeUninit::uninit(),
            _pad2: MaybeUninit::uninit(),
            DrawMeshInstance: MaybeUninit::uninit(),
            CreateSphereLight: MaybeUninit::uninit(),
            CreateRectLight: MaybeUninit::uninit(),
            CreateDiskLight: MaybeUninit::uninit(),
            CreateCylinderLight: MaybeUninit::uninit(),
            CreateDistantLight: MaybeUninit::uninit(),
            DestroyLight: MaybeUninit::uninit(),
            DrawLightInstance: MaybeUninit::uninit(),
            SetConfigVariable: MaybeUninit::uninit(),
            RegisterDevice: MaybeUninit::uninit(),
            RegisterEndSceneCallback: MaybeUninit::uninit(),
        }
    }
}

pub type InitFuncs = extern "system" fn(*mut Interface) -> ErrorCode;

pub fn initialize() -> Result<Interface, ErrorCode> {
    let module_name = CString::new("d3d9.dll").unwrap();
    let module_name_ptr = PCSTR(module_name.as_bytes_with_nul().as_ptr());

    let proc_name = CString::new("bridgeapi_InitFuncs").unwrap();
    let proc_name_ptr = PCSTR(proc_name.as_bytes_with_nul().as_ptr());

    let h_module = match unsafe { GetModuleHandleA(module_name_ptr) } {
        Ok(v) => {
            if v.is_invalid() {
                return Err(ErrorCode::GetModuleHandleFailure);
            }
            v
        }
        Err(_) => return Err(ErrorCode::GetModuleHandleFailure),
    };

    let func = match unsafe { GetProcAddress(h_module, proc_name_ptr) } {
        Some(v) => v,
        None => return Err(ErrorCode::GetProcAddressFailure),
    };

    let _initialize: InitFuncs = unsafe { mem::transmute(func) };

    let mut bridge_interface = Interface::default();

    let status = _initialize(&mut bridge_interface);
    if status == ErrorCode::Success {
        bridge_interface.initialized = true;
        return Ok(bridge_interface);
    }

    Err(status)
}

pub mod remix_api {
    #[repr(C)]
    #[derive(Debug, Clone, PartialEq)]
    pub enum StructType {
        None = 0,
        InitializeLibraryInfo = 1,
        MaterialInfo = 2,
        MaterialInfoPortalExt = 3,
        MaterialInfoTranslucentExt = 4,
        MaterialInfoOpaqueExt = 5,
        LightInfo = 6,
        LightInfoDistantExt = 7,
        LightInfoCylinderExt = 8,
        LightInfoDiskExt = 9,
        LightInfoRextExt = 10,
        LightInfoSphereExt = 11,
        MeshInfo = 12,
        InstanceInfo = 13,
        InstanceInfoBoneTransformsExt = 14,
        InstanceInfoBlendExt = 15,
        CameraInfo = 16,
        CameraInfoParameterizedExt = 17,
        MaterialInfoOpaqueSubsurfaceExt = 18,
        InstanceInfoObjectPickingExt = 19,
        LightInfoDomeExt = 20,
        LightInfoUsdExt = 21,
        StartupInfo = 22,
        PresentInfo = 23,
    }

    #[repr(C)]
    #[derive(Debug, Clone)]
    pub struct Rect2D {
        pub left: i32,
        pub top: i32,
        pub right: i32,
        pub bottom: i32,
    }

    #[repr(C)]
    #[derive(Debug, Clone)]
    pub struct Float2D {
        pub x: f32,
        pub y: f32,
    }

    #[repr(C)]
    #[derive(Debug, Clone)]
    pub struct Float3D {
        pub x: f32,
        pub y: f32,
        pub z: f32,
    }

    #[repr(C)]
    #[derive(Debug, Clone)]
    pub struct Float4D {
        pub x: f32,
        pub y: f32,
        pub z: f32,
        pub w: f32,
    }

    #[repr(C)]
    #[derive(Debug, Clone)]
    pub struct Transform {
        pub matrix: [[f32; 4]; 3],
    }

    #[repr(C)]
    pub struct MaterialHandleT {}

    #[repr(C)]
    pub struct MeshHandleT {}

    #[repr(C)]
    pub struct LightHandleT {}

    pub type Bool = u32;
    pub type Path = *const char;
    pub type MaterialHandle = *mut MaterialHandleT;
    pub type MeshHandle = *mut MeshHandleT;
    pub type LightHandle = *mut LightHandleT;

    #[repr(C)]
    #[derive(Debug, Clone)]
    pub struct MaterialInfo {
        pub s_type: StructType,
        pub hash: u64,
        pub albedo_texture: Path,
        pub normal_texture: Path,
        pub tangent_texture: Path,
        pub emissive_texture: Path,
        pub emissive_intensity: f32,
        pub emissive_color_constant: Float3D,
        pub sprite_sheet_row: u8,
        pub sprite_sheet_col: u8,
        pub sprite_sheet_fps: u8,
        pub filter_mode: u8,
        pub wrap_mode_u: u8,
        pub wrap_mode_v: u8,
    }

    #[repr(C)]
    #[derive(Debug, Clone)]
    pub struct HardcodedVertex {
        pub position: [f32; 3],
        pub normal: [f32; 3],
        pub texcoord: [f32; 2],
        pub color: u32,
        pub _pad0: u32,
        pub _pad1: u32,
        pub _pad2: u32,
        pub _pad3: u32,
        pub _pad4: u32,
        pub _pad5: u32,
        pub _pad6: u32,
    }

    #[repr(C)]
    #[derive(Debug, Clone)]
    pub struct MeshInfoSurfaceTriangles {
        pub vertices_values: *const HardcodedVertex,
        pub vertices_count: u64,
        pub indices_values: *const u32,
        pub indices_count: u64,
        pub skinning_hasvalue: Bool,
        pub material: MaterialHandle,
    }

    #[repr(C)]
    #[derive(Debug, Clone)]
    pub struct MeshInfo {
        pub s_type: StructType,
        pub hash: u64,
        pub surfaces_values: *const MeshInfoSurfaceTriangles,
        pub surfaces_count: u32,
    }

    #[repr(C)]
    #[derive(Debug, Clone)]
    pub struct LightInfoLightShaping {
        pub direction: Float3D,
        pub cone_angle_degrees: f32,
        pub cone_softness: f32,
        pub focus_exponent: f32,
    }

    #[repr(C)]
    #[derive(Debug, Clone)]
    pub struct LightInfoSphereExt {
        pub s_type: StructType,
        pub position: Float3D,
        pub radius: f32,
        pub shaping_hasvalue: Bool,
        pub shaping_value: LightInfoLightShaping,
    }

    #[repr(C)]
    #[derive(Debug, Clone)]
    pub struct LightInfoRectExt {
        pub s_type: StructType,
        pub position: Float3D,
        pub x_axis: Float3D,
        pub x_size: f32,
        pub y_axis: Float3D,
        pub y_size: f32,
        pub direction: Float3D,
        pub shaping_hasvalue: Bool,
        pub shaping_value: LightInfoLightShaping,
    }

    #[repr(C)]
    #[derive(Debug, Clone)]
    pub struct LightInfoDiskExt {
        pub s_type: StructType,
        pub position: Float3D,
        pub x_axis: Float3D,
        pub x_radius: f32,
        pub y_axis: Float3D,
        pub y_radius: f32,
        pub direction: Float3D,
        pub shaping_hasvalue: Bool,
        pub shaping_value: LightInfoLightShaping,
    }

    #[repr(C)]
    #[derive(Debug, Clone)]
    pub struct LightInfoCylinderExt {
        pub s_type: StructType,
        pub position: Float3D,
        pub radius: f32,
        pub axis: Float3D,
        pub axis_length: f32,
    }

    #[repr(C)]
    #[derive(Debug, Clone)]
    pub struct LightInfoDistantExt {
        pub s_type: StructType,
        pub direction: Float3D,
        pub angular_diameter_degrees: f32,
    }

    #[repr(C)]
    #[derive(Debug, Clone)]
    pub struct LightInfo {
        pub s_type: StructType,
        pub hash: u64,
        pub radiance: Float3D,
    }
}
