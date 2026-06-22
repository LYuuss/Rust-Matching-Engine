use rust_matching_engine::{MatchingEngine, Price, Side};

fn price(value: &str) -> Price {
    Price::parse(value).unwrap()
}

#[test]
fn non_crossing_orders_rest_in_the_book() {
    let mut engine = MatchingEngine::new();

    let sell = engine
        .submit_limit_order(Side::Sell, 10, price("101.00"))
        .unwrap();
    let buy = engine
        .submit_limit_order(Side::Buy, 5, price("100.00"))
        .unwrap();

    assert_eq!(sell.trades.len(), 0);
    assert_eq!(buy.trades.len(), 0);
    assert_eq!(engine.book().best_bid(), Some(price("100.00")));
    assert_eq!(engine.book().best_ask(), Some(price("101.00")));
}

#[test]
fn buy_order_crosses_with_best_ask() {
    let mut engine = MatchingEngine::new();

    let resting_sell = engine
        .submit_limit_order(Side::Sell, 10, price("100.00"))
        .unwrap();
    let incoming_buy = engine
        .submit_limit_order(Side::Buy, 4, price("101.00"))
        .unwrap();

    assert_eq!(resting_sell.order_id, 1);
    assert_eq!(incoming_buy.order_id, 2);
    assert_eq!(incoming_buy.trades.len(), 1);
    assert_eq!(incoming_buy.trades[0].maker_order_id, 1);
    assert_eq!(incoming_buy.trades[0].taker_order_id, 2);
    assert_eq!(incoming_buy.trades[0].quantity, 4);
    assert_eq!(incoming_buy.trades[0].price, price("100.00"));
    assert_eq!(incoming_buy.remaining, 0);
    assert_eq!(engine.book().best_ask(), Some(price("100.00")));
}

#[test]
fn partial_fill_leaves_remainder_on_resting_order() {
    let mut engine = MatchingEngine::new();

    engine
        .submit_limit_order(Side::Sell, 10, price("100.00"))
        .unwrap();
    engine
        .submit_limit_order(Side::Buy, 4, price("100.00"))
        .unwrap();

    assert_eq!(engine.book().total_volume(Side::Sell), 6);
    assert_eq!(engine.book().total_volume(Side::Buy), 0);
}

#[test]
fn incoming_remainder_rests_after_partial_execution() {
    let mut engine = MatchingEngine::new();

    engine
        .submit_limit_order(Side::Sell, 3, price("100.00"))
        .unwrap();
    let buy = engine
        .submit_limit_order(Side::Buy, 10, price("100.00"))
        .unwrap();

    assert_eq!(buy.trades.len(), 1);
    assert_eq!(buy.remaining, 7);
    assert_eq!(engine.book().best_bid(), Some(price("100.00")));
    assert_eq!(engine.book().total_volume(Side::Buy), 7);
}

#[test]
fn price_time_priority_is_respected() {
    let mut engine = MatchingEngine::new();

    let first_sell = engine
        .submit_limit_order(Side::Sell, 5, price("100.00"))
        .unwrap();
    let second_sell = engine
        .submit_limit_order(Side::Sell, 5, price("100.00"))
        .unwrap();
    let buy = engine
        .submit_limit_order(Side::Buy, 7, price("100.00"))
        .unwrap();

    assert_eq!(first_sell.order_id, 1);
    assert_eq!(second_sell.order_id, 2);
    assert_eq!(buy.trades.len(), 2);
    assert_eq!(buy.trades[0].maker_order_id, 1);
    assert_eq!(buy.trades[0].quantity, 5);
    assert_eq!(buy.trades[1].maker_order_id, 2);
    assert_eq!(buy.trades[1].quantity, 2);
    assert_eq!(engine.book().total_volume(Side::Sell), 3);
}

