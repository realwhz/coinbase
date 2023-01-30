use ordered_float::*;
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
        if self.BestBidPrice().is_none()
            || self.BestAskPrice().is_none()
            || self.BestBidPrice().unwrap().0 > self.BestAskPrice().unwrap().0
        {
            None
        } else {
            Some((self.BestAskPrice().unwrap().0 + self.BestBidPrice().unwrap().0) / 2.0)
        }
    }

    pub fn BidAskSpread(&self) -> Option<f64> {
        if self.BestBidPrice().is_none()
            || self.BestAskPrice().is_none()
            || self.BestBidPrice().unwrap().0 > self.BestAskPrice().unwrap().0
        {
            None
        } else {
            Some(
                (self.BestAskPrice().unwrap().0 - self.BestBidPrice().unwrap().0)
                    / self.BestAskPrice().unwrap().0,
            )
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
                .insert(OrderedFloat(item.price), item.size)
                .is_none()
            {
                continue;
            }
        }
        for item in &data.asks {
            if self
                .asks
                .insert(OrderedFloat(item.price), item.size)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_book_updatefullbook() {
        let mut book = Book::new();
        let msg = r#"{"type":"snapshot","product_id":"BTC-USD","asks":[["93811.50","0.00003210"],["93811.51","0.00001070"],["100000.00","9991.07093828"],["101199.04","0.00100000"],["1000000.00","107239.00019802"],["2000000.00","1.00700000"],["5803600.00","1.00000000"],["5884800.00","1090.00000000"],["5885000.00","8.00000000"],["8000070.30","0.02570000"],["9999999.02","0.00100000"],["10000000.00","1.00000000"],["10000000.01","0.00400000"],["10000009.00","0.50000000"],["11099891.00","0.05000000"],["40000000.00","0.15013000"],["99999999.00","0.02000000"],["100000000.00","1.00000000"],["200000000.00","0.00000001"],["261642900.00","0.00445499"],["999999999.39","0.18599975"],["1000000000.00","4.00000000"],["1200000000.00","0.00000001"],["1222222222.00","10.84329557"],["2000000000.00","0.00000001"],["3300000000.00","0.00000001"],["5000000000.00","0.00000001"],["5400000000.00","0.00000001"],["5600000000.00","0.00000002"],["7200000000.00","0.00000001"],["7800000000.00","0.00000001"],["8000000000.00","0.00000001"],["8700000000.00","0.00000001"],["9100000000.00","0.00000001"],["9700000000.00","0.00000001"]],"bids":[["93711.51","1.30000000"],["93711.50","1.30000000"],["6240.00","0.00486124"],["3014.86","0.00033302"],["3014.85","0.01000000"],["300.00","35.00000000"],["222.00","2.10000000"],["200.90","0.05955202"],["200.00","200.99999500"],["139.50","9981.63682939"],["111.00","1.00000000"],["100.98","493.82400000"],["100.00","27.03234510"],["90.45","12.00000000"],["50.00","0.30000000"],["20.00","21.30000000"],["15.00","1.00000000"],["12.00","1.00000000"],["10.00","100430.00000000"],["8.00","2.00000000"],["6.64","12.16587225"],["6.00","1.00000000"],["5.00","926.00000000"],["4.43","1.00000000"],["3.00","1001.00000000"],["2.50","3.00000000"],["2.25","3.00000000"],["2.00","32.80634018"],["1.20","1.00000000"],["1.12","2.00000000"],["1.00","1354208.42448538"],["0.95","0.01000000"],["0.79","400.00000000"],["0.70","0.08000000"],["0.56","26.00000000"],["0.50","1.00000000"],["0.42","19500.00000000"],["0.20","0.00440000"],["0.19","1.00000000"],["0.16","12.00000000"],["0.15","0.01000000"],["0.12","0.54300000"],["0.10","1002.55510000"],["0.01","110092.45165027"]]}"#;
        let data = serde_json::from_str(&msg).expect("Invalid JSON msg");
        book.UpdateFullBook(data);
        assert_eq!(book.asks.len(), 35);
        assert_eq!(
            book.asks.first_key_value().unwrap(),
            (&OrderedFloat(93811.50), &0.00003210)
        );
        assert_eq!(book.bids.len(), 44);
        assert_eq!(
            book.bids.last_key_value().unwrap(),
            (&OrderedFloat(93711.51), &1.30000000)
        );
    }

    #[test]
    fn test_book_updatebook() {
        let mut book = Book::new();
        let msg = r#"{"type":"snapshot","product_id":"BTC-USD","asks":[["93811.50","0.00003210"],["93811.51","0.00001070"],["100000.00","9991.07093828"],["101199.04","0.00100000"],["1000000.00","107239.00019802"],["2000000.00","1.00700000"],["5803600.00","1.00000000"],["5884800.00","1090.00000000"],["5885000.00","8.00000000"],["8000070.30","0.02570000"],["9999999.02","0.00100000"],["10000000.00","1.00000000"],["10000000.01","0.00400000"],["10000009.00","0.50000000"],["11099891.00","0.05000000"],["40000000.00","0.15013000"],["99999999.00","0.02000000"],["100000000.00","1.00000000"],["200000000.00","0.00000001"],["261642900.00","0.00445499"],["999999999.39","0.18599975"],["1000000000.00","4.00000000"],["1200000000.00","0.00000001"],["1222222222.00","10.84329557"],["2000000000.00","0.00000001"],["3300000000.00","0.00000001"],["5000000000.00","0.00000001"],["5400000000.00","0.00000001"],["5600000000.00","0.00000002"],["7200000000.00","0.00000001"],["7800000000.00","0.00000001"],["8000000000.00","0.00000001"],["8700000000.00","0.00000001"],["9100000000.00","0.00000001"],["9700000000.00","0.00000001"]],"bids":[["93711.51","1.30000000"],["93711.50","1.30000000"],["6240.00","0.00486124"],["3014.86","0.00033302"],["3014.85","0.01000000"],["300.00","35.00000000"],["222.00","2.10000000"],["200.90","0.05955202"],["200.00","200.99999500"],["139.50","9981.63682939"],["111.00","1.00000000"],["100.98","493.82400000"],["100.00","27.03234510"],["90.45","12.00000000"],["50.00","0.30000000"],["20.00","21.30000000"],["15.00","1.00000000"],["12.00","1.00000000"],["10.00","100430.00000000"],["8.00","2.00000000"],["6.64","12.16587225"],["6.00","1.00000000"],["5.00","926.00000000"],["4.43","1.00000000"],["3.00","1001.00000000"],["2.50","3.00000000"],["2.25","3.00000000"],["2.00","32.80634018"],["1.20","1.00000000"],["1.12","2.00000000"],["1.00","1354208.42448538"],["0.95","0.01000000"],["0.79","400.00000000"],["0.70","0.08000000"],["0.56","26.00000000"],["0.50","1.00000000"],["0.42","19500.00000000"],["0.20","0.00440000"],["0.19","1.00000000"],["0.16","12.00000000"],["0.15","0.01000000"],["0.12","0.54300000"],["0.10","1002.55510000"],["0.01","110092.45165027"]]}"#;
        let data = serde_json::from_str(&msg).expect("Invalid JSON msg");
        book.UpdateFullBook(data);
        let msg = r#"{"type":"l2update","product_id":"BTC-USD","changes":[["buy","100.00","27.04234510"]],"time":"2023-01-29T22:20:36.872152Z"}"#;
        let data2 = serde_json::from_str(&msg).expect("Invalid JSON msg");
        book.UpdateBook(data2);
        assert_eq!(book.bids[&OrderedFloat(100.00)], 27.04234510);
    }
}
