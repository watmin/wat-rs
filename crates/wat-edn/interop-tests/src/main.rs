//! Cross-tool handshake: writes a realistic EDN blob to stdout
//! using wat-edn. The Clojure side (wat-edn-clj/dashboard.clj)
//! reads it via clojure.edn/read and asserts the parsed shape.
//!
//! This proves bytes flow wat-edn → Clojure cleanly.

use chrono::{TimeZone, Utc};
use std::io::Write;
use uuid::Uuid;
use wat_edn::{write, Keyword, Symbol, Tag, Value};

fn build_trade_signal() -> Value<'static> {
    // Mirror of consumer-bridge.md's TradeSignal struct shape.
    // Build a tagged map that a Clojure dashboard would consume.
    let asset = Value::Keyword(Keyword::new("BTC"));
    let side = Value::Keyword(Keyword::new("Buy"));
    let size = Value::Float(0.025);
    let confidence = Value::Float(0.73);

    // reasoning: a Vec of HolonAST atoms (just symbols here for simplicity)
    let reasoning = Value::Tagged(
        Tag::ns("wat.core", "Vec<wat.holon.HolonAST>"),
        Box::new(Value::Vector(vec![
            Value::Tagged(
                Tag::ns("wat.holon", "Atom"),
                Box::new(Value::Keyword(Keyword::new("rsi-rising"))),
            ),
            Value::Tagged(
                Tag::ns("wat.holon", "Atom"),
                Box::new(Value::Keyword(Keyword::new("flow-positive"))),
            ),
        ])),
    );

    let proposed_at = Value::Inst(Utc.with_ymd_and_hms(2026, 4, 27, 14, 30, 0).unwrap());
    let id = Value::Uuid(Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap());

    let body = Value::Map(vec![
        (Value::Keyword(Keyword::new("asset")), asset),
        (Value::Keyword(Keyword::new("side")), side),
        (Value::Keyword(Keyword::new("size")), size),
        (Value::Keyword(Keyword::new("confidence")), confidence),
        (Value::Keyword(Keyword::new("reasoning")), reasoning),
        (Value::Keyword(Keyword::new("proposed-at")), proposed_at),
        (Value::Keyword(Keyword::new("id")), id),
    ]);

    Value::Tagged(
        Tag::ns("enterprise.observer.market", "TradeSignal"),
        Box::new(body),
    )
}

fn main() {
    let sig = build_trade_signal();
    let edn = write(&sig);
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    handle.write_all(edn.as_bytes()).unwrap();
    handle.write_all(b"\n").unwrap();
}

// Also include a test that proves the EDN we wrote round-trips
// through wat-edn itself. This locks the FORMAT before we even
// involve Clojure.
#[cfg(test)]
mod tests {
    use super::*;
    use wat_edn::parse;

    #[test]
    fn rust_self_round_trip() {
        let v1 = build_trade_signal();
        let s = write(&v1);
        let v2 = parse(&s).unwrap().into_owned();
        assert_eq!(v1, v2);
    }
}
