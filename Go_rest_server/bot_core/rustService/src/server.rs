use tonic::{transport::Server, Request, Response, Status};
use tokio::sync::Mutex;
use std::sync::Arc;
use tokio_stream::wrappers::ReceiverStream;
use std::time::{SystemTime, UNIX_EPOCH};
use log::{info, debug, error, warn};

// Импортируем сгенерированный код
pub mod proto {
    tonic::include_proto!("proto");
}

use proto::core_manager_service_server::{CoreManagerService, CoreManagerServiceServer};
use proto::{
    StartRequest, UpdateRequest, StopRequest,
    ActionResponse, SignalResponse, UserId
};

// Импортируем наши модули
//use rust_service::{CoreManager, StrategySignalWithCoreId};
mod core_manager;
use core_manager::CoreManager;

// Враппер для управления состоянием
pub struct CoreManagerServiceImpl {
    manager: Arc<Mutex<CoreManager>>,
}

impl CoreManagerServiceImpl {
    pub fn new() -> Self {
        Self {
            manager: Arc::new(Mutex::new(CoreManager::new())),
        }
    }
}

#[tonic::async_trait]
impl CoreManagerService for CoreManagerServiceImpl {
    async fn start_core(
        &self,
        request: Request<StartRequest>,
    ) -> Result<Response<ActionResponse>, Status> {
        let req = request.into_inner();
        println!("[gRPC] StartCore request: user_id={}, coins={:?}", req.user_id, req.coins);

        let mut manager = self.manager.lock().await;

        // Проверяем, зарегистрирован ли пользователь
        if manager.get_user_cores(req.user_id).is_none() {
            // Регистрируем пользователя, но receiver будет в StreamSignals
            let _ = manager.register_user(req.user_id);
        }

        // Запускаем core
        match manager.start_core(req.user_id, req.coins) {
            Ok(core_id) => {
                println!("[gRPC] Core {} started successfully for user {}", core_id, req.user_id);
                Ok(Response::new(ActionResponse {
                    user_id: req.user_id,
                    core_id,
                    is_done: true,
                    error_message: String::new(),
                }))
            }
            Err(e) => {
                eprintln!("[gRPC] Failed to start core: {}", e);
                Ok(Response::new(ActionResponse {
                    user_id: req.user_id,
                    core_id: -1,
                    is_done: false,
                    error_message: e,
                }))
            }
        }
    }

    async fn update_core(
        &self,
        request: Request<UpdateRequest>,
    ) -> Result<Response<ActionResponse>, Status> {
        let req = request.into_inner();
        println!("[gRPC] UpdateCore request: user_id={}, core_id={}", req.user_id, req.core_id);

        let mut manager = self.manager.lock().await;

        // Останавливаем старый core
        match manager.stop_core(req.user_id, req.core_id).await {
            Ok(_) => {
                // Запускаем новый с обновленными параметрами
                match manager.start_core(req.user_id, req.coins) {
                    Ok(new_core_id) => {
                        println!("[gRPC] Core updated: old={}, new={}", req.core_id, new_core_id);
                        Ok(Response::new(ActionResponse {
                            user_id: req.user_id,
                            core_id: new_core_id,
                            is_done: true,
                            error_message: String::new(),
                        }))
                    }
                    Err(e) => {
                        Ok(Response::new(ActionResponse {
                            user_id: req.user_id,
                            core_id: -1,
                            is_done: false,
                            error_message: format!("Failed to start new core: {}", e),
                        }))
                    }
                }
            }
            Err(e) => {
                Ok(Response::new(ActionResponse {
                    user_id: req.user_id,
                    core_id: req.core_id,
                    is_done: false,
                    error_message: format!("Failed to stop old core: {}", e),
                }))
            }
        }
    }

    async fn stop_core(
        &self,
        request: Request<StopRequest>,
    ) -> Result<Response<ActionResponse>, Status> {
        let req = request.into_inner();
        println!("[gRPC] StopCore request: user_id={}, core_id={}", req.user_id, req.core_id);

        let mut manager = self.manager.lock().await;

        match manager.stop_core(req.user_id, req.core_id).await {
            Ok(_) => {
                println!("[gRPC] Core {} stopped successfully", req.core_id);
                Ok(Response::new(ActionResponse {
                    user_id: req.user_id,
                    core_id: req.core_id,
                    is_done: true,
                    error_message: String::new(),
                }))
            }
            Err(e) => {
                eprintln!("[gRPC] Failed to stop core: {}", e);
                Ok(Response::new(ActionResponse {
                    user_id: req.user_id,
                    core_id: req.core_id,
                    is_done: false,
                    error_message: e,
                }))
            }
        }
    }

    type StreamSignalsStream = ReceiverStream<Result<SignalResponse, Status>>;

    async fn stream_signals(
        &self,
        request: Request<UserId>,
    ) -> Result<Response<Self::StreamSignalsStream>, Status> {
        let user_id = request.into_inner().user_id;
        println!("[gRPC] StreamSignals started for user {}", user_id);

        let mut manager = self.manager.lock().await;

        // Регистрируем пользователя и получаем receiver
        let mut signal_receiver = manager.register_user(user_id);

        // Создаем канал для gRPC stream
        let (tx, rx) = tokio::sync::mpsc::channel(100);

        // Spawним задачу для пересылки сигналов в gRPC stream
        tokio::spawn(async move {
            println!("[gRPC] Signal forwarding task started for user {}", user_id);

            while let Some(signal_with_id) = signal_receiver.recv().await {
                let timestamp = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64;

                let grpc_signal = SignalResponse {
                    user_id,
                    core_id: signal_with_id.core_id,
                    tactic_name: signal_with_id.signal.tactic_name.clone(),
                    tactic_signal: signal_with_id.signal.tactic_signal.clone(),
                    timestamp,
                };

                println!("[gRPC] Sending signal to user {}: core={}, tactic={}, signal={}",
                         user_id,
                         signal_with_id.core_id,
                         signal_with_id.signal.tactic_name,
                         signal_with_id.signal.tactic_signal
                );

                if tx.send(Ok(grpc_signal)).await.is_err() {
                    println!("[gRPC] Client disconnected, stopping signal stream for user {}", user_id);
                    break;
                }
            }

            println!("[gRPC] Signal forwarding task stopped for user {}", user_id);
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    info!("🚀 Starting server...");

    let addr = "[::1]:50051".parse()?;
    let service = CoreManagerServiceImpl::new();

    println!("🚀 CoreManagerService gRPC server listening on {}", addr);

    Server::builder()
        .add_service(CoreManagerServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}