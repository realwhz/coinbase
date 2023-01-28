#![allow(non_snake_case)]

#[allow(unused_imports)]
use coinbase::WS_PRODUCTION_URL;
#[allow(unused_imports)]
use coinbase::WS_SANDBOX_URL;
use serde_json;
use std::env;
use std::io;
use std::sync::{Arc, Mutex};
use tungstenite::{connect, Message};
use url::Url;

mod book;

type Book = Arc<Mutex<book::Book>>;

#[tokio::main]
async fn main() {
    let local_book = Arc::new(Mutex::new(book::Book::new()));

    {
        println!("Connecting to the Coinbase feed wss server");
        let local_book = local_book.clone();
        let instrument = vec![env::args().nth(1).expect("./coinbase instrument")];
        tokio::spawn(async move { Subscribe(WS_SANDBOX_URL, &instrument, local_book).await });
    }

    loop {
        let input =
            read_from_stdin("Choose operation [bestbid, bestask, fullbook, quit], confirm with return:");
        match input.as_str() {
            "bestbid" => {
                let book = local_book.lock().unwrap();
                if let Some((price, size)) = book.BestBidPrice() {
                    println!("Best bid price {} with size {}", price, size);
                } else {
                    println!("Empty bid book");
                }
            }
            "bestask" => {
                let book = local_book.lock().unwrap();
                if let Some((price, size)) = book.BestAskPrice() {
                    println!("Best ask price {} with size {}", price, size);
                } else {
                    println!("Empty ask book");
                }
            }
            "fullbook" => {
                let book = local_book.lock().unwrap();
                book.PrintFullBook();
            }
            "quit" => {
                println!("Quitting...");
                break;
            }
            _ => {
                println!("Invalid option: '{}'", input);
            }
        }
    }

    println!("Bye!");
    std::process::exit(0);
}

async fn Subscribe(url: &str, instrument: &Vec<String>, book: Book) {
    // Connect to the Coinbase WSS server
    let (mut socket, _) = connect(Url::parse(url).unwrap()).expect("Cannot connect");
    // Subscribe to the server
    let subscription = serde_json::json!({
        "type": "subscribe",
        "product_ids": instrument,
        "channels": ["level2"]
    });
    socket
        .write_message(Message::Text(subscription.to_string()))
        .unwrap();

    // Loop forever, handling parsing each message
    loop {
        let msg = socket.read_message().expect("Error reading message");
        let msg = match msg {
            Message::Text(s) => s,
            _ => {
                println!("Unexpected message type");
                continue;
            }
        };
        let parsed: serde_json::Value = serde_json::from_str(&msg).expect("Can't parse to JSON");
        if parsed["type"].to_string() == "\"snapshot\"" {
            let data: serde_json::Result<book::SnapshotData> = serde_json::from_str(&msg);
            if data.is_err() {
                println!("Failed to recognize {} due to {}", msg, data.err().unwrap());
                continue;
            }
            let mut local_book = book.lock().unwrap();
            local_book.UpdateFullBook(data.unwrap());
        } else if parsed["type"].to_string() == "\"l2update\"" {
            let data: serde_json::Result<book::L2UpdateData> = serde_json::from_str(&msg);
            if data.is_err() {
                println!("Failed to recognize {} due to {}", msg, data.err().unwrap());
                continue;
            }
            let mut local_book = book.lock().unwrap();
            local_book.UpdateBook(data.unwrap());
        } else if parsed["type"].to_string() == "\"subscriptions\"" {
            continue;
        } else {
            println!("Unexpected message type {}", parsed["type"]);
            continue;
        }
    }
}

fn read_from_stdin(label: &str) -> String {
    let mut buffer = String::new();
    println!("{}", label);
    io::stdin()
        .read_line(&mut buffer)
        .expect("Couldn't read from stdin");
    buffer.trim().to_owned()
}
