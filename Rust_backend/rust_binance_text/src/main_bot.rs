use std::collections::VecDeque;
use std::ffi::{c_char, CStr, CString};
use std::fs::read;
use std::sync::{Arc, Mutex, RwLock};
use std::io;
use std::num::ParseIntError;
use std::sync::atomic::AtomicBool;
use binance::api::*;
use binance::market::*;
use std::thread::sleep;
use std::time::Duration;
use std::thread;
use binance::websockets::{WebSockets, WebsocketEvent};
use std::sync::atomic::Ordering;
use binance::model::{AggTrade, DayTickerEvent};
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Sender, Receiver};
use tokio::time::{Instant, Sleep};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;
/*fn main() {
    let keep_running = Arc::new(AtomicBool::new(true));
    let keep_running_clone = Arc::clone(&keep_running);

    /*
    let handler = |event: WebsocketEvent| {
        println!("Event :{:?}", event);
        Ok(())
    };
    let mut ws = WebSockets:: new(handler);

    ws.connect("btcusdt@trade").expect("Can't connect");
    ws.event_loop(&keep_running).expect("Error during websocket event loop");

     */

    let handler = |event: WebsocketEvent| {
        match event {
            WebsocketEvent::Trade(trade) => {
                println!("Получена сделка: символ: {}, цена: {}, количество: {}",
                         trade.symbol, trade.price, trade.qty);
                Ok(())
            },
            WebsocketEvent::Kline(kline) => {
                println!("Получен Kline: {}", kline.kline.close);
                Ok(())
            },
            WebsocketEvent::DepthOrderBook(depth) => {
                println!("Получено обновление стакана заявок");
                Ok(())
            },
            WebsocketEvent::BookTicker(book) => {
                println!("Получен BookTicker: символ: {}, лучшая ставка: {}",
                         book.symbol, book.best_bid);
                Ok(())
            },
            other => {
                println!("Получено другое событие: {:?}", other);
                Ok(())
            }
        }
    };

    println!("Создание WebSocket клиента");
    let mut ws = WebSockets::new(handler);

    // Создаем таймер для остановки через 30 секунд
    thread::spawn(move || {
        println!("Таймер запущен на 30 секунд");
        thread::sleep(Duration::from_secs(300));
        println!("30 секунд прошло, останавливаем WebSocket");
        keep_running_clone.store(false, Ordering::SeqCst);
    });

    // Пробуем различные варианты формата подписки
    println!("Подключение к потоку...");
    let endpoints = ["btcusdt@trade", "btcusdt@kline_1m"];

    for endpoint in endpoints {
        println!("Пробуем подключиться к: {}", endpoint);
        match ws.connect(endpoint) {
            Ok(_) => {
                println!("Успешно подключились к {}", endpoint);

                //Traid();


                println!("Запуск цикла обработки событий");
                match ws.event_loop(&keep_running) {
                    Ok(_) => println!("Цикл событий завершен нормально"),
                    Err(e) => println!("Ошибка в цикле событий: {:?}", e)
                }

                break;
            },
            Err(e) => {
                println!("Не удалось подключиться к {}: {:?}", endpoint, e);
                continue;
            }
        }
    }

    println!("Программа завершена");
}

 */
