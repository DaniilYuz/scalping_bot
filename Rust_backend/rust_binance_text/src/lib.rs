// src/lib.rs
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};

// Тип C-функции обратного вызова
pub type DataCallback = extern "C" fn(*const c_char);

mod main_bot;
use crate::main_bot::run_trading_bot;

#[no_mangle]
pub extern "C" fn start_bot(
    coins: *const c_char,
    stream_types: *const c_char,
    keep_running: *mut c_int,
    callback: Option<DataCallback>, // Новый параметр: callback
) -> *mut c_char {
    // Безопасное преобразование C-строк в Rust String
    let coins_str = unsafe {
        CStr::from_ptr(coins).to_str().map(|s| s.to_owned())
    };

    let streams_str = unsafe {
        CStr::from_ptr(stream_types).to_str().map(|s| s.to_owned())
    };

    let (coins_owned, streams_owned) = match (coins_str, streams_str) {
        (Ok(coins), Ok(streams)) => (coins, streams),
        _ => {
            return CString::new("Invalid input strings").unwrap().into_raw();
        }
    };

    // Проверка на наличие callback
    let Some(callback_fn) = callback else {
        return CString::new("Callback function is null").unwrap().into_raw();
    };

    // Запуск асинхронного бота
    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => return CString::new(format!("Runtime creation error: {e}")).unwrap().into_raw(),
    };

    match rt.block_on(run_trading_bot(
        &coins_owned,
        &streams_owned,
        keep_running,
        callback_fn,
    )) {
        Ok(_) => std::ptr::null_mut(), // Успешно
        Err(err) => CString::new(err).unwrap().into_raw(), // Возврат ошибки как C-строки
    }
}

#[no_mangle]
pub extern "C" fn free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe { let _ = CString::from_raw(s); }
    }
}
