#![allow(non_camel_case_types)]

use ext_php_rs::ffi::zend_string;
use std::os::raw::{c_int, c_uint};

// PHP error callback function pointer type
// The signature is the same across PHP versions in terms of Rust FFI
pub type zend_error_cb_t = unsafe extern "C" fn(
    error_type: c_int,
    #[cfg(not(php81))] error_filename: *const std::os::raw::c_char,
    #[cfg(php81)] error_filename: *const zend_string,
    error_lineno: c_uint,
    message: *mut zend_string,
);

extern "C" {
    // Global error callback pointer
    pub static mut zend_error_cb: zend_error_cb_t;
}
