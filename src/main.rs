use std::env;
use std::fs;
use std::str::FromStr;
use std::time::Instant;

use rust_matching_engine::{MatchingEngine, Price, Side};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_help();
        return;
    }

    let mut engine = MatchingEngine::new();

    match args[1].as_str() {
        "demo" => run_demo(&mut engine),
        "benchmark" => {
            let order_count = args
                .get(2)
                .and_then(|value| value.parse::<usize>().ok())
                .unwrap_or(100_000);
            run_benchmark(order_count);
        }
        "scenario" => {
            if args.len() != 3 {
                eprintln!("Usage: cargo run -- scenario examples/basic.txt");
                std::process::exit(1);
            }
            let content = fs::read_to_string(&args[2]).unwrap_or_else(|error| {
                eprintln!("Could not read scenario file: {error}");
                std::process::exit(1);
            });
            run_scenario(&mut engine, &content);
        }
        "submit" => {
            if args.len() != 5 {
                eprintln!("Usage: cargo run -- submit <buy|sell> <quantity> <price>");
                std::process::exit(1);
            }
            let command = format!("submit {} {} {}", args[2], args[3], args[4]);
            execute_command(&mut engine, &command);
            println!("{}", engine.book().format_depth(5));
        }
        "submit-market" => {
            if args.len() != 4 {
                eprintln!("Usage: cargo run -- submit-market <buy|sell> <quantity>");
                std::process::exit(1);
            }
            let command = format!("submit-market {} {}", args[2], args[3]);
            execute_command(&mut engine, &command);
            println!("{}", engine.book().format_depth(5));
        }
        _ => print_help(),
    }
}

fn print_help() {
    println!("rust_matching_engine");
    println!();
    println!("Commands:");
    println!("  cargo run -- demo");
    println!("  cargo run -- scenario examples/basic.txt");
    println!("  cargo run -- submit <buy|sell> <quantity> <price>");
    println!("  cargo run -- submit-market <buy|sell> <quantity>");
    println!("  cargo run -- benchmark [order_count]");
    println!();
    println!("Scenario commands:");
    println!("  submit <buy|sell> <quantity> <price>");
    println!("  submit-market <buy|sell> <quantity>");
    println!("  cancel <order_id>");
    println!("  book [depth]");
    println!("  trades");
    println!("  stats");
}

fn run_demo(engine: &mut MatchingEngine) {
    let scenario = r#"
# Resting sell order
submit sell 10 100.00

# Incoming buy crosses the ask and partially fills it
submit buy 4 101.00

# Another sell does not cross with current bids, so it rests
submit sell 3 102.00

# Buy order consumes the rest of the first ask and part of the second
submit buy 8 102.00

# Market buy consumes remaining liquidity at the best available ask
submit-market buy 1

book 10
trades
stats
"#;

    run_scenario(engine, scenario);
}

fn run_scenario(engine: &mut MatchingEngine, scenario: &str) {
    for raw_line in scenario.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        println!("> {line}");
        execute_command(engine, line);
    }
}

fn execute_command(engine: &mut MatchingEngine, line: &str) {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.is_empty() {
        return;
    }

    match parts[0] {
        "submit" => submit_command(engine, &parts),
        "submit-market" => submit_market_command(engine, &parts),
        "cancel" => cancel_command(engine, &parts),
        "book" => {
            let depth = parts
                .get(1)
                .and_then(|value| value.parse::<usize>().ok())
                .unwrap_or(5);
            println!("{}", engine.book().format_depth(depth));
        }
        "trades" => print_trades(engine),
        "stats" => println!("{}", engine.stats().format()),
        other => eprintln!("Unknown command: {other}"),
    }
}

