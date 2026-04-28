//! Simple timing harness for parse + write throughput.
//! Run with: `cargo run --release --example bench -p wat-edn`

use std::time::Instant;
use wat_edn::{parse, write};

const SMALL: &str = "[1 2 3 4 5]";

const REALISTIC: &str = r#"
#enterprise.observer.market/TradeSignal
{:asset       :BTC
 :side        :Buy
 :size        0.025
 :confidence  0.73
 :reasoning   #wat.core/Vec<wat.holon.HolonAST>
                [#wat.holon/Atom :rsi-rising
                 #wat.holon/Atom :flow-positive
                 #wat.holon/Bind [:flow-up :rate-up]]
 :proposed-at #inst "2026-04-26T14:30:00Z"
 :id          #uuid "550e8400-e29b-41d4-a716-446655440000"}
"#;

// Identifier-heavy: lots of short keywords (keyword-as-key map).
const IDENTIFIER_HEAVY: &str = r#"
{:user-id 42 :name "Alice" :email "a@b.c" :role :admin :active true
 :created-at 1000 :updated-at 2000 :tier :gold :balance 99.50
 :tags #{:vip :early-access :verified} :session-id :s12345
 :preferences {:theme :dark :lang :en :notifications true}
 :metrics {:visits 100 :purchases 5 :referrals 2}}
"#;

// String-heavy: lots of moderate-length strings (common JSON-ish).
const STRING_HEAVY: &str = r#"
{:title "The quick brown fox jumps over the lazy dog"
 :body "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua."
 :author "Alice Smith"
 :tags ["important" "review" "draft" "tech-debt" "performance"]
 :description "An EDN-encoded record with several text fields of varying length to exercise the string-handling path."}
"#;

// Large flat collection — vector of 50 maps.
fn build_large_flat() -> String {
    let mut s = String::from("[");
    for i in 0..50 {
        s.push_str(&format!("{{:id {} :name \"item{}\" :price {}.5}}", i, i, i));
        if i < 49 { s.push(' '); }
    }
    s.push(']');
    s
}

// Deeply nested — 30 levels of single-element vectors.
fn build_deeply_nested() -> String {
    let mut s = String::new();
    for _ in 0..30 { s.push('['); }
    s.push_str("42");
    for _ in 0..30 { s.push(']'); }
    s
}

fn time_parse(name: &str, input: &str, iters: usize) {
    // Warm up
    for _ in 0..(iters / 10).max(1) {
        let _ = parse(input).unwrap();
    }

    let start = Instant::now();
    for _ in 0..iters {
        let _ = parse(input).unwrap();
    }
    let elapsed = start.elapsed();
    let bytes = input.len() as f64 * iters as f64;
    let mb = bytes / (1024.0 * 1024.0);
    let secs = elapsed.as_secs_f64();
    println!(
        "{:30} {:>8} bytes  x{:<8} = {:.2} MB/s   ({:>8.2}µs/op)",
        name,
        input.len(),
        iters,
        mb / secs,
        elapsed.as_micros() as f64 / iters as f64,
    );
}

fn time_round_trip(name: &str, input: &str, iters: usize) {
    let v = parse(input).unwrap();
    for _ in 0..(iters / 10).max(1) {
        let _ = write(&v);
    }
    let start = Instant::now();
    for _ in 0..iters {
        let _ = write(&v);
    }
    let elapsed = start.elapsed();
    let secs = elapsed.as_secs_f64();
    let bytes = input.len() as f64 * iters as f64;
    let mb = bytes / (1024.0 * 1024.0);
    println!(
        "{:30} {:>8} bytes  x{:<8} = {:.2} MB/s   ({:>8.2}µs/op)",
        name,
        input.len(),
        iters,
        mb / secs,
        elapsed.as_micros() as f64 / iters as f64,
    );
}

fn main() {
    let large_flat = build_large_flat();
    let deeply_nested = build_deeply_nested();

    println!("wat-edn benchmark — parse throughput");
    println!("{}", "─".repeat(80));
    time_parse("parse small  [1 2 3 4 5]",       SMALL, 1_000_000);
    time_parse("parse realistic blob",            REALISTIC, 100_000);
    time_parse("parse identifier-heavy",          IDENTIFIER_HEAVY, 100_000);
    time_parse("parse string-heavy",              STRING_HEAVY, 100_000);
    time_parse("parse large flat (50-map vec)",   &large_flat, 50_000);
    time_parse("parse deeply nested (30 levels)", &deeply_nested, 200_000);

    println!();
    println!("wat-edn benchmark — write throughput");
    println!("{}", "─".repeat(80));
    time_round_trip("write small  [1 2 3 4 5]",       SMALL, 1_000_000);
    time_round_trip("write realistic blob",            REALISTIC, 100_000);
    time_round_trip("write identifier-heavy",          IDENTIFIER_HEAVY, 100_000);
    time_round_trip("write string-heavy",              STRING_HEAVY, 100_000);
    time_round_trip("write large flat (50-map vec)",   &large_flat, 50_000);
    time_round_trip("write deeply nested (30 levels)", &deeply_nested, 200_000);
}