/*
trait Strategy {
    fn on_agg_trade(&mut self, trade: &AggTrade);
    fn evaluate(&self);
}
struct MomentumStrategy {
    trades: VecDeque<(AggTrade, Instant)>,
    window: Duration,
}
impl MomentumStrategy {
    fn new(window_secs: u64) -> Self {
        Self {
            trades: VecDeque::<(AggTrade, Instant)>::new(),
            window: Duration::from_secs(window_secs),
        }
    }
}
impl Strategy  for MomentumStrategy {
    fn on_agg_trade (&mut   self, trade: &AggTrade) {
        let now = Instant::now();
        self.trades.push_back((trade.clone(), now));

        while let Some((_, ts)) = self.trades.front() {
            if now.duration_since(*ts) > self.window {
                self.trades.pop_front();
            } else {
                break;
            }
        }
    }

    fn evaluate(&self){
        let mut buy_volume = 0.0;
        let mut sell_volume = 0.0;

        for (trade, _) in &self.trades {
            let volume: f64 = trade.qty;
            if trade.maker {
                sell_volume += volume;
            } else {
                buy_volume += volume;
            }
        }

        println!("💡 Buy: {:.2}, Sell: {:.2}", buy_volume, sell_volume);

        if sell_volume > 0.0 && buy_volume / sell_volume > 1.1 {
            println!("🔥 Покупатели доминируют — можно входить в LONG!");
            std::process::exit(0);
        } else if buy_volume > 0.0 && sell_volume / buy_volume > 1.1 {
            println!("🔥 Продавцы доминируют — можно входить в SHORT!");
            std::process::exit(0);
        }
    }
}
*/
enum StreamData {
    AggTrade(AggTradeData),
    Kline(KlineData),
    DepthOrderBook(OrderBookData),
    BookTicker(BookTickerData),
}
#[derive(Clone)]
struct AggTradeData {
    symbol: String,
    average_price: f64,
    current_close: f64,
}
#[derive(Clone)]
struct KlineData {
    symbol: String,
    open: f64,
    close: f64,
    high: f64,
    low: f64,
    volume: f64,
    taker_buy_base_asset_volume: f64,
    taker_buy_quote_asset_volume: f64,
    is_final_bar: bool,
}
#[derive(Clone,Debug)]
struct OrderBookEntry {
    price: f64,
    quantity: f64,
}

#[derive(Clone)]
struct OrderBookData {
    symbol: String,
    bids: Vec<OrderBookEntry>,
    asks: Vec<OrderBookEntry>,
    first_update_id: u64,
    final_update_id: u64,
}
#[derive(Clone)]
struct BookTickerData {
    symbol: String,
    best_bid: f64,
    best_bid_qty: f64,
    best_ask: f64,
    best_ask_qty: f64,
}
#[derive(Clone)]
struct MarketSnapshot {
    last_agg_trade: Option<AggTradeData>,
    last_kline: Option<KlineData>,
    last_order_book: Option<OrderBookData>,
    last_book_ticker: Option<BookTickerData>,
}

fn handle_event_AggTrade(event: &WebsocketEvent, sender: &mpsc::Sender<StreamData>, coinName: &Vec<String>) -> Result<(), ()> {
    match event {
        WebsocketEvent::DayTickerAll(ticker_events) => {
            for tick_event in ticker_events {
                for coin in coinName {
                    if tick_event.symbol.to_string() == coin.to_uppercase() {
                        println!("Монета: {}. Средняя цена: {} - цена сделки: {}",
                                 tick_event.symbol,
                                 tick_event.average_price.parse::<f64>().unwrap(),
                                 tick_event.current_close.parse::<f64>().unwrap());

                        let agg_trade_data = AggTradeData {
                            symbol: tick_event.symbol.to_string(),
                            average_price: tick_event.average_price.parse::<f64>().unwrap(),
                            current_close: tick_event.current_close.parse::<f64>().unwrap(),
                        };

                        let stream_whole_data = StreamData::AggTrade(agg_trade_data);

                        // Клонируем sender перед перемещением в async блок
                        let sender_clone = sender.clone();

                        tokio::spawn(async move {
                            if let Err(e) = sender_clone.send(stream_whole_data).await {
                                eprintln!("Ошибка отправки KlineData: {:?}", e);
                            }
                        });
                    }
                }
            }
            Ok(())
        },
        _ => Ok(println!("No data(2)")),
    }
}

