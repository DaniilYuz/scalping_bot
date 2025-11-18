// src/lib.rs
pub mod core;
pub mod core_manager;

// Реэкспорт для удобства
pub use core::{StrategySignal, run_trading_bot};
pub use core_manager::{CoreManager, StrategySignalWithCoreId};