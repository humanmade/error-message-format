#![allow(non_camel_case_types)]

use ext_php_rs::ffi::zend_string;
use std::os::raw::{c_int, c_uint};

// PHP error callback function pointer type
pub type zend_error_cb_t = unsafe extern "C" fn(
    error_type: c_int,
    error_filename: *mut zend_string,
    error_lineno: c_uint,
    message: *mut zend_string,
);

extern "C" {
    // Global error callback pointer
    pub static mut zend_error_cb: zend_error_cb_t;
}
