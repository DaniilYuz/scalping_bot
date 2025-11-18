#[path = "core.rs"]
mod core;

use std::collections::HashMap;
use core::{StrategySignal, run_trading_bot};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use log::{info, debug, error, warn, trace};

#[derive(Debug, Clone)]
pub struct StrategySignalWithCoreId {
    pub core_id: i64,
    pub signal: StrategySignal,
}

struct CoreInstance {
    user_id: i64,
    core_id: i64,
    coins: Vec<String>,
    handle: JoinHandle<()>,
    shutdown_tx: mpsc::Sender<()>,
}

struct UserState {
    signal_sender: mpsc::Sender<StrategySignalWithCoreId>,
    cores: HashMap<i64, CoreInstance>,
}

pub struct CoreManager {
    users: HashMap<i64, UserState>,
    next_core_id: i64,
}

impl CoreManager {
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
            next_core_id: 1,
        }
    }

    fn generate_core_id(&mut self) -> i64 {
        let id = self.next_core_id;
        self.next_core_id += 1;
        id
    }

    /// Регистрирует пользователя и возвращает receiver для получения сигналов
    /// ВАЖНО: Вызывать только один раз на пользователя!
    pub fn register_user(&mut self, user_id: i64) -> mpsc::Receiver<StrategySignalWithCoreId> {
        // Если пользователь уже зарегистрирован, возвращаем новый receiver
        if self.users.contains_key(&user_id) {
            println!("[MANAGER] User {} already registered, creating new receiver", user_id);
        }

        let (signal_sender, signal_receiver) = mpsc::channel(1000);

        self.users.insert(user_id, UserState {
            signal_sender,
            cores: HashMap::new(),
        });

        signal_receiver
    }

    pub fn start_core(&mut self, user_id: i64, coins: Vec<String>) -> Result<i64, String> {
        let core_id = self.generate_core_id();
        let user_state = self.users.get_mut(&user_id)
            .ok_or_else(|| format!("User {} not registered", user_id))?;

        // Канал для получения сигналов от торгового бота
        let (core_tx, mut core_rx) = mpsc::channel::<StrategySignal>(100);

        let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);

        let user_signal_sender = user_state.signal_sender.clone();

        // Пересылка с добавлением core_id
        let forward_handle = tokio::spawn(async move {
            loop {
                tokio::select! {
                    Some(signal) = core_rx.recv() => {
                        // Оборачиваем сигнал, добавляя core_id
                        let signal_with_id = StrategySignalWithCoreId {
                            core_id,
                            signal,
                        };

                        println!("[USER {} CORE {}] Signal: {:?}", user_id, core_id, signal_with_id);

                        if let Err(e) = user_signal_sender.send(signal_with_id).await {
                            eprintln!("[CORE {}] User channel closed: {}", core_id, e);
                            break;
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        println!("[CORE {}] Shutdown signal received", core_id);
                        break;
                    }
                }
            }
            println!("[CORE {}] Forward task stopped", core_id);
        });

        // Запуск бота - используем импортированную функцию
        let coins_clone = coins.clone();
        let bot_handle = tokio::spawn(async move {
            println!("[CORE {}] Starting trading bot", core_id);
            if let Err(e) = run_trading_bot(coins_clone, core_tx).await {
                eprintln!("[CORE {}] Trading bot error: {}", core_id, e);
            }
            println!("[CORE {}] Trading bot stopped", core_id);
        });

        let main_handle = tokio::spawn(async move {
            tokio::select! {
                _ = bot_handle => println!("[CORE {}] Bot finished", core_id),
                _ = forward_handle => println!("[CORE {}] Forward finished", core_id),
            }
        });

        user_state.cores.insert(core_id, CoreInstance {
            user_id,
            core_id,
            coins,
            handle: main_handle,
            shutdown_tx,
        });

        println!("[MANAGER] Started core {} for user {}", core_id, user_id);
        Ok(core_id)
    }

    pub async fn stop_core(&mut self, user_id: i64, core_id: i64) -> Result<(), String> {
        let user_state = self.users.get_mut(&user_id)
            .ok_or_else(|| format!("User {} not found", user_id))?;

        let core = user_state.cores.remove(&core_id)
            .ok_or_else(|| format!("Core {} not found for user {}", core_id, user_id))?;

        println!("[MANAGER] Stopping core {} for user {}", core_id, user_id);
        let _ = core.shutdown_tx.send(()).await;
        let _ = core.handle.await;
        println!("[MANAGER] Core {} stopped", core_id);
        Ok(())
    }

    pub async fn stop_user_cores(&mut self, user_id: i64) -> Result<(), String> {
        let user_state = self.users.get_mut(&user_id)
            .ok_or_else(|| format!("User {} not found", user_id))?;

        let core_ids: Vec<i64> = user_state.cores.keys().copied().collect();

        for core_id in core_ids {
            self.stop_core(user_id, core_id).await?;
        }

        Ok(())
    }

    pub async fn unregister_user(&mut self, user_id: i64) -> Result<(), String> {
        self.stop_user_cores(user_id).await?;
        self.users.remove(&user_id);
        println!("[MANAGER] User {} unregistered", user_id);
        Ok(())
    }

    pub fn get_user_cores(&self, user_id: i64) -> Option<Vec<(i64, Vec<String>)>> {
        self.users.get(&user_id).map(|user_state| {
            user_state.cores.iter()
                .map(|(id, core)| (*id, core.coins.clone()))
                .collect()
        })
    }

    pub fn total_cores(&self) -> usize {
        self.users.values()
            .map(|user| user.cores.len())
            .sum()
    }
}