#[cfg(windows)]
pub mod d3d11;
#[cfg(windows)]
pub mod d3d12;
pub mod opengl;
pub mod vulkan;

#[derive(Debug, Clone, Copy)]
pub enum GraphicsAPI {
    DirectX11,
    DirectX12,
    Vulkan,
    OpenGL,
}

pub fn detect_graphics_api() -> Option<GraphicsAPI> {
    #[cfg(windows)]
    {
        use std::ffi::CString;

        unsafe {
            let d3d11_name = CString::new("d3d11.dll").unwrap();
            let d3d11 = winapi::um::libloaderapi::GetModuleHandleA(d3d11_name.as_ptr());

            if !d3d11.is_null() {
                return Some(GraphicsAPI::DirectX11);
            }

            let d3d12_name = CString::new("d3d12.dll").unwrap();
            let d3d12 = winapi::um::libloaderapi::GetModuleHandleA(d3d12_name.as_ptr());

            if !d3d12.is_null() {
                return Some(GraphicsAPI::DirectX12);
            }
        }
    }

    let vulkan_name = std::ffi::CString::new("vulkan-1.dll").ok()?;
    #[cfg(windows)]
    unsafe {
        let vulkan = winapi::um::libloaderapi::GetModuleHandleA(vulkan_name.as_ptr());
        if !vulkan.is_null() {
            return Some(GraphicsAPI::Vulkan);
        }
    }

    None
}