fn submit_command(engine: &mut MatchingEngine, parts: &[&str]) {
    if parts.len() != 4 {
        eprintln!("Usage: submit <buy|sell> <quantity> <price>");
        return;
    }

    let side = match Side::from_str(parts[1]) {
        Ok(side) => side,
        Err(error) => {
            eprintln!("{error}");
            return;
        }
    };

    let quantity = match parts[2].parse::<u64>() {
        Ok(quantity) => quantity,
        Err(_) => {
            eprintln!(
                "invalid quantity '{}': expected a positive integer",
                parts[2]
            );
            return;
        }
    };

    let price = match Price::parse(parts[3]) {
        Ok(price) => price,
        Err(error) => {
            eprintln!("{error}");
            return;
        }
    };

    match engine.submit_limit_order(side, quantity, price) {
        Ok(response) => {
            println!(
                "SUBMITTED id={} side={} qty={} price={} remaining={} trades={}",
                response.order_id,
                side,
                quantity,
                price,
                response.remaining,
                response.trades.len()
            );

            for trade in response.trades {
                println!(
                    "TRADE taker={} maker={} qty={} price={}",
                    trade.taker_order_id, trade.maker_order_id, trade.quantity, trade.price
                );
            }
        }
        Err(error) => eprintln!("{error}"),
    }
}

fn submit_market_command(engine: &mut MatchingEngine, parts: &[&str]) {
    if parts.len() != 3 {
        eprintln!("Usage: submit-market <buy|sell> <quantity>");
        return;
    }

    let side = match Side::from_str(parts[1]) {
        Ok(side) => side,
        Err(error) => {
            eprintln!("{error}");
            return;
        }
    };

    let quantity = match parts[2].parse::<u64>() {
        Ok(quantity) => quantity,
        Err(_) => {
            eprintln!(
                "invalid quantity '{}': expected a positive integer",
                parts[2]
            );
            return;
        }
    };

    match engine.submit_market_order(side, quantity) {
        Ok(response) => {
            println!(
                "MARKET_SUBMITTED id={} side={} qty={} remaining={} trades={}",
                response.order_id,
                side,
                quantity,
                response.remaining,
                response.trades.len()
            );

            for trade in response.trades {
                println!(
                    "TRADE taker={} maker={} qty={} price={}",
                    trade.taker_order_id, trade.maker_order_id, trade.quantity, trade.price
                );
            }
        }
        Err(error) => eprintln!("{error}"),
    }
}

fn cancel_command(engine: &mut MatchingEngine, parts: &[&str]) {
    if parts.len() != 2 {
        eprintln!("Usage: cancel <order_id>");
        return;
    }

    let order_id = match parts[1].parse::<u64>() {
        Ok(order_id) => order_id,
        Err(_) => {
            eprintln!("invalid order id '{}'", parts[1]);
            return;
        }
    };

    match engine.cancel_order(order_id) {
        Some(order) => println!(
            "CANCELLED id={} side={} remaining={} price={}",
            order.id, order.side, order.remaining, order.price
        ),
        None => println!("CANCEL_REJECTED id={order_id}"),
    }
}

fn print_trades(engine: &MatchingEngine) {
    if engine.trades().is_empty() {
        println!("No trades yet");
        return;
    }

    println!("\n========== TRADES ==========");
    for trade in engine.trades() {
        println!(
            "taker={} maker={} qty={} price={}",
            trade.taker_order_id, trade.maker_order_id, trade.quantity, trade.price
        );
    }
    println!("============================\n");
}

fn run_benchmark(order_count: usize) {
    if order_count == 0 {
        eprintln!("order_count must be greater than zero");
        std::process::exit(1);
    }

    let mut engine = MatchingEngine::new();
    let start = Instant::now();

    for index in 0..order_count {
        let side = if index % 2 == 0 {
            Side::Sell
        } else {
            Side::Buy
        };
        let quantity = (index % 10 + 1) as u64;
        let price = benchmark_price(index, side);

        engine
            .submit_limit_order(side, quantity, price)
            .expect("generated benchmark orders should be valid");
    }

    let elapsed = start.elapsed();
    let throughput = order_count as f64 / elapsed.as_secs_f64();

    println!("\n========== BENCHMARK ==========");
    println!("Generated orders : {order_count}");
    println!("Elapsed          : {:.3?}", elapsed);
    println!("Throughput       : {:.2} orders/sec", throughput);
    println!("===============================");
    println!("{}", engine.stats().format());
}

fn benchmark_price(index: usize, side: Side) -> Price {
    let offset = (index % 100) as i64;
    let cents = match side {
        Side::Sell => 10_000 + offset,
        Side::Buy => 10_050 + offset,
    };

    Price::from_cents(cents).expect("benchmark price should be positive")
}
