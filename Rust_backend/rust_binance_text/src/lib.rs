use std::sync::OnceLock;
// src/lib.rs
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}, Mutex};
use tokio::runtime::Runtime;

// Тип C-функции обратного вызова
pub type DataCallback = extern "C" fn(*const c_char);

mod main_bot;
use crate::main_bot::run_trading_bot;

static RUNTIME_CELL: OnceLock<Mutex<Option<Runtime>>> = OnceLock::new();

#[no_mangle]
pub extern "C" fn start_bot(
    coins: *const c_char,
    stream_types: *const c_char,
    keep_running: Arc<AtomicBool>,
    callback: Option<DataCallback>,
) -> *mut c_char {
    println!("🚀 [Rust] start_bot вызван");

    RUNTIME_CELL.get_or_init(|| Mutex::new(None));


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

    let mut runtime_guard = RUNTIME_CELL.get().unwrap().lock().unwrap();
    if runtime_guard.is_some() {
        return CString::new("Runtime already running").unwrap().into_raw();
    }

    let runtime = match Runtime::new() {
        Ok(rt) => rt,
        Err(e) => return CString::new(format!("Failed to create Tokio Runtime: {}", e)).unwrap().into_raw(),
    };

    runtime.spawn(async move {
        if let Err(e) = run_trading_bot(
            &coins_owned,
            &streams_owned,
            keep_running.clone(),
            callback_fn,
        ).await {
            eprintln!("❌ Ошибка в run_trading_bot: {e}");
        } else {
            println!("✅ run_trading_bot завершён");
        }

        // После завершения очищаем runtime
        let mut runtime_lock = RUNTIME_CELL.get().unwrap().lock().unwrap();
        *runtime_lock = None;
        println!("🧹 Runtime очищен после завершения");
    });

    *runtime_guard = Some(runtime);

    std::ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe { let _ = CString::from_raw(s); }
    }
}

#[no_mangle]
pub extern "C" fn is_runtime_initialized() -> bool {
    if let Some(lock) = RUNTIME_CELL.get() {
        let guard = lock.lock().unwrap();
        guard.is_some()
    } else {
        false
    }
}