#[test]
fn cancel_order_removes_it_from_book() {
    let mut engine = MatchingEngine::new();

    let response = engine
        .submit_limit_order(Side::Buy, 5, price("99.00"))
        .unwrap();

    let cancelled = engine.cancel_order(response.order_id).unwrap();

    assert_eq!(cancelled.id, response.order_id);
    assert_eq!(engine.book().total_orders(), 0);
    assert_eq!(engine.book().best_bid(), None);
}

#[test]
fn price_parser_uses_cents_not_floats() {
    assert_eq!(price("100").to_string(), "100.00");
    assert_eq!(price("100.5").to_string(), "100.50");
    assert_eq!(price("100.05").to_string(), "100.05");
}

#[test]
fn stats_track_orders_trades_volume_and_spread() {
    let mut engine = MatchingEngine::new();

    engine
        .submit_limit_order(Side::Sell, 10, price("100.00"))
        .unwrap();
    engine
        .submit_limit_order(Side::Buy, 4, price("101.00"))
        .unwrap();
    engine
        .submit_limit_order(Side::Buy, 7, price("99.00"))
        .unwrap();

    let stats = engine.stats();

    assert_eq!(stats.submitted_orders, 3);
    assert_eq!(stats.resting_orders, 2);
    assert_eq!(stats.total_trades, 1);
    assert_eq!(stats.executed_volume, 4);
    assert_eq!(stats.resting_bid_volume, 7);
    assert_eq!(stats.resting_ask_volume, 6);
    assert_eq!(stats.best_bid, Some(price("99.00")));
    assert_eq!(stats.best_ask, Some(price("100.00")));
    assert_eq!(stats.spread_cents, Some(100));
    assert_eq!(stats.notional_traded_cents, 40_000);
    assert_eq!(stats.average_trade_price(), Some("100.00".to_string()));
}

#[test]
fn filled_maker_order_cannot_be_cancelled_after_matching() {
    let mut engine = MatchingEngine::new();

    let resting_sell = engine
        .submit_limit_order(Side::Sell, 5, price("100.00"))
        .unwrap();
    engine
        .submit_limit_order(Side::Buy, 5, price("100.00"))
        .unwrap();

    assert_eq!(engine.book().total_orders(), 0);
    assert!(engine.cancel_order(resting_sell.order_id).is_none());
}

#[test]
fn partially_filled_maker_order_stays_cancellable() {
    let mut engine = MatchingEngine::new();

    let resting_sell = engine
        .submit_limit_order(Side::Sell, 10, price("100.00"))
        .unwrap();
    engine
        .submit_limit_order(Side::Buy, 4, price("100.00"))
        .unwrap();

    let cancelled = engine.cancel_order(resting_sell.order_id).unwrap();

    assert_eq!(cancelled.id, resting_sell.order_id);
    assert_eq!(cancelled.remaining, 6);
    assert_eq!(engine.book().total_orders(), 0);
    assert_eq!(engine.book().best_ask(), None);
}

#[test]
fn cancel_order_uses_exact_side_and_price_level() {
    let mut engine = MatchingEngine::new();

    let buy_99 = engine
        .submit_limit_order(Side::Buy, 5, price("99.00"))
        .unwrap();
    let buy_98 = engine
        .submit_limit_order(Side::Buy, 7, price("98.00"))
        .unwrap();
    let sell_101 = engine
        .submit_limit_order(Side::Sell, 3, price("101.00"))
        .unwrap();

    let cancelled = engine.cancel_order(buy_98.order_id).unwrap();

    assert_eq!(cancelled.id, buy_98.order_id);
    assert_eq!(engine.book().best_bid(), Some(price("99.00")));
    assert_eq!(engine.book().best_ask(), Some(price("101.00")));
    assert_eq!(engine.book().total_volume(Side::Buy), 5);
    assert_eq!(engine.book().total_volume(Side::Sell), 3);

    assert!(engine.cancel_order(buy_99.order_id).is_some());
    assert!(engine.cancel_order(sell_101.order_id).is_some());
    assert_eq!(engine.book().total_orders(), 0);
}

