use std::collections::HashMap;

use crate::order::Order;
use crate::order_book::OrderBook;
use crate::stats::EngineStats;
use crate::trade::Trade;
use crate::types::{OrderId, Price, Quantity, Side};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderResponse {
    pub order_id: OrderId,
    pub trades: Vec<Trade>,
    pub remaining: Quantity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct OrderLocation {
    side: Side,
    price: Price,
}

#[derive(Debug, Default)]
pub struct MatchingEngine {
    book: OrderBook,
    next_order_id: OrderId,
    next_sequence: u64,
    trade_log: Vec<Trade>,
    order_locations: HashMap<OrderId, OrderLocation>,
}

impl MatchingEngine {
    pub fn new() -> Self {
        Self {
            book: OrderBook::new(),
            next_order_id: 1,
            next_sequence: 1,
            trade_log: Vec::new(),
            order_locations: HashMap::new(),
        }
    }

    pub fn submit_limit_order(
        &mut self,
        side: Side,
        quantity: Quantity,
        price: Price,
    ) -> Result<OrderResponse, String> {
        let order_id = self.next_order_id;
        self.next_order_id += 1;

        let sequence = self.next_sequence;
        self.next_sequence += 1;

        let mut incoming = Order::new(order_id, side, price, quantity, sequence)?;
        let trades = match side {
            Side::Buy => self.match_buy_order(&mut incoming),
            Side::Sell => self.match_sell_order(&mut incoming),
        };

        let remaining = incoming.remaining;
        if !incoming.is_filled() {
            self.order_locations
                .insert(incoming.id, OrderLocation { side, price });
            self.book.add_order(incoming);
        }

        self.trade_log.extend(trades.iter().cloned());

        Ok(OrderResponse {
            order_id,
            trades,
            remaining,
        })
    }

    pub fn cancel_order(&mut self, order_id: OrderId) -> Option<Order> {
        let location = self.order_locations.remove(&order_id)?;
        self.book.cancel_at(location.side, location.price, order_id)
    }

    pub fn book(&self) -> &OrderBook {
        &self.book
    }

    pub fn trades(&self) -> &[Trade] {
        &self.trade_log
    }

    pub fn stats(&self) -> EngineStats {
        let executed_volume = self
            .trade_log
            .iter()
            .map(|trade| trade.quantity)
            .sum::<Quantity>();

        let notional_traded_cents = self
            .trade_log
            .iter()
            .map(|trade| trade.quantity as u128 * trade.price.cents() as u128)
            .sum::<u128>();

        EngineStats {
            submitted_orders: self.next_order_id - 1,
            resting_orders: self.book.total_orders(),
            total_trades: self.trade_log.len(),
            executed_volume,
            resting_bid_volume: self.book.total_volume(Side::Buy),
            resting_ask_volume: self.book.total_volume(Side::Sell),
            best_bid: self.book.best_bid(),
            best_ask: self.book.best_ask(),
            spread_cents: self.book.spread(),
            notional_traded_cents,
        }
    }

    fn match_buy_order(&mut self, incoming: &mut Order) -> Vec<Trade> {
        let mut trades = Vec::new();

        while incoming.remaining > 0 {
            let Some(best_ask_price) = self.book.best_ask() else {
                break;
            };

            if best_ask_price > incoming.price {
                break;
            }

            let should_remove_level = {
                let ask_queue = self
                    .book
                    .asks
                    .get_mut(&best_ask_price)
                    .expect("best ask price must exist");

                while incoming.remaining > 0 && !ask_queue.is_empty() {
                    let maker = ask_queue.front_mut().expect("queue is not empty");
                    let quantity = incoming.remaining.min(maker.remaining);

                    incoming.remaining -= quantity;
                    maker.remaining -= quantity;

                    trades.push(Trade {
                        maker_order_id: maker.id,
                        taker_order_id: incoming.id,
                        price: maker.price,
                        quantity,
                    });

                    if maker.is_filled() {
                        let maker_order_id = maker.id;
                        ask_queue.pop_front();
                        self.order_locations.remove(&maker_order_id);
                    }
                }

                ask_queue.is_empty()
            };

            if should_remove_level {
                self.book.asks.remove(&best_ask_price);
            }
        }

        trades
    }

    fn match_sell_order(&mut self, incoming: &mut Order) -> Vec<Trade> {
        let mut trades = Vec::new();

        while incoming.remaining > 0 {
            let Some(best_bid_price) = self.book.best_bid() else {
                break;
            };

            if best_bid_price < incoming.price {
                break;
            }

            let should_remove_level = {
                let bid_queue = self
                    .book
                    .bids
                    .get_mut(&best_bid_price)
                    .expect("best bid price must exist");

                while incoming.remaining > 0 && !bid_queue.is_empty() {
                    let maker = bid_queue.front_mut().expect("queue is not empty");
                    let quantity = incoming.remaining.min(maker.remaining);

                    incoming.remaining -= quantity;
                    maker.remaining -= quantity;

                    trades.push(Trade {
                        maker_order_id: maker.id,
                        taker_order_id: incoming.id,
                        price: maker.price,
                        quantity,
                    });

                    if maker.is_filled() {
                        let maker_order_id = maker.id;
                        bid_queue.pop_front();
                        self.order_locations.remove(&maker_order_id);
                    }
                }

                bid_queue.is_empty()
            };

            if should_remove_level {
                self.book.bids.remove(&best_bid_price);
            }
        }

        trades
    }
}
