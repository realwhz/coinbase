use coinbase::SCALE;
use serde::de;
use serde::{Deserialize, Deserializer};
use std::collections::BTreeMap;

#[derive(Debug, Deserialize)]
pub struct PriceData {
    #[serde(deserialize_with = "de_float_from_str")]
    pub price: f64,
    #[serde(deserialize_with = "de_float_from_str")]
    pub size: f64,
}

#[derive(Debug, Deserialize)]
pub struct PriceDataWithSide {
    pub side: String,
    #[serde(deserialize_with = "de_float_from_str")]
    pub price: f64,
    #[serde(deserialize_with = "de_float_from_str")]
    pub size: f64,
}

// {
//     "type": "snapshot",
//     "product_id": "BTC-USD",
//     "bids": [["10101.10", "0.45054140"]],
//     "asks": [["10102.55", "0.57753524"]]
// }
#[derive(Debug, Deserialize)]
pub struct SnapshotData {
    //pub _type: String,
    pub product_id: String,
    pub bids: Vec<PriceData>,
    pub asks: Vec<PriceData>,
}

// {
//     "type": "l2update",
//     "product_id": "BTC-USD",
//     "changes": [
//       [
//         "buy",
//         "22356.270000",
//         "0.00000000"
//       ],
//       [
//         "buy",
//         "22356.300000",
//         "1.00000000"
//       ]
//     ],
//     "time": "2022-08-04T15:25:05.010758Z"
// }
#[derive(Debug, Deserialize)]
pub struct L2UpdateData {
    //pub _type: String,
    pub product_id: String,
    pub changes: Vec<PriceDataWithSide>,
    // ignore time
    //pub time: String,
}

pub fn de_float_from_str<'a, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'a>,
{
    let str_val = String::deserialize(deserializer)?;
    str_val.parse::<f64>().map_err(de::Error::custom)
}

pub struct Book {
    /// The "Bid" or "Buy" side of the order book. Ordered by price mapped to size.
    /// Scaled by SCALE
    bids: BTreeMap<u64, u64>,
    /// The "Ask" or "Sell" side of the order book. Ordered by price mapped to size.
    /// Scaled by SCALE
    asks: BTreeMap<u64, u64>,
}

impl Book {
    pub fn new() -> Self {
        Book {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
        }
    }

    pub fn BestBidPrice(&self) -> Option<(f64, f64)> {
        if self.bids.len() == 0 {
            None
        } else {
            let (price, size) = self.bids.iter().next_back().unwrap();
            Some((*price as f64 / SCALE as f64, *size as f64 / SCALE as f64))
        }
    }

    pub fn BestAskPrice(&self) -> Option<(f64, f64)> {
        if self.asks.len() == 0 {
            None
        } else {
            let (price, size) = self.asks.iter().next().unwrap();
            Some((*price as f64 / SCALE as f64, *size as f64 / SCALE as f64))
        }
    }

    pub fn PrintFullBook(&self) {
        println!("bids:\n--------------------------------");
        println!("Price\tSize");
        for item in self.bids.iter().rev() {
            println!("{}\t{}", *item.0 as f64 / SCALE as f64, *item.1 as f64 / SCALE as f64);
        }
        println!("\nasks:\n--------------------------------");
        println!("Price\tSize");
        for item in &self.asks {
            println!("{}\t{}", *item.0 as f64 / SCALE as f64, *item.1 as f64 / SCALE as f64);
        }
    }

    pub fn UpdateFullBook(&mut self, data: SnapshotData) {
        self.bids.clear();
        self.asks.clear();
        for item in &data.bids {
            if self
                .bids
                .insert(
                    (item.price * SCALE as f64) as u64,
                    (item.size * SCALE as f64) as u64,
                )
                .is_none()
            {
                continue;
            }
        }
        for item in &data.asks {
            if self
                .asks
                .insert(
                    (item.price * SCALE as f64) as u64,
                    (item.size * SCALE as f64) as u64,
                )
                .is_none()
            {
                continue;
            }
        }
    }

    pub fn UpdateBook(&mut self, data: L2UpdateData) {
        for item in &data.changes {
            let _price = (item.price * SCALE as f64) as u64;
            let _size = (item.size * SCALE as f64) as u64;
            if item.side == "buy" {
                if _size == 0 {
                    self.bids.remove(&_price);
                } else {
                    self.bids.insert(_price, _size);
                }
            } else if item.side == "sell" {
                if _size == 0 {
                    self.asks.remove(&_price);
                } else {
                    self.asks.insert(_price, _size);
                }
            } else {
                unreachable!()
            }
        }
    }
}
