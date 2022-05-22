mod models;

use futures_util::stream::StreamExt;
use futures_util::SinkExt;
use structopt::StructOpt;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite;
use tokio_tungstenite::tungstenite::Message;
use tracing::*;
use url::Url;

const BITSMAP_WS_API: &str = "wss://ws.bitstamp.net";

#[derive(Debug, StructOpt)]
#[structopt(name = "bitsmap client", about = "An example of StructOpt usage.")]
struct Opt {
    #[structopt(short, long, default_value = "btcusdt")]
    symbol: String,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt().init();
    let opt = Opt::from_args();
    let bitsmap_url = format!("{}", BITSMAP_WS_API,);
    let (socket, response) = connect_async(Url::parse(&bitsmap_url)?).await?;
    info!("Connected to bitsmap stream.");
    info!("HTTP status code: {}", response.status());
    info!(headers=?response.headers());
    
    // The type of `json` is `serde_json::Value`
    let subscribe = serde_json::json!({
        "event": "bts:subscribe",
        "data": {
            "channel": "order_book_".to_string() + opt.symbol.to_lowercase().as_str(),
        }
    });
    let (mut write, mut read) = socket.split();
    // subscribe
    write.send(serde_json::to_vec(&subscribe)?.into()).await?;

    // So read and write will be in different tokio tasks
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let handle = tokio::spawn(async move {
        while let Some(inner) = rx.recv().await {
            write.send(inner).await?;
        }
        Result::<_, anyhow::Error>::Ok(())
    });

    while let Some(msg) = read.next().await {
        match msg? {
            Message::Text(str) => {
                let parsed : models::BitsMap = serde_json::from_str(&str)?;
                let book = match parsed.data {
                    models::Data::Book(inner) => inner,
                    _ => {
                        info!(received_text=?parsed);
                        continue;
                    } 
                };
                for i in 0..std::cmp::min(10, book.asks.len()) {
                    info!(
                        "{}. ask: {}, size: {}",
                        i, book.asks[i].price, book.asks[i].qty
                    );
                }
            }
            Message::Ping(p) => {
                info!("Ping message received! {:?}", String::from_utf8_lossy(&p));
                let pong = tungstenite::Message::Pong(vec![]);
                tx.send(pong)?;
            }
            Message::Pong(p) => info!("Pong received: {:?}", p),

            Message::Close(c) => {
                info!("Close received from bitsmap: {:?}", c);
                break;
            }
            unexpected_msg => {
                warn!(?unexpected_msg);
            }
        }
    }

    drop(read);
    handle.await??;

    Ok(())
}
