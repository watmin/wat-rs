//! Arc 271 — MULTI-type-param generic protocol methods (`method<S,R>` in defprotocol).
//!
//! arc-232 / probe_arc232_generic_method shipped SINGLE-param generic methods (`make<T>`). A method
//! with TWO type params — `combine<A,B>` — does not parse: the method name is a bare Symbol, and
//! `lex_symbol` (lexer.rs:811) is angle-blind (a naive scan-until-`is_symbol_break`), so the comma in
//! `combine<A,B>` (EDN whitespace, is_symbol_break:467) splits the token into `combine<A` + `B>`.
//! The KEYWORD lexer (lex_keyword, lexer.rs:637) IS angle-depth-aware (keeps commas inside `<…>`),
//! which is why generic FNS (`:foldl<T,Acc>`) work but generic METHODS (bare-symbol names) don't —
//! one design, applied in one place only. The fix teaches `lex_symbol` the same angle-awareness.
//!
//! THE CALLER THAT SURFACED THIS: the arc-209 host seam. The host-agnostic `Host/spawn` method must
//! be generic over BOTH S and R (`Listener'<S,R>` arg + the `Peer'<R,S>` closure it builds) — see
//! DESIGN-STONE-host-parity-4a-start.md. The call-site instantiation (check.rs:5541) already loops
//! over ALL type_params (N-safe); only the lex/parse path blocked multi-param.
//!
//! RED at HEAD: `combine<A,B>` fails to parse — `"combine<A" opens '<' but does not close '>'`.
//! GREEN once `lex_symbol` keeps commas inside `<…>` so the method name lexes as one token, the
//! splitter (`split_name_and_type_params`, runtime.rs:2998 — already multi-param) yields `["A","B"]`,
//! and the call site instantiates both.
//!
//! Run: cargo test --release -p wat --test probe_arc271_multi_param_generic_method

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

#[test]
fn multi_param_generic_method_parses_and_instantiates() {
    let world = startup_beside(file!())
        .expect("startup should succeed (combine<A,B>: both type params parse + instantiate at the call)");
    let ast = wat::parse_one!("(:user::go)").expect("parse");
    let got = eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .unwrap_or_else(|e| panic!("go raised: {e:?}"));
    assert!(
        matches!(got, Value::i64(5)),
        "expected 5: combine<A,B> with (i64,String) → A=i64,B=String → returns x=5; got {got:?}"
    );
}
