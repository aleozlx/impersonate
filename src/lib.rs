extern "C" { fn _su(user: *mut std::os::raw::c_char) -> i32; }
pub fn su(user: &str) { unsafe { _su(std::ffi::CString::new(user).unwrap().into_raw()); } }