// src/lib.rs
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use tokio::runtime::Runtime;

// Тип C-функции обратного вызова
pub type DataCallback = extern "C" fn(*const c_char);

mod main_bot;
use crate::main_bot::run_trading_bot;

#[no_mangle]
pub extern "C" fn start_bot(
    coins: *const c_char,
    stream_types: *const c_char,
    keep_running: Arc<AtomicBool>,
    callback: Option<DataCallback>,
) -> *mut c_char {
    println!("🚀 [Rust] start_bot вызван");

    if coins.is_null() || stream_types.is_null() {
        return CString::new("One of the input pointers is null").unwrap().into_raw();
    }




    let coins_str = unsafe { CStr::from_ptr(coins).to_str().map(|s| s.to_owned()) };
    let streams_str = unsafe { CStr::from_ptr(stream_types).to_str().map(|s| s.to_owned()) };

    let (coins_owned, streams_owned) = match (coins_str, streams_str) {
        (Ok(c), Ok(s)) => (c, s),
        _ => return CString::new("Invalid UTF-8 strings").unwrap().into_raw(),
    };

    let Some(callback_fn) = callback else {
        return CString::new("Callback function is null").unwrap().into_raw();
    };

    println!("🧵 [Rust] Spawning async task...");

    // Создаем Tokio Runtime
    let rt = match Runtime::new() {
        Ok(runtime) => runtime,
        Err(e) => {
            return CString::new(format!("Failed to create Tokio Runtime: {}", e))
                .unwrap()
                .into_raw();
        }
    };

    // Запускаем async задачу внутри Runtime
    rt.spawn(async move {
        if let Err(e) = run_trading_bot(
            &coins_owned,
            &streams_owned,
            keep_running,
            callback_fn,
        )
            .await
        {
            eprintln!("❌ [Rust] Ошибка в run_trading_bot: {e}");
        } else {
            println!("✅ [Rust] run_trading_bot завершён");
        }
    });

    std::ptr::null_mut() // Успешный запуск, ошибки нет
}

#[no_mangle]
pub extern "C" fn free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe { let _ = CString::from_raw(s); }
    }
}
