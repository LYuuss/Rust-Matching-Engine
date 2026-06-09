use std::fmt;
use std::str::FromStr;

pub type OrderId = u64;
pub type Quantity = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Side {
    Buy,
    Sell,
}

impl FromStr for Side {
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input.to_lowercase().as_str() {
            "buy" | "bid" | "b" => Ok(Side::Buy),
            "sell" | "ask" | "s" => Ok(Side::Sell),
            other => Err(format!("unknown side '{other}', expected buy or sell")),
        }
    }
}

impl fmt::Display for Side {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Side::Buy => write!(f, "BUY"),
            Side::Sell => write!(f, "SELL"),
        }
    }
}

/// Price represented in cents/ticks to avoid floating-point comparison issues.
/// Example: 100.50 is stored as 10050.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Price(i64);

impl Price {
    pub fn from_cents(cents: i64) -> Result<Self, String> {
        if cents <= 0 {
            return Err("price must be positive".to_string());
        }
        Ok(Self(cents))
    }

    pub fn cents(self) -> i64 {
        self.0
    }

    pub fn parse(input: &str) -> Result<Self, String> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Err("price cannot be empty".to_string());
        }
        if trimmed.starts_with('-') {
            return Err("price must be positive".to_string());
        }

        let parts: Vec<&str> = trimmed.split('.').collect();
        if parts.len() > 2 {
            return Err(format!("invalid price '{input}'"));
        }

        let whole: i64 = parts[0]
            .parse()
            .map_err(|_| format!("invalid price '{input}'"))?;

        let fractional = if parts.len() == 2 {
            let fraction = parts[1];
            if fraction.len() > 2 {
                return Err("prices support at most two decimals".to_string());
            }
            let padded = format!("{fraction:0<2}");
            padded
                .parse::<i64>()
                .map_err(|_| format!("invalid price '{input}'"))?
        } else {
            0
        };

        Price::from_cents(whole * 100 + fractional)
    }
}

impl FromStr for Price {
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Price::parse(input)
    }
}

impl fmt::Display for Price {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{:02}", self.0 / 100, self.0.abs() % 100)
    }
}
