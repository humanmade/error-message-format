#![cfg_attr(windows, feature(abi_vectorcall))]

mod ffi;

use ext_php_rs::flags::IniEntryPermission;
use ext_php_rs::prelude::*;
use ext_php_rs::types::ZendStr;
use ext_php_rs::zend::{ExecutorGlobals, IniEntryDef};
use std::sync::{Mutex, OnceLock};

// Global storage for the original error callback
static ORIGINAL_ERROR_CB: OnceLock<Mutex<Option<ffi::zend_error_cb_t>>> = OnceLock::new();

/// Our custom error callback that intercepts ALL PHP errors
unsafe extern "C" fn error_callback(
    error_type: std::os::raw::c_int,
    #[cfg(not(php81))] error_filename: *const std::os::raw::c_char,
    #[cfg(php81)] error_filename: *const ext_php_rs::ffi::zend_string,
    error_lineno: std::os::raw::c_uint,
    message: *mut ext_php_rs::ffi::zend_string,
) {
    dbg!("Error Callback Invoked");
    dbg!("Has original callback? {}", ORIGINAL_ERROR_CB.get().is_some());
    let original_mutex = ORIGINAL_ERROR_CB.get_or_init(|| Mutex::new(None));

    // Get the original callback first
    let original_cb = if let Ok(original_guard) = original_mutex.lock() {
        *original_guard
    } else {
        None
    };

    // Get the current INI setting for error_message_format
    let ini_values = ExecutorGlobals::get().ini_values();
    let Some(Some(format_string)) = ini_values.get("error_message_format") else {
        // If no format string, call the original error handler with original message
        if let Some(original_callback) = original_cb {
            original_callback(error_type, error_filename, error_lineno, message);
        }
        return;
    };

    if format_string.is_empty() {
        // If format string is empty, call the original error handler with original message
        if let Some(original_callback) = original_cb {
            original_callback(error_type, error_filename, error_lineno, message);
        }
        return;
    }

    // Extract filename and message
    let filename = if error_filename.is_null() {
        "".to_string()
    } else {
        #[cfg(php81)]
        let filename = String::try_from(&*error_filename).unwrap_or("".to_string());
        #[cfg(not(php81))]
        let filename =
            String::from_utf8_lossy(unsafe { std::ffi::CStr::from_ptr(error_filename) }.to_bytes())
                .to_string();
        filename
    };

    let original_message = if message.is_null() {
        "".to_string()
    } else {
        String::try_from(&*message).unwrap_or("".to_string())
    };

    // Apply our custom formatting
    let formatted_message = format_string
        .replace("{type}", &error_type.to_string())
        .replace("{file}", &filename)
        .replace("{line}", &error_lineno.to_string())
        .replace("{message}", &original_message);

    // Create a new zend_string with the formatted message
    let mut zend_str = ZendStr::new(&formatted_message, false);
    let new_message = zend_str.as_mut_ptr();

    // Call the original error handler with the formatted message
    if let Some(original_callback) = original_cb {
        original_callback(error_type, error_filename, error_lineno, new_message);
    }
}

/// Startup function to register INI entries and install error hook
pub fn startup(_ty: i32, module_number: i32) -> i32 {
    // Register our INI entry for error_message_format
    let ini_entries: Vec<IniEntryDef> = vec![IniEntryDef::new(
        "error_message_format".to_owned(),
        "".to_owned(),            // Default empty - no custom formatting
        &IniEntryPermission::All, // Can be changed at runtime
    )];
    IniEntryDef::register(ini_entries, module_number);

    // Install the error callback hook
    let original_mutex = ORIGINAL_ERROR_CB.get_or_init(|| Mutex::new(None));

    if let Ok(mut original) = original_mutex.lock() {
        unsafe {
            // Save the original callback
            *original = Some(ffi::zend_error_cb);
            // Install our custom callback
            ffi::zend_error_cb = error_callback;
        }
    } else {
        eprintln!("Failed to lock ORIGINAL_ERROR_CB mutex");
        dbg!("failed to init.");
        return 1; // Error
    }

    0 // Success
}

/// Used by the `phpinfo()` function and when you run `php -i`.
pub extern "C" fn php_module_info(_module: *mut ext_php_rs::zend::ModuleEntry) {
    ext_php_rs::info_table_start!();
    ext_php_rs::info_table_row!("Error Message Format Extension", "enabled");
    ext_php_rs::info_table_row!("Error Message Format Version", env!("CARGO_PKG_VERSION"));
    ext_php_rs::info_table_end!();
}

#[php_module]
#[php(startup = "startup")]
pub fn get_module(module: ModuleBuilder) -> ModuleBuilder {
    module.info_function(php_module_info)
}
