// src/lib.rs
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;

// Импортируем функцию из main_bot
mod main_bot;
use crate::main_bot::run_trading_bot;

#[no_mangle]
pub extern "C" fn start_bot(
    coins: *const c_char,
    stream_types: *const c_char,
) -> *mut c_char {
    // Безопасное преобразование C-строк в Rust String
    let input_coin = unsafe { CStr::from_ptr(coins).to_string_lossy().into_owned() };
    let input_stream_types = unsafe { CStr::from_ptr(stream_types).to_string_lossy().into_owned() };

    // Запуск бота (блокирующий вызов, так как FFI не поддерживает async напрямую)
    let rt = tokio::runtime::Runtime::new().unwrap();
    match rt.block_on(run_trading_bot(input_coin, input_stream_types)) {
        Ok(_) => ptr::null_mut(),
        Err(e) => CString::new(e.to_string()).unwrap().into_raw(),
    }
}

// Функция для освобождения памяти строки (обязательно для FFI!)
#[no_mangle]
pub extern "C" fn free_string(s: *mut c_char) {
    unsafe {
        if s.is_null() {
            return;
        }
        let _ = CString::from_raw(s);
    }
}