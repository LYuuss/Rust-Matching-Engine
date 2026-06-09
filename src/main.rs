use std::env;
use std::fs;
use std::str::FromStr;

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
    println!();
    println!("Scenario commands:");
    println!("  submit <buy|sell> <quantity> <price>");
    println!("  cancel <order_id>");
    println!("  book [depth]");
    println!("  trades");
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

book 10
trades
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
        "cancel" => cancel_command(engine, &parts),
        "book" => {
            let depth = parts
                .get(1)
                .and_then(|value| value.parse::<usize>().ok())
                .unwrap_or(5);
            println!("{}", engine.book().format_depth(depth));
        }
        "trades" => print_trades(engine),
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
            eprintln!("invalid quantity '{}': expected a positive integer", parts[2]);
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
