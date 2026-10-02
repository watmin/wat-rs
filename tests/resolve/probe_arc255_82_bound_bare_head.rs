//! 255.82 — a symbol call head with no `/` that names nothing is an
//! `UnresolvedReference` at check. A let-bound, param-bound, or match-bound
//! bare head still checks and runs. Quote data is left alone.

use wat::freeze::startup_from_file;
use wat::runtime::{apply_function, Value};

fn assert_unresolved(path: &str, needles: &[&str]) {
    let err = startup_from_file(path).expect_err(path);
    let msg = format!("{err}");
    for needle in needles {
        assert!(
            msg.contains(needle),
            "{path} should name {needle}, got:\n{msg}"
        );
    }
    // rune:lint(loose-assert) — the refusal is a multi-reference EDN value whose spans move; pin the resolve tag, not the whole value
    assert!(
        msg.contains("UnresolvedReference"),
        "{path} should be a resolve refusal, got:\n{msg}"
    );
}

#[test]
fn bound_bare_heads_check_and_run() {
    let world = startup_from_file("tests/resolve/probe_arc255_82_bound_bare_head.wat")
        .expect("let, param, and match binders are call heads");
    let func = world
        .symbols()
        .get(":user::compute")
        .expect("user/compute")
        .clone();
    let v = apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
        .expect("main runs");
    match v {
        Value::i64(n) => assert_eq!(n, 16),
        other => panic!("expected i64 16, got {other:?}"),
    }
    let shadow = world
        .symbols()
        .get(":user::shadow-some")
        .expect("user/shadow-some")
        .clone();
    let s = apply_function(shadow, vec![], world.symbols(), wat::rust_caller_span!())
        .expect("a local named Some runs");
    match s {
        Value::i64(n) => assert_eq!(n, 4),
        other => panic!("expected i64 4, got {other:?}"),
    }
}

#[test]
fn quasiquoted_bare_head_is_data() {
    let world = startup_from_file("tests/resolve/probe_arc255_82_quasiquote_bare_head.wat")
        .expect("quasiquoted (foozle 1) is data");
    let func = world
        .symbols()
        .get(":user::compute")
        .expect("user/compute")
        .clone();
    let v = apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
        .expect("main runs");
    assert!(matches!(v, Value::Nil), "expected nil, got {v:?}");
}

#[test]
fn unquote_escape_refuses_bare_head() {
    assert_unresolved(
        "tests/resolve/probe_arc255_82_unquote_bare_head.wat.bad",
        &["foozle"],
    );
}

#[test]
fn probe_three_heads_refused_by_name() {
    let err = startup_from_file(
        "wat-scripts/scratch-pad/arc-255/probe-255.82-a-head-that-names-nothing.wat.bad",
    )
    .expect_err("the three heads name nothing");
    let msg = format!("{err}");
    // rune:lint(loose-assert) — the count is one field of a multi-reference EDN value; the three path names are pinned below
    assert!(
        msg.contains("3 unresolved"),
        "expected exactly the three heads, got:\n{msg}"
    );
    for name in ["foozle", "my.made.up.thing", "wat.core.Option.zzznope"] {
        assert!(msg.contains(name), "missing {name} in:\n{msg}");
    }
}

#[test]
fn retired_bare_constructors_name_their_replacement() {
    // The checker's existing TypeMismatch for a bare Some/Ok/Err is not
    // UnknownCallee, so freeze reports it ahead of the resolve record.
    // The visible diagnostic is that TypeMismatch, and it names the replacement.
    let err = startup_from_file("tests/resolve/probe_arc255_82_retired_bare_ctor.wat.bad")
        .expect_err("bare Some/Ok/Err are refused");
    let msg = format!("{err}");
    for needle in [":wat::core::Some", ":wat::core::Ok", ":wat::core::Err"] {
        assert!(msg.contains(needle), "missing {needle} in:\n{msg}");
    }
}

#[test]
fn wrong_join_remedy_is_the_slash_spelling() {
    assert_unresolved(
        "tests/resolve/probe_arc255_82_wrong_join.wat.bad",
        &["wat.core.Option.expect", "wat.core.Option/expect"],
    );
}
