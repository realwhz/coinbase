# Coinbase feed client in Rust
Supports level2 data

## Usage
This is a command line utitlity.  The instrument is specified as an arugment, e.g., BTC-USD.  Once started, it will automatically connect to the Coinbase feed websocket (sandbox or production).  Once the connection is estalibshed, an interactive interface will be launched.  Currently the following commands are supported:
- bestbid
- bestask
- midprice
- spread
- fullbook
- quit

## Tests
cargo test
