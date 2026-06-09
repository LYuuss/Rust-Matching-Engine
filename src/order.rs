use crate::types::{OrderId, Price, Quantity, Side};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Order {
    pub id: OrderId,
    pub side: Side,
    pub price: Price,
    pub quantity: Quantity,
    pub remaining: Quantity,
    pub sequence: u64,
}

impl Order {
    pub fn new(
        id: OrderId,
        side: Side,
        price: Price,
        quantity: Quantity,
        sequence: u64,
    ) -> Result<Self, String> {
        if quantity == 0 {
            return Err("quantity must be greater than zero".to_string());
        }

        Ok(Self {
            id,
            side,
            price,
            quantity,
            remaining: quantity,
            sequence,
        })
    }

    pub fn is_filled(&self) -> bool {
        self.remaining == 0
    }
}
