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
    callback: Option<DataCallback>,
) -> *mut c_char {
    println!("🚀 [Rust] start_bot вызван");

    // Проверка на null-указатели
    if coins.is_null() {
        println!("❌ [Rust] coins == NULL");
        return CString::new("coins pointer is null").unwrap().into_raw();
    }

    if stream_types.is_null() {
        println!("❌ [Rust] stream_types == NULL");
        return CString::new("stream_types pointer is null").unwrap().into_raw();
    }

    if keep_running.is_null() {
        println!("❌ [Rust] keep_running == NULL");
        return CString::new("keep_running pointer is null").unwrap().into_raw();
    }

    // Преобразование строк
    let coins_str = unsafe { CStr::from_ptr(coins).to_str().map(|s| s.to_owned()) };
    let streams_str = unsafe { CStr::from_ptr(stream_types).to_str().map(|s| s.to_owned()) };

    let (coins_owned, streams_owned) = match (coins_str, streams_str) {
        (Ok(coins), Ok(streams)) => {
            println!("✅ [Rust] Получены строки: coins = '{}', streams = '{}'", coins, streams);
            (coins, streams)
        }
        _ => {
            println!("❌ [Rust] Ошибка при конвертации CStr в String");
            return CString::new("Invalid input strings").unwrap().into_raw();
        }
    };

    // Проверка callback
    let Some(callback_fn) = callback else {
        println!("❌ [Rust] Callback == NULL");
        return CString::new("Callback function is null").unwrap().into_raw();
    };

    println!("🔄 [Rust] Инициализация Tokio runtime...");
    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => {
            println!("✅ [Rust] Tokio runtime создан");
            rt
        }
        Err(e) => {
            println!("❌ [Rust] Ошибка создания Tokio runtime: {e}");
            return CString::new(format!("Runtime creation error: {e}")).unwrap().into_raw();
        }
    };

    println!("🏁 [Rust] Запуск run_trading_bot...");
    match rt.block_on(run_trading_bot(
        &coins_owned,
        &streams_owned,
        keep_running,
        callback_fn,
    )) {
        Ok(_) => {
            println!("✅ [Rust] Бот успешно завершил работу");
            std::ptr::null_mut()
        }
        Err(err) => {
            println!("❌ [Rust] Ошибка в run_trading_bot: {err}");
            CString::new(err).unwrap().into_raw()
        }
    }
}
#[no_mangle]
pub extern "C" fn free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe { let _ = CString::from_raw(s); }
    }
}
