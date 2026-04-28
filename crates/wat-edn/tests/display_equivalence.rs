//! Locks the agreement between the writer's direct push_str path
//! (`write_symbol` / `write_keyword` / `write_tag`) and the
//! `Display` impls on `Symbol` / `Keyword` / `Tag`. The two paths
//! must produce byte-identical output — surfaced by the /sever
//! ward as a complection (rule defined in two places). The
//! agreement is the cheap fix; consolidation to one source would
//! cost the writer's perf advantage.

use wat_edn::{write, Keyword, Symbol, Tag, Value};

#[test]
fn symbol_writer_matches_display() {
    let cases = [
        Symbol::new("foo"),
        Symbol::new("foo-bar?"),
        Symbol::ns("ns", "name"),
        Symbol::ns("my.app.events", "OrderPlaced"),
    ];
    for s in cases {
        let via_writer = write(&Value::Symbol(s.clone()));
        let via_display = format!("{}", s);
        assert_eq!(via_writer, via_display, "symbol byte-equivalence");
    }
}

#[test]
fn bare_slash_symbol_matches_display() {
    let s = Symbol::try_new("/").unwrap();
    let via_writer = write(&Value::Symbol(s.clone()));
    let via_display = format!("{}", s);
    assert_eq!(via_writer, via_display);
    assert_eq!(via_writer, "/");
}

#[test]
fn keyword_writer_matches_display() {
    let cases = [
        Keyword::new("foo"),
        Keyword::new("valid?"),
        Keyword::ns("ns", "name"),
        Keyword::ns("my.app.events", "OrderPlaced"),
    ];
    for k in cases {
        let via_writer = write(&Value::Keyword(k.clone()));
        let via_display = format!("{}", k);
        assert_eq!(via_writer, via_display, "keyword byte-equivalence");
    }
}

#[test]
fn tag_writer_matches_display() {
    let cases = [
        Tag::ns("myapp", "Person"),
        Tag::ns("my.app.events", "OrderPlaced"),
        Tag::ns("wat.core", "Vec<i64>"),
        Tag::ns("enterprise.observer.market", "TradeSignal"),
    ];
    for t in cases {
        let via_writer = write(&Value::Tagged(t.clone(), Box::new(Value::Nil)));
        let via_display = format!("{} nil", t);
        assert_eq!(via_writer, via_display, "tag byte-equivalence");
    }
}