#[test]
fn market_buy_consumes_best_asks_without_resting_remainder() {
    let mut engine = MatchingEngine::new();

    let first_sell = engine
        .submit_limit_order(Side::Sell, 5, price("100.00"))
        .unwrap();
    let second_sell = engine
        .submit_limit_order(Side::Sell, 5, price("101.00"))
        .unwrap();
    let market_buy = engine.submit_market_order(Side::Buy, 7).unwrap();

    assert_eq!(market_buy.order_id, 3);
    assert_eq!(market_buy.trades.len(), 2);
    assert_eq!(market_buy.trades[0].maker_order_id, first_sell.order_id);
    assert_eq!(market_buy.trades[0].quantity, 5);
    assert_eq!(market_buy.trades[0].price, price("100.00"));
    assert_eq!(market_buy.trades[1].maker_order_id, second_sell.order_id);
    assert_eq!(market_buy.trades[1].quantity, 2);
    assert_eq!(market_buy.trades[1].price, price("101.00"));
    assert_eq!(market_buy.remaining, 0);
    assert_eq!(engine.book().total_volume(Side::Sell), 3);
    assert_eq!(engine.book().total_volume(Side::Buy), 0);
}

#[test]
fn market_sell_consumes_best_bids_without_resting_remainder() {
    let mut engine = MatchingEngine::new();

    let first_buy = engine
        .submit_limit_order(Side::Buy, 5, price("101.00"))
        .unwrap();
    let second_buy = engine
        .submit_limit_order(Side::Buy, 5, price("100.00"))
        .unwrap();
    let market_sell = engine.submit_market_order(Side::Sell, 7).unwrap();

    assert_eq!(market_sell.order_id, 3);
    assert_eq!(market_sell.trades.len(), 2);
    assert_eq!(market_sell.trades[0].maker_order_id, first_buy.order_id);
    assert_eq!(market_sell.trades[0].quantity, 5);
    assert_eq!(market_sell.trades[0].price, price("101.00"));
    assert_eq!(market_sell.trades[1].maker_order_id, second_buy.order_id);
    assert_eq!(market_sell.trades[1].quantity, 2);
    assert_eq!(market_sell.trades[1].price, price("100.00"));
    assert_eq!(market_sell.remaining, 0);
    assert_eq!(engine.book().total_volume(Side::Buy), 3);
    assert_eq!(engine.book().total_volume(Side::Sell), 0);
}

#[test]
fn unfilled_market_order_does_not_rest_in_book() {
    let mut engine = MatchingEngine::new();

    engine
        .submit_limit_order(Side::Sell, 3, price("100.00"))
        .unwrap();
    let market_buy = engine.submit_market_order(Side::Buy, 10).unwrap();

    assert_eq!(market_buy.trades.len(), 1);
    assert_eq!(market_buy.remaining, 7);
    assert_eq!(engine.book().total_orders(), 0);
    assert_eq!(engine.book().best_bid(), None);
    assert_eq!(engine.book().best_ask(), None);
}

#[test]
fn market_order_rejects_zero_quantity() {
    let mut engine = MatchingEngine::new();

    let result = engine.submit_market_order(Side::Buy, 0);

    assert!(result.is_err());
    assert_eq!(engine.stats().submitted_orders, 0);
}

