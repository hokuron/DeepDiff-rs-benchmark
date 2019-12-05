use std::os::raw::{c_char, c_int};
use std::ffi::CStr;
use std::slice;

mod hackel;

#[no_mangle]
pub extern fn diffWithString(old: *const *const c_char, old_len: c_int, new: *const *const c_char, new_len: c_int) {
    let old = unsafe { slice::from_raw_parts(old, old_len as usize) };
    let new = unsafe { slice::from_raw_parts(new, new_len as usize) };
    hackel::diff(old, &new);
}