fn handle_event_Kline(event: &WebsocketEvent, sender: &mpsc::Sender<StreamData>) -> Result<(), ()> {

    match event {
        WebsocketEvent::Kline(kline_event) => {
            //println!("Symbol: {}, high: {}, low: {}", kline_event.kline.symbol, kline_event.kline.low, kline_event.kline.high);
            println!("Монета: {}. Свеча открытия: {}", kline_event.symbol, kline_event.kline.open);
            println!("Монета: {}. Свеча закрытия: {}", kline_event.symbol, kline_event.kline.close);
            println!("Монета: {}. High свечи: {}", kline_event.symbol, kline_event.kline.high);
            println!("Монета: {}. Low свечи: {}", kline_event.symbol, kline_event.kline.low);
            println!("Монета: {}. Объем свечи: {}", kline_event.symbol, kline_event.kline.volume);
            println!("Монета: {}. Покупки по рынку (base): {}", kline_event.symbol, kline_event.kline.taker_buy_base_asset_volume);
            println!("Монета: {}. Покупки по рынку (quote): {}", kline_event.symbol, kline_event.kline.taker_buy_quote_asset_volume);
            println!("Монета: {}. Завершена ли свеча: {}", kline_event.symbol, kline_event.kline.is_final_bar);

            //let mut tick_event_vector : Vec<String> = Vec::new();
            //tick_event_vector.push("Kline".to_string());
            //tick_event_vector.push((&kline_event.kline.symbol).to_string());
            //tick_event_vector.push((&kline_event.kline.open).to_string());
            //tick_event_vector.push((&kline_event.kline.close).to_string());
            //tick_event_vector.push((&kline_event.kline.high).to_string());
            //tick_event_vector.push((&kline_event.kline.low).to_string());
            //tick_event_vector.push((&kline_event.kline.volume).to_string());
            //tick_event_vector.push((&kline_event.kline.taker_buy_base_asset_volume).to_string());
            //tick_event_vector.push((&kline_event.kline.taker_buy_quote_asset_volume).to_string());
            //tick_event_vector.push((&kline_event.kline.is_final_bar).to_string());

            let mut symbol = kline_event.kline.symbol.to_string();
            let mut open = kline_event.kline.open.parse::<f64>().unwrap();
            let mut close = kline_event.kline.close.parse::<f64>().unwrap();
            let mut high = kline_event.kline.high.parse::<f64>().unwrap();
            let mut low = kline_event.kline.low.parse::<f64>().unwrap();
            let mut volume = kline_event.kline.volume.parse::<f64>().unwrap();
            let mut taker_buy_base_asset_volume = kline_event.kline.taker_buy_base_asset_volume.parse::<f64>().unwrap();
            let mut taker_buy_quote_asset_volume = kline_event.kline.taker_buy_quote_asset_volume.parse::<f64>().unwrap();
            let mut is_final_bar = kline_event.kline.is_final_bar;


            let mut kline_data = KlineData {
                symbol,
                open,
                close,
                high,
                low,
                volume,
                taker_buy_base_asset_volume,
                taker_buy_quote_asset_volume,
                is_final_bar,
            };
            let mut streamWholeData = StreamData::Kline(kline_data);
            //process_trade(tick_event_vector, &mut None);
            let sender_clone = sender.clone();

            tokio::spawn(async move {
                println!("[DEBUG] Sending Kline data to channel");
                match sender_clone.send(streamWholeData).await {
                    Ok(_) => println!("[DEBUG] Kline data sent successfully"),
                    Err(e) => println!("[ERROR] Failed to send Kline data: {:?}", e),
                }
            });
            Ok(())
        },
        _=> Ok(())
    }
}
fn handle_event_DepthOrderBook(event: &WebsocketEvent, sender: &mpsc::Sender<StreamData>) -> Result<(), ()> {
    println!("handle_event_DepthOrderBookE");
    match event {
        WebsocketEvent::DepthOrderBook(depth) => {
            //let mut tick_event_vector : Vec<String> = Vec::new();

            println!("Монета: {}", depth.symbol);
            //tick_event_vector.push("DepthOrderBook".to_string());
            //tick_event_vector.push((&depth.symbol).to_string());
            let mut symbol = depth.symbol.to_string();


            for bid in depth.bids.iter(){
                println!("Монета: {}. Покупка по цене {} на количество {}",depth.symbol, bid.price, bid.qty);
                //tick_event_vector.push(format!("{}, {}", &bid.price, &bid.qty));

                let mut bids: Vec<OrderBookEntry> = depth.bids.iter().filter_map(|bid| {
                    let price = bid.price;
                    let quantity = bid.qty;
                    Some(OrderBookEntry { price, quantity })
                })
                    .collect();
                let mut asks: Vec<OrderBookEntry> = depth.asks.iter().filter_map(|ask| {
                    let price = ask.price;
                    let quantity = ask.qty;
                    Some(OrderBookEntry { price, quantity })
                })
                    .collect();

                let order_book_data = OrderBookData {
                    symbol: depth.symbol.to_string(),
                    bids,
                    asks,
                    first_update_id: depth.first_update_id,
                    final_update_id: depth.final_update_id,
                };
                let streamWholeData = StreamData::DepthOrderBook(order_book_data);

                let sender = sender.clone();
                tokio::spawn(async move {
                    println!("[DEBUG] Sending data to channel");
                    if let Err(e) = sender.send(streamWholeData).await {
                        println!("[ERROR] Failed to send data: {:?}", e);
                    }
                });
            }

            /*
            for ask in depth.asks.iter() {
                println!("Монета: {}. Продажа по цене {} на количество {}",depth.symbol, ask.price, ask.qty);
                tick_event_vector.push(format!("{}, {}", &ask.price, &ask.qty));
            }
            println!("Монета: {}. Первая часть апдейта: {}",depth.symbol,depth.first_update_id);
            println!("Монета: {}. Финальный апдейт: {}",depth.symbol, depth.final_update_id);

            tick_event_vector.push(depth.first_update_id.to_string());
            tick_event_vector.push(depth.final_update_id.to_string());
            */
            //process_trade(tick_event_vector, &mut None);

            Ok(())
        },
        _ => Ok(())
    }
}
fn handle_event_BookTicker(event: &WebsocketEvent, sender: &mpsc::Sender<StreamData>) -> Result<(), ()> {
    //println!("handle_event_BookTicker");
    match event {
        WebsocketEvent::BookTicker(book) => {
            println!("Монета: {}, Лучшая ставка: {}",
                     book.symbol, book.best_bid);
            println!("Монета: {}, Объём по лучшей цене покупки: {}",
                     book.symbol, book.best_bid_qty);
            println!("Монета: {}, Лучшая цена продажи: {}",
                     book.symbol, book.best_ask);
            println!("Монета: {}, Объём по лучшей цене продажи: {}",
                     book.symbol, book.best_ask_qty);

            let mut symbol = book.symbol.to_string();
            let mut best_bid = book.best_bid.parse::<f64>().unwrap();
            let mut best_bid_qty = book.best_bid_qty.parse::<f64>().unwrap();
            let mut best_ask = book.best_ask.parse::<f64>().unwrap();
            let mut best_ask_qty = book.best_ask_qty.parse::<f64>().unwrap();

            let mut BookTickerData = BookTickerData{
                symbol,
                best_bid,
                best_bid_qty,
                best_ask,
                best_ask_qty
            };
            let mut streamWholeData = StreamData::BookTicker(BookTickerData);
            //let mut tick_event_vector : Vec<String> = Vec::new();
            //tick_event_vector.push("BookTicker".to_string());
            //tick_event_vector.push((&book.best_bid).to_string());
            //tick_event_vector.push((&book.best_bid_qty).to_string());
            //tick_event_vector.push((&book.best_ask).to_string());
            //tick_event_vector.push((&book.best_ask_qty).to_string());
            //process_trade(tick_event_vector, &mut None);
            let sender = sender.clone();
            tokio::spawn(async move {
                println!("[DEBUG] Sending data to channel");
                if let Err(e) = sender.send(streamWholeData).await {
                    println!("[ERROR] Failed to send data: {:?}", e);
                }
            });
            Ok(())
        },
        _=> Ok(())
    }
}
async fn snapshotUpdater (receiver: &mut mpsc::Receiver<StreamData>, market_snapshot: &Arc<RwLock<MarketSnapshot>>) {
    print!("[TEST] snapshotUpdater start");
    while let Some(msg) = receiver.recv().await {
        {
            println!("[TEST] snapshotUpdater update");

            let mut snapshot = market_snapshot.write().unwrap();

            match msg {
                StreamData::AggTrade(data) => snapshot.last_agg_trade = Some(data),
                StreamData::Kline(data) => snapshot.last_kline = Some(data),
                StreamData::DepthOrderBook(data) => snapshot.last_order_book = Some(data),
                StreamData::BookTicker(data) => snapshot.last_book_ticker = Some(data),
            }
        }
        run_all_strategies(Arc::clone(market_snapshot)).await;
    }
}
async fn run_all_strategies (snapshot: Arc<RwLock<MarketSnapshot>>) {
    println!("[DEBUG] Running strategies");
    let snap = snapshot.read().unwrap();

    println!("AggTrade data present: {}", snap.last_agg_trade.is_some());
    println!("Kline data present: {}", snap.last_kline.is_some());
    println!("OrderBook data present: {}", snap.last_order_book.is_some());
    println!("BookTicker data present: {}", snap.last_book_ticker.is_some());

    let agg_data = snap.last_agg_trade.clone();
    let kline_data = snap.last_kline.clone();
    let depth_data = snap.last_order_book.clone();
    let book_ticker_data = snap.last_book_ticker.clone();

    if let (Some(agg), Some(kline), Some(book)) = (agg_data.clone(), kline_data.clone(), book_ticker_data.clone()) {
        Momentum_strategy(agg, kline, book);
    }

    if let (Some(agg), Some(kline)) = (agg_data.clone(), kline_data.clone()) {
        Mean_Reversion(agg, kline);
    }

    if let (Some(depth), Some(book)) = (depth_data, book_ticker_data) {
        Order_Book_Pressure(depth, book);
    }

    drop(snap);
}