#[test]
fn decreasing_quantity_with_same_price_keeps_time_priority() {
    let mut engine = MatchingEngine::new();

    let first_buy = engine
        .submit_limit_order(Side::Buy, 10, price("100.00"))
        .unwrap();
    let second_buy = engine
        .submit_limit_order(Side::Buy, 5, price("100.00"))
        .unwrap();

    let modified = engine
        .modify_order(first_buy.order_id, 4, price("100.00"))
        .unwrap();

    assert!(!modified.reprioritized);
    assert_eq!(modified.trades.len(), 0);
    assert_eq!(modified.remaining, 4);
    assert_eq!(engine.book().total_volume(Side::Buy), 9);

    let market_sell = engine.submit_market_order(Side::Sell, 6).unwrap();

    assert_eq!(market_sell.trades.len(), 2);
    assert_eq!(market_sell.trades[0].maker_order_id, first_buy.order_id);
    assert_eq!(market_sell.trades[0].quantity, 4);
    assert_eq!(market_sell.trades[1].maker_order_id, second_buy.order_id);
    assert_eq!(market_sell.trades[1].quantity, 2);
}

#[test]
fn increasing_quantity_with_same_price_loses_time_priority() {
    let mut engine = MatchingEngine::new();

    let first_buy = engine
        .submit_limit_order(Side::Buy, 5, price("100.00"))
        .unwrap();
    let second_buy = engine
        .submit_limit_order(Side::Buy, 5, price("100.00"))
        .unwrap();

    let modified = engine
        .modify_order(first_buy.order_id, 8, price("100.00"))
        .unwrap();

    assert!(modified.reprioritized);
    assert_eq!(modified.trades.len(), 0);
    assert_eq!(modified.remaining, 8);

    let market_sell = engine.submit_market_order(Side::Sell, 5).unwrap();

    assert_eq!(market_sell.trades.len(), 1);
    assert_eq!(market_sell.trades[0].maker_order_id, second_buy.order_id);
    assert_eq!(market_sell.trades[0].quantity, 5);
    assert_eq!(engine.book().total_volume(Side::Buy), 8);
}

#[test]
fn price_change_reprioritizes_and_can_cross_the_book() {
    let mut engine = MatchingEngine::new();

    let resting_sell = engine
        .submit_limit_order(Side::Sell, 3, price("101.00"))
        .unwrap();
    let buy = engine
        .submit_limit_order(Side::Buy, 5, price("99.00"))
        .unwrap();

    let modified = engine
        .modify_order(buy.order_id, 5, price("101.00"))
        .unwrap();

    assert!(modified.reprioritized);
    assert_eq!(modified.order_id, buy.order_id);
    assert_eq!(modified.trades.len(), 1);
    assert_eq!(modified.trades[0].maker_order_id, resting_sell.order_id);
    assert_eq!(modified.trades[0].taker_order_id, buy.order_id);
    assert_eq!(modified.trades[0].quantity, 3);
    assert_eq!(modified.trades[0].price, price("101.00"));
    assert_eq!(modified.remaining, 2);
    assert_eq!(engine.book().best_bid(), Some(price("101.00")));
    assert_eq!(engine.book().best_ask(), None);
    assert_eq!(engine.book().total_volume(Side::Buy), 2);
}

#[test]
fn fully_filled_modified_order_is_removed_from_index() {
    let mut engine = MatchingEngine::new();

    engine
        .submit_limit_order(Side::Sell, 5, price("101.00"))
        .unwrap();
    let buy = engine
        .submit_limit_order(Side::Buy, 5, price("99.00"))
        .unwrap();

    let modified = engine
        .modify_order(buy.order_id, 5, price("101.00"))
        .unwrap();

    assert_eq!(modified.remaining, 0);
    assert_eq!(engine.book().total_orders(), 0);
    assert!(engine.cancel_order(buy.order_id).is_none());
}

#[test]
fn modify_rejects_unknown_or_zero_quantity_orders() {
    let mut engine = MatchingEngine::new();

    let missing = engine.modify_order(999, 5, price("100.00"));
    assert!(missing.is_err());

    let resting = engine
        .submit_limit_order(Side::Buy, 5, price("100.00"))
        .unwrap();
    let zero_quantity = engine.modify_order(resting.order_id, 0, price("100.00"));

    assert!(zero_quantity.is_err());
    assert_eq!(engine.book().total_volume(Side::Buy), 5);
}
