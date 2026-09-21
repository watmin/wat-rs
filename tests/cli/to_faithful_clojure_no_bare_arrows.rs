//! 251.8d-i-b AMEND — converted output has no bare `<-` / `->` symbols.
//!
//! The replay fixture is the gate: `before.pre` still has the arrows (non-vacuity),
//! `after.post` must not. Comments may mention the spelling; the AST walk ignores them.

use wat_reader::ast::WatAST;
use wat_reader::parser::parse_all_with_file;

const BEFORE: &str =
    include_str!("../../wat-scripts/fixes/replay/to-faithful-clojure/before.pre");
const AFTER: &str =
    include_str!("../../wat-scripts/fixes/replay/to-faithful-clojure/after.post");

fn bare_arrow_count(forms: &[WatAST]) -> usize {
    fn walk(n: &WatAST, n_hits: &mut usize) {
        if n.is_bare_symbol("<-") || n.is_bare_symbol("->") {
            *n_hits += 1;
        }
        match n {
            WatAST::List(xs, _) | WatAST::Vector(xs, _) | WatAST::Set(xs, _) => {
                for c in xs {
                    walk(c, n_hits);
                }
            }
            WatAST::Map(pairs, _) => {
                for (k, v) in pairs {
                    walk(k, n_hits);
                    walk(v, n_hits);
                }
            }
            _ => {}
        }
    }
    let mut n = 0;
    for f in forms {
        walk(f, &mut n);
    }
    n
}

#[test]
fn to_faithful_clojure_converted_has_no_bare_arrow_symbols() {
    let before = parse_all_with_file(BEFORE, "before.pre")
        .unwrap_or_else(|e| panic!("before.pre must parse: {e:?}"));
    let after = parse_all_with_file(AFTER, "after.post")
        .unwrap_or_else(|e| panic!("after.post must parse: {e:?}"));
    let n_before = bare_arrow_count(&before);
    // NON-VACUITY: the pre-image still carries the arrows this gate forbids
    // in the post-image. A fixture that never had `<-` / `->` would make a
    // green "zero leftovers" assertion over nothing.
    assert!(
        n_before > 0,
        "before.pre has no bare <- / -> symbols — the leftover-arrow gate is vacant"
    );
    let n_after = bare_arrow_count(&after);
    assert_eq!(
        n_after, 0,
        "after.post still has {n_after} bare <- / -> symbol(s); every annotation/binding \
         arrow must become :-"
    );
}
