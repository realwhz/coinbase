#!/usr/bin/env python3

from websocket import create_connection, WebSocketConnectionClosedException

import json

ws = create_connection("wss://ws-feed-public.sandbox.exchange.coinbase.com")
ws.send(
    json.dumps(
        {
            "type": "subscribe",
            "product_ids": ["BTC-USD"],
            "channels": ["level2"],
        }
    )
)

try:
    while True:
        data = json.loads(ws.recv())
        print(data)
except KeyboardInterrupt:
    pass
