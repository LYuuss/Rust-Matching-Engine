# Rust Matching Engine

A simplified matching engine written in Rust.

This project is designed as a short, high-signal portfolio project for Rust / trading infrastructure roles.
It implements a small but realistic subset of an exchange matching engine:

- limit buy/sell orders
- price-time priority
- partial fills
- resting order book
- trade log
- order cancellation
- deterministic scenario runner
- engine statistics
- simple benchmark command
- indexed order lookup for faster cancellation
- integration tests

## Why this project matters

A matching engine is the core component of many trading systems. It receives incoming orders, compares them with the opposite side of the order book, executes trades when prices cross, and stores the remaining quantity when an order is not fully filled.

The goal here is not to build a production exchange. The goal is to show clean Rust, strong data-structure choices, and a solid understanding of trading infrastructure basics.

## Technical highlights

- `BTreeMap<Price, VecDeque<Order>>` for sorted price levels and FIFO time priority.
- Integer price representation in cents/ticks instead of floating-point prices.
- Deterministic order IDs and sequence numbers.
- Maker/taker trade logs.
- Partial fill handling on both incoming and resting orders.
- Scenario runner for reproducible examples.
- Lightweight benchmark using `std::time::Instant`.
- `HashMap<OrderId, OrderLocation>` index so cancellation can jump directly to the expected side and price level instead of scanning every price level.

## Project structure

```txt
rust_matching_engine/
├── Cargo.toml
├── README.md
├── examples/
│   └── basic.txt
├── src/
│   ├── engine.rs
│   ├── lib.rs
│   ├── main.rs
│   ├── order.rs
│   ├── order_book.rs
│   ├── stats.rs
│   ├── trade.rs
│   └── types.rs
└── tests/
    └── matching_tests.rs
```

## Core model

### Order

Each order has:

- an ID
- a side: `BUY` or `SELL`
- a price
- an initial quantity
- a remaining quantity
- a sequence number used for time priority

### Price representation

Prices are represented as integer cents/ticks, not floats.

For example:

```txt
100.50 -> 10050
99.01  -> 9901
```

This avoids floating-point comparison bugs, which matter a lot in trading systems.

### Matching rule

The engine uses price-time priority:

1. Best price wins.
2. If prices are equal, the oldest resting order wins.

For a `BUY` order:

```txt
match while best_ask <= buy_price
```

For a `SELL` order:

```txt
match while best_bid >= sell_price
```

The execution price is the maker/resting order price.

## Run

```bash
cargo run -- demo
```

Or run the example scenario:

```bash
cargo run -- scenario examples/basic.txt
```

You can also submit a single order to a fresh engine:

```bash
cargo run -- submit buy 10 100.50
```

## Statistics

Inside a scenario, use:

```txt
stats
```

It prints:

- submitted orders
- resting orders
- total trades
- executed volume
- resting bid/ask volume
- best bid / best ask
- spread
- notional traded
- average trade price

Example:

```txt
========== ENGINE STATS ==========
Submitted orders     : 4
Resting orders       : 1
Total trades         : 3
Executed volume      : 12
Resting bid volume   : 0
Resting ask volume   : 1
Best bid             : -
Best ask             : 102.00
Spread               : -
Notional traded      : 1204.00
Average trade price  : 100.33
==================================
```

## Cancellation index

The engine keeps an internal order-location index:

```txt
OrderId -> (Side, Price)
```

This avoids scanning the whole order book when cancelling an order. The engine can directly jump to the expected side and price level, then remove the order from the FIFO queue at that level.

The index is updated when:

- a resting order is added to the book
- a resting order is fully filled by an incoming order
- an order is cancelled

This is still simplified compared to a production exchange, but it is a more realistic design than a full linear scan across all price levels.

## Benchmark

Run a deterministic benchmark with 100,000 generated orders:

```bash
cargo run --release -- benchmark 100000
```

Or use the default:

```bash
cargo run --release -- benchmark
```

This prints elapsed time, approximate throughput, and final engine statistics.

## Scenario file syntax

```txt
submit sell 10 100.00
submit buy 4 101.00
book 10
trades
stats
```

Supported scenario commands:

```txt
submit <buy|sell> <quantity> <price>
cancel <order_id>
book [depth]
trades
stats
```

## Run tests

```bash
cargo test
```

The tests cover:

- non-crossing orders resting in the book
- buy order crossing best ask
- partial fills
- incoming remainder resting after a partial execution
- price-time priority
- cancellation
- cancellation after full and partial fills
- price parsing without floats
- engine statistics

## Example output

```txt
> submit sell 10 100.00
SUBMITTED id=1 side=SELL qty=10 price=100.00 remaining=10 trades=0

> submit buy 4 101.00
SUBMITTED id=2 side=BUY qty=4 price=101.00 remaining=0 trades=1
TRADE taker=2 maker=1 qty=4 price=100.00
```

## CV bullet

```txt
Built a simplified matching engine in Rust implementing limit orders, price-time priority, partial fills, indexed cancellation, trade logs, engine statistics, benchmarking and integration tests.
```

French version:

```txt
Développement d’un mini matching engine en Rust : ordres limit buy/sell, priorité prix/temps, exécutions partielles, annulation indexée d’ordres, statistiques moteur, benchmark simple et tests d’intégration.
```

## Next improvements

Good extensions for a stronger portfolio version:

1. Add market orders.
2. Add order status: accepted, partially filled, filled, cancelled, rejected.
3. Add benchmarks with `criterion`.
4. Add CSV scenario input/output.
5. Add a simple TCP or REST API.
6. Add latency measurements.
7. Add multi-symbol support, e.g. BTC-USD, ETH-USD.
8. Add persistent event replay.
