//! Stone 255.89 amend 2 — each cured site, both spellings, the same value.
//! Fixture: probe_arc255_89_identity.wat beside this file.

use wat::freeze::call_beside_value;
use wat::runtime::Value;

fn call(entry: &str) -> Value {
    call_beside_value(file!(), entry).unwrap_or_else(|e| panic!("{entry}: {e:?}"))
}

#[test]
fn exists_derives_one_hit_in_each_spelling() {
    assert_eq!(call(":user::exists-kw"), Value::i64(1));
    assert_eq!(call(":user::exists-sym"), Value::i64(1));
}

#[test]
fn make_rate_derives_the_same_count_in_each_spelling() {
    assert_eq!(call(":user::rate-kw"), Value::i64(7));
    assert_eq!(call(":user::rate-sym"), Value::i64(7));
}

#[test]
fn pure_cond_is_true_in_each_spelling() {
    assert_eq!(call(":user::pure-cond-kw"), Value::bool(true));
    assert_eq!(call(":user::pure-cond-sym"), Value::bool(true));
}

#[test]
fn vocabulary_admits_rete_cond_and_refuses_i64_plus_in_each_spelling() {
    assert_eq!(call(":user::admit-kw"), Value::bool(true));
    assert_eq!(call(":user::admit-sym"), Value::bool(true));
    assert_eq!(call(":user::refuse-kw"), Value::bool(false));
    assert_eq!(call(":user::refuse-sym"), Value::bool(false));
}

#[test]
fn matches_pattern_head_agrees_in_each_spelling() {
    assert_eq!(call(":user::matches-kw"), Value::bool(true));
    assert_eq!(call(":user::matches-sym"), Value::bool(true));
}

#[test]
fn nested_option_some_binds_42_in_each_spelling() {
    assert_eq!(call(":user::nested-kw"), Value::i64(42));
    assert_eq!(call(":user::nested-sym"), Value::i64(42));
}

#[test]
fn total_agrees_across_spellings_and_length_stays_partial() {
    // `:wat::core::length` is `@Totality Unreviewed`. Both spellings stay false:
    // reading the symbol through the identity door does not promote it.
    assert_eq!(call(":user::total-length-kw"), Value::bool(false));
    assert_eq!(call(":user::total-length-sym"), Value::bool(false));
    assert_eq!(call(":user::total-lt-kw"), Value::bool(true));
    assert_eq!(call(":user::total-lt-sym"), Value::bool(true));
    assert_eq!(call(":user::total-plus-kw"), Value::bool(false));
    assert_eq!(call(":user::total-plus-sym"), Value::bool(false));
    assert_eq!(call(":user::total-subs-kw"), Value::bool(false));
    assert_eq!(call(":user::total-subs-sym"), Value::bool(false));
}
