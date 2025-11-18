use tonic::Request;

pub mod proto {
    tonic::include_proto!("proto");
}

use proto::core_manager_service_client::CoreManagerServiceClient;
use proto::{StartRequest, StopRequest, UserId};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Подключаемся к серверу
    let mut client = CoreManagerServiceClient::connect("http://[::1]:50051").await?;

    println!("✅ Connected to gRPC server");

    // 1. Запускаем core для пользователя
    let start_request = Request::new(StartRequest {
        user_id: 1,
        coins: vec!["BTCUSDT".to_string(), "ETHUSDT".to_string()],
    });

    let response = client.start_core(start_request).await?;
    let start_resp = response.into_inner();

    if start_resp.is_done {
        println!("✅ Core {} started for user {}", start_resp.core_id, start_resp.user_id);
    } else {
        println!("❌ Failed to start core: {}", start_resp.error_message);
        return Ok(());
    }

    let core_id = start_resp.core_id;

    // 2. Запускаем stream для получения сигналов
    let stream_request = Request::new(UserId { user_id: 1 });
    let mut stream = client.stream_signals(stream_request).await?.into_inner();

    println!("📡 Listening for signals...");

    // Spawним задачу для чтения сигналов
    let stream_handle = tokio::spawn(async move {
        while let Some(signal) = stream.message().await.unwrap() {
            println!(
                "📊 Signal received: user={}, core={}, tactic={}, signal={}",
                signal.user_id, signal.core_id, signal.tactic_name, signal.tactic_signal
            );
        }
        println!("Stream ended");
    });

    // 3. Ждем немного (в реальном приложении это будет бесконечный цикл)
    tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;

    // 4. Останавливаем core
    let stop_request = Request::new(StopRequest {
        user_id: 1,
        core_id,
    });

    let response = client.stop_core(stop_request).await?;
    let stop_resp = response.into_inner();

    if stop_resp.is_done {
        println!("✅ Core {} stopped", core_id);
    } else {
        println!("❌ Failed to stop core: {}", stop_resp.error_message);
    }

    // Ждем завершения stream
    stream_handle.abort();

    Ok(())
}