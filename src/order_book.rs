use std::collections::{BTreeMap, VecDeque};

use crate::order::Order;
use crate::types::{OrderId, Price, Quantity, Side};

#[derive(Debug, Default)]
pub struct OrderBook {
    pub(crate) bids: BTreeMap<Price, VecDeque<Order>>,
    pub(crate) asks: BTreeMap<Price, VecDeque<Order>>,
}

impl OrderBook {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_order(&mut self, order: Order) {
        let book_side = match order.side {
            Side::Buy => &mut self.bids,
            Side::Sell => &mut self.asks,
        };

        book_side.entry(order.price).or_default().push_back(order);
    }

    pub fn best_bid(&self) -> Option<Price> {
        self.bids.keys().next_back().copied()
    }

    pub fn best_ask(&self) -> Option<Price> {
        self.asks.keys().next().copied()
    }

    pub fn spread(&self) -> Option<i64> {
        Some(self.best_ask()?.cents() - self.best_bid()?.cents())
    }

    pub fn cancel(&mut self, order_id: OrderId) -> Option<Order> {
        Self::cancel_from_side(&mut self.bids, order_id)
            .or_else(|| Self::cancel_from_side(&mut self.asks, order_id))
    }

    pub fn total_orders(&self) -> usize {
        self.bids.values().map(VecDeque::len).sum::<usize>()
            + self.asks.values().map(VecDeque::len).sum::<usize>()
    }

    pub fn total_volume(&self, side: Side) -> Quantity {
        let levels = match side {
            Side::Buy => &self.bids,
            Side::Sell => &self.asks,
        };

        levels
            .values()
            .flat_map(|orders| orders.iter())
            .map(|order| order.remaining)
            .sum()
    }

    pub fn format_depth(&self, depth: usize) -> String {
        let mut output = String::new();
        output.push_str("\n========== ORDER BOOK ==========\n");
        output.push_str("ASKS\n");
        output.push_str("Price      Quantity   Orders\n");
        output.push_str("-------------------------------\n");

        for (price, orders) in self.asks.iter().take(depth) {
            let quantity: Quantity = orders.iter().map(|order| order.remaining).sum();
            output.push_str(&format!("{:<10} {:<10} {}\n", price, quantity, orders.len()));
        }

        output.push_str("-------------------------------\n");
        output.push_str("BIDS\n");
        output.push_str("Price      Quantity   Orders\n");
        output.push_str("-------------------------------\n");

        for (price, orders) in self.bids.iter().rev().take(depth) {
            let quantity: Quantity = orders.iter().map(|order| order.remaining).sum();
            output.push_str(&format!("{:<10} {:<10} {}\n", price, quantity, orders.len()));
        }

        output.push_str("===============================\n");
        output
    }

    fn cancel_from_side(
        levels: &mut BTreeMap<Price, VecDeque<Order>>,
        order_id: OrderId,
    ) -> Option<Order> {
        let mut empty_price = None;
        let mut cancelled = None;

        for (price, queue) in levels.iter_mut() {
            if let Some(index) = queue.iter().position(|order| order.id == order_id) {
                cancelled = queue.remove(index);
                if queue.is_empty() {
                    empty_price = Some(*price);
                }
                break;
            }
        }

        if let Some(price) = empty_price {
            levels.remove(&price);
        }

        cancelled
    }
}
