#[no_mangle]
pub extern "C" fn CreateInterface(
    _name: *const i8,
    _return_code: *mut i32,
) -> *mut std::ffi::c_void {
    std::ptr::null_mut()
}