fn Momentum_strategy (aggTrade: AggTradeData, kline: KlineData, bookTicker: BookTickerData) {
    //if let (Some(agg), Some(kl), Some(ticker)) = (&aggTrade, &kline, &bookTicker) {}

    println!("AggTrade: average_price: {}, current_close: {}, Kline: open {}, close: {}, is_final_bar: {}, BookTicker: best_bid: {}, best_ask: {}",
             aggTrade.average_price, aggTrade.current_close, kline.open, kline.close, kline.is_final_bar, bookTicker.best_bid, bookTicker.best_ask);
}

fn Mean_Reversion(aggTrade: AggTradeData, kline: KlineData) {
    //if let (Some(agg), Some(kl)) = (&aggTrade, &kline) {}

    println!("AggTraade: average_price: {}, Kline: open {}, close {}, hight: {}, low: {}",
             aggTrade.average_price, kline.open, kline.close, kline.high, kline.low);
}

fn Order_Book_Pressure (depth: OrderBookData, book: BookTickerData) {
    //if let (Some(dep), Some(book)) = (&depth, &book) {}

    println!("DepthOrderBook: bid.price:{:?}, bid.qty:{:?}, ask.price:{:?}, ask.qty:{:?}, BookTicker: best_bid_qty: {}, best_ask_qty: {}",
             depth.bids.get(0), depth.bids.get(1), depth.asks.get(0), depth.asks.get(1), book.best_bid_qty, book.best_ask_qty);
}
fn Volume_spike (kline_data: KlineData){
    //if let (Some(kl)) = &kline_data {}

    println!("Kline: volume: {}, taker_buy_base_asset_volume: {}, is_final_bar:{} ",
             kline_data.volume, kline_data.taker_buy_base_asset_volume, kline_data.is_final_bar);
}


