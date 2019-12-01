use std::os::raw::{c_char, c_int};
use std::ffi::CStr;
use std::slice;

mod hackel;

#[no_mangle]
pub extern fn diffWithString(old: *const *const c_char, old_len: c_int, new: *const *const c_char, new_len: c_int) {
    let old = unsafe { collect_strs_from(old, old_len) };
    let new = unsafe { collect_strs_from(new, new_len) };
    hackel::diff(&old, &new);
}

#[inline]
unsafe fn collect_strs_from(array_of_c_char: *const *const c_char, len: c_int) -> Vec<&'static str> {
    slice::from_raw_parts(array_of_c_char, len as usize)
        .iter()
        .map( |char| CStr::from_ptr(*char).to_str().unwrap())
        .collect()
}
