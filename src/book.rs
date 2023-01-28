use serde::de;
use serde::{Deserialize, Deserializer};
use std::collections::BTreeMap;
use ordered_float::*;

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
    bids: BTreeMap<OrderedFloat<f64>, f64>,
    /// The "Ask" or "Sell" side of the order book. Ordered by price mapped to size.
    /// Scaled by SCALE
    asks: BTreeMap<OrderedFloat<f64>, f64>,
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
            Some((**price, *size))
        }
    }

    pub fn BestAskPrice(&self) -> Option<(f64, f64)> {
        if self.asks.len() == 0 {
            None
        } else {
            let (price, size) = self.asks.iter().next().unwrap();
            Some((**price, *size))
        }
    }

    pub fn MidPrice(&self) -> Option<f64> {
        if self.BestBidPrice().is_none() || self.BestAskPrice().is_none() || self.BestBidPrice().unwrap().0 > self.BestAskPrice().unwrap().0  {
            None
        } else {
            Some((self.BestAskPrice().unwrap().0 - self.BestBidPrice().unwrap().0) / 2.0)
        }
    }

    pub fn BidAskSpread(&self) -> Option<f64> {
        if self.BestBidPrice().is_none() || self.BestAskPrice().is_none() || self.BestBidPrice().unwrap().0 > self.BestAskPrice().unwrap().0  {
            None
        } else {
            Some((self.BestAskPrice().unwrap().0 - self.BestBidPrice().unwrap().0) / self.BestAskPrice().unwrap().0)
        }
    }

    pub fn PrintFullBook(&self) {
        println!("bids:\n--------------------------------");
        println!("Price\tSize");
        for item in self.bids.iter().rev() {
            println!("{}\t{}", *item.0, *item.1);
        }
        println!("\nasks:\n--------------------------------");
        println!("Price\tSize");
        for item in &self.asks {
            println!("{}\t{}", *item.0, *item.1);
        }
    }

    pub fn UpdateFullBook(&mut self, data: SnapshotData) {
        self.bids.clear();
        self.asks.clear();
        for item in &data.bids {
            if self
                .bids
                .insert(
                    OrderedFloat(item.price),
                    item.size,
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
                    OrderedFloat(item.price),
                    item.size,
                )
                .is_none()
            {
                continue;
            }
        }
    }

    pub fn UpdateBook(&mut self, data: L2UpdateData) {
        for item in &data.changes {
            if item.side == "buy" {
                if item.size == 0.0 {
                    self.bids.remove(&OrderedFloat(item.price));
                } else {
                    self.bids.insert(OrderedFloat(item.price), item.size);
                }
            } else if item.side == "sell" {
                if item.size == 0.0 {
                    self.asks.remove(&OrderedFloat(item.price));
                } else {
                    self.asks.insert(OrderedFloat(item.price), item.size);
                }
            } else {
                unreachable!()
            }
        }
    }
}