#[no_mangle]
pub extern "C" fn run_trading_bot(
    coins: *const c_char,
    stream_types: *const c_char,
    keep_running: *mut std::ffi::c_int,
) -> bool {
    // Безопасно преобразуем C-строки в Rust String
    let coins_cstr = unsafe { CStr::from_ptr(coins) };
    let stream_types_cstr = unsafe { CStr::from_ptr(stream_types) };

    let input_coin = coins_cstr.to_string_lossy().into_owned();
    let input_stream_types = stream_types_cstr.to_string_lossy().into_owned();

    // Создаем runtime для tokio
    let rt = tokio::runtime::Runtime::new().unwrap();

    // Запускаем асинхронный код
    rt.block_on(async {
        let (sender, mut receiver) = mpsc::channel(5000);

        let snapshot = Arc::new(RwLock::new(MarketSnapshot {
            last_agg_trade: None,
            last_kline: None,
            last_order_book: None,
            last_book_ticker: None,
        }));

        let snapshot_clone = Arc::clone(&snapshot);
        let snapshot_updater_handle = tokio::spawn(async move {
            snapshotUpdater(&mut receiver, &snapshot_clone).await;
        });

        let coins_vector_string: Vec<String> = input_coin.trim().split(",").map(|s| s.trim().to_string()).collect();

        let stream_types_string: Vec<String> = input_stream_types
            .trim()
            .split(',')
            .map(|s| match s.trim().parse::<i32>() {
                Ok(1) => "arr".to_string(),
                Ok(2) => "kline_1m".to_string(),
                Ok(3) => "depth".to_string(),
                Ok(4) => "bookTicker".to_string(),
                _ => "Unknown stream type".to_string(),
            })
            .collect();

        let coins_vector: Vec<&str> = input_coin.trim().split(",").map(|s| s.trim()).collect();

        let endpoints: Vec<String> = coins_vector.iter().flat_map(|coin| {
            stream_types_string.iter().map(move |stream| {
                let formatted_stream = format!("{}@{}", coin.to_lowercase(), stream);
                if formatted_stream.contains(&"@arr") {
                    "!ticker@arr".to_string()
                } else {
                    formatted_stream
                }
            })
        }).collect();

        let keep_running_flag = Arc::new(AtomicBool::new(true));
        let keep_running_ws = keep_running_flag.clone();
        let keep_running_strategy = keep_running_flag.clone();

        let websocket_handle = tokio::spawn(async move {
            let mut web_socket = WebSockets::new(move |event: WebsocketEvent| {
                let stream_types_vector: Vec<i32> = input_stream_types
                    .trim()
                    .split(',')
                    .filter_map(|s| s.trim().parse::<i32>().ok())
                    .collect();

                for stream_type in &stream_types_vector {
                    match stream_type {
                        1 => handle_event_AggTrade(&event, &sender, &coins_vector_string).unwrap_or_else(|_| ()),
                        2 => handle_event_Kline(&event, &sender).unwrap_or_else(|_| ()),
                        3 => handle_event_DepthOrderBook(&event, &sender).unwrap_or_else(|_| ()),
                        4 => handle_event_BookTicker(&event, &sender).unwrap_or_else(|_| ()),
                        _ => println!("Unknown stream type: {}", stream_type),
                    }
                }
                Ok(())
            });

            if let Err(e) = web_socket.connect_multiple_streams(&endpoints) {
                println!("Websocket connection error: {:?}", e);
                return;
            }

            if let Err(e) = web_socket.event_loop(&keep_running_ws) {
                println!("Error in event_loop: {:?}", e);
            }

            web_socket.disconnect().unwrap();
        });

        let strategy_handle = {
            let snapshot = Arc::clone(&snapshot);
            tokio::spawn(async move {
                while keep_running_strategy.load(Ordering::Relaxed) {
                    run_all_strategies(snapshot.clone()).await;
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                }
            })
        };

        // Ожидаем сигнала остановки через переданный указатель
        unsafe {
            while *keep_running != 0 {
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            keep_running_flag.store(false, Ordering::Relaxed);
        }

        let _ = tokio::join!(websocket_handle, strategy_handle, snapshot_updater_handle);
    });

    true
}


//main v1
/*
#[tokio::main]

async fn main() {
    use std::sync::{Arc, Mutex};
    use std::sync::atomic::{AtomicBool, Ordering};
    use tokio::sync::broadcast;

    //let strategy = Box::new(MomentumStrategy::new(2));
    //let (tx, rx) = broadcast::channel(5000);

    let mut all_trades_vec_of_vec: Vec<Vec<String>> = Vec::new();

    let (sender, mut receiver): (mpsc::Sender<StreamData>, mpsc::Receiver<StreamData>) = mpsc::channel(5000);

    let snapshot = Arc::new(RwLock::new(MarketSnapshot {
        last_agg_trade: None,
        last_kline: None,
        last_order_book: None,
        last_book_ticker: None,
    }));
    let snapshot_clone = Arc::clone(&snapshot);

    let snapshot_updater_handle = tokio::spawn(async move {
        snapshotUpdater(&mut receiver, &snapshot_clone).await;
    });

    let mut inputCoin = String::new();
    let mut inputStream_type_int_asString = String::new();


    println!("Wrtite name of the coin");
    io::stdin().read_line(&mut inputCoin).expect("Failed to read line");

    let coins_vector_string: Vec<String> = inputCoin.trim().split(",").map(|s| s.trim().to_string()).collect();

    println!("{},: inputCoin", inputCoin);

    println!("Choose type of the stream: 1)Agg Trade, 2)Kline, 3)Depth Order Book, 4)Book Ticker");
    io::stdin().read_line(&mut inputStream_type_int_asString).expect("Failed to read line");


    let stream_types_string: Vec<String> = inputStream_type_int_asString
        .trim()
        .split(',')
        .map(|s| match s.trim().parse::<i32>() {
            Ok(1) => "arr".to_string(),
            Ok(2) => "kline_1m".to_string(),
            Ok(3) => "depth".to_string(),
            Ok(4) => "bookTicker".to_string(),
            _ => "Unknown stream type".to_string(),
        })
        .collect();
    let coins_vector: Vec<&str> = inputCoin.trim().split(",").map(|s| s.trim()).collect();

    //let type_vector: Vec<&str> = stream_types_string.trim().split(',').map(|s| s.trim()).collect();

    //WB_socket();
    let endpoints: Vec<String> = coins_vector.iter().flat_map(|coin| {
        stream_types_string.iter().map(move |stream| {
            let formatted_stream = format!("{}@{}", coin.to_lowercase(), stream);
            if formatted_stream.contains(&"@arr"){
                "!ticker@arr".to_string()
            }else {
                formatted_stream
            }} )
    }).collect();
    println!("{}", endpoints.join(", "));

    let keep_running = Arc::new(AtomicBool::new(true));

    // Клонируем Arc для вебсокета
    let keep_running_ws = keep_running.clone();

    let keep_running_strategy = keep_running.clone();

    // Запускаем вебсокет в отдельной задаче
    let websocket_handle = tokio::spawn(async move {
        let mut web_socket = WebSockets::new(move |event: WebsocketEvent| {
            let stream_types_vector: Vec<i32> = inputStream_type_int_asString
                .trim()
                .split(',')
                .filter_map(|s| s.trim().parse::<i32>().ok())
                .collect();

            for stream_type in &stream_types_vector {
                match stream_type {
                    1 => handle_event_AggTrade(&event, &sender, &coins_vector_string).unwrap_or_else(|_| ()),
                    2 => handle_event_Kline(&event, &sender).unwrap_or_else(|_| ()),
                    3 => handle_event_DepthOrderBook(&event, &sender).unwrap_or_else(|_| ()),
                    4 => handle_event_BookTicker(&event, &sender).unwrap_or_else(|_| ()),
                    _ => println!("Неизвестный тип потока: {}", stream_type),
                }
            }
            Ok(())
        });

        if let Err(e) = web_socket.connect_multiple_streams(&endpoints) {
            println!("Ошибка подключения вебсокета: {:?}", e);
            return;
        }

        if let Err(e) = web_socket.event_loop(&keep_running_ws) {
            println!("Ошибка в event_loop: {:?}", e);
        }

        web_socket.disconnect().unwrap();
    });

    // Запускаем стратегии и обработку снимков
    let strategy_handle = {
        let snapshot = Arc::clone(&snapshot);
        tokio::spawn(async move {
            while keep_running_strategy.load(Ordering::Relaxed) {
                run_all_strategies(snapshot.clone()).await;
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
        })
    };

    // Ожидаем Ctrl+C для graceful shutdown
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            println!("Получен сигнал Ctrl+C, завершаем работу...");
            keep_running.store(false, Ordering::Relaxed);
        }
    }

    // Даём время на завершение задач
    let _ = tokio::join!(websocket_handle, strategy_handle);
}
*/



// main v2
/*#[tokio::main]
async fn main() {
    use std::sync::{Arc, Mutex};
    use std::sync::atomic::{AtomicBool, Ordering};
    use tokio::sync::broadcast;

    //let strategy = Box::new(MomentumStrategy::new(2));
    //let (tx, rx) = broadcast::channel(5000);

    let mut all_trades_vec_of_vec: Vec<Vec<String>> = Vec::new();

    let (sender, mut receiver): (tokio::sync::mpsc::Sender<StreamData>, tokio::sync::mpsc::Receiver<StreamData>) = tokio::sync::mpsc::channel(5000);

    let snapshot = Arc::new(RwLock::new(MarketSnapshot {
        last_agg_trade: None,
        last_kline: None,
        last_order_book: None,
        last_book_ticker: None,
    }));

    let snapshot_updater_handle = {
        let snapshot = Arc::clone(&snapshot);
        tokio::spawn(snapshotUpdater(&mut receiver, &snapshot))
    };

    let mut inputCoin = String::new();
    let mut inputStream_type_int_asString = String::new();


    println!("Wrtite name of the coin");
    io::stdin().read_line(&mut inputCoin).expect("Failed to read line");

    let coins_vector_string: Vec<String> = inputCoin.trim().split(",").map(|s| s.trim().to_string()).collect();

    println!("{},: inputCoin", inputCoin);

    println!("Choose type of the stream: 1)Agg Trade, 2)Kline, 3)Depth Order Book, 4)Book Ticker");
    io::stdin().read_line(&mut inputStream_type_int_asString).expect("Failed to read line");


    let stream_types_string: Vec<String> = inputStream_type_int_asString
        .trim()
        .split(',')
        .map(|s| match s.trim().parse::<i32>() {
            Ok(1) => "arr".to_string(),
            Ok(2) => "kline_1m".to_string(),
            Ok(3) => "depth".to_string(),
            Ok(4) => "bookTicker".to_string(),
            _ => "Unknown stream type".to_string(),
        })
        .collect();
    let coins_vector: Vec<&str> = inputCoin.trim().split(",").map(|s| s.trim()).collect();

    //let type_vector: Vec<&str> = stream_types_string.trim().split(',').map(|s| s.trim()).collect();

    //WB_socket();
    let endpoints: Vec<String> = coins_vector.iter().flat_map(|coin| {
        stream_types_string.iter().map(move |stream| {
            let formatted_stream = format!("{}@{}", coin.to_lowercase(), stream);
            if formatted_stream.contains(&"@arr"){
                "!ticker@arr".to_string()
            }else {
                formatted_stream
            }} )
    }).collect();
    println!("{}", endpoints.join(", "));

    let keep_running =  Arc::new(AtomicBool::new(true));

    tokio::spawn(async move {
        let mut web_socket = WebSockets::new(|event: WebsocketEvent| {
            //println!("Получено событие: {:?}", event);
            let stream_types_vector: Vec<i32> = inputStream_type_int_asString
                .trim()
                .split(',')
                .filter_map(|s| s.trim().parse::<i32>().ok()) // Преобразуем в числа, игнорируем ошибки
                .collect();


            for stream_type in &stream_types_vector {
                match stream_type {
                    1 => handle_event_AggTrade(&event, &sender, &coins_vector_string).unwrap_or_else(|_| ()),
                    2 => handle_event_Kline(&event,&sender).unwrap_or_else(|_| ()),
                    3 => handle_event_DepthOrderBook(&event, &sender).unwrap_or_else(|_| ()),
                    4 => handle_event_BookTicker(&event, &sender).unwrap_or_else(|_| ()),
                    _ => println!("Неизвестный тип потока: {}", stream_type),
                }
            }
            Ok(())
        });

        web_socket.connect_multiple_streams(&endpoints).unwrap();

        let strategy_handle = {
            let snapshot = Arc::clone(&snapshot);
            tokio::spawn(async move {
                loop {
                    run_all_strategies(snapshot.clone()).await;
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                }
            })
        };

        tokio::try_join!(snapshot_updater_handle, strategy_handle).unwrap();

        let keep_running_clone =  Arc::clone(&keep_running);
        tokio::spawn(async move {
            if let Err(e) = web_socket.event_loop(&keep_running_clone) {
                println!("WebSocket error: {:?}", e);
            }
        });
        web_socket.disconnect().unwrap();
    });



}
 */