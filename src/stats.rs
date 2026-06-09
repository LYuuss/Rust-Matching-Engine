use crate::types::{Price, Quantity};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineStats {
    pub submitted_orders: u64,
    pub resting_orders: usize,
    pub total_trades: usize,
    pub executed_volume: Quantity,
    pub resting_bid_volume: Quantity,
    pub resting_ask_volume: Quantity,
    pub best_bid: Option<Price>,
    pub best_ask: Option<Price>,
    pub spread_cents: Option<i64>,
    pub notional_traded_cents: u128,
}

impl EngineStats {
    pub fn average_trade_price(&self) -> Option<String> {
        if self.executed_volume == 0 {
            return None;
        }

        Some(format_unsigned_cents(
            self.notional_traded_cents / self.executed_volume as u128,
        ))
    }

    pub fn format(&self) -> String {
        let avg_trade_price = self
            .average_trade_price()
            .unwrap_or_else(|| "-".to_string());

        let mut output = String::new();
        output.push_str("\n========== ENGINE STATS ==========");
        output.push('\n');
        output.push_str(&format!(
            "Submitted orders     : {}\n",
            self.submitted_orders
        ));
        output.push_str(&format!("Resting orders       : {}\n", self.resting_orders));
        output.push_str(&format!("Total trades         : {}\n", self.total_trades));
        output.push_str(&format!(
            "Executed volume      : {}\n",
            self.executed_volume
        ));
        output.push_str(&format!(
            "Resting bid volume   : {}\n",
            self.resting_bid_volume
        ));
        output.push_str(&format!(
            "Resting ask volume   : {}\n",
            self.resting_ask_volume
        ));
        output.push_str(&format!(
            "Best bid             : {}\n",
            format_price(self.best_bid)
        ));
        output.push_str(&format!(
            "Best ask             : {}\n",
            format_price(self.best_ask)
        ));
        output.push_str(&format!(
            "Spread               : {}\n",
            format_spread(self.spread_cents)
        ));
        output.push_str(&format!(
            "Notional traded      : {}\n",
            format_unsigned_cents(self.notional_traded_cents)
        ));
        output.push_str(&format!("Average trade price  : {}\n", avg_trade_price));
        output.push_str("==================================\n");
        output
    }
}

fn format_price(price: Option<Price>) -> String {
    price
        .map(|price| price.to_string())
        .unwrap_or_else(|| "-".to_string())
}

fn format_spread(spread_cents: Option<i64>) -> String {
    match spread_cents {
        Some(value) => format_signed_cents(value),
        None => "-".to_string(),
    }
}

fn format_signed_cents(cents: i64) -> String {
    let sign = if cents < 0 { "-" } else { "" };
    let absolute = cents.abs();
    format!("{}{}.{:02}", sign, absolute / 100, absolute % 100)
}

fn format_unsigned_cents(cents: u128) -> String {
    format!("{}.{:02}", cents / 100, cents % 100)
}
