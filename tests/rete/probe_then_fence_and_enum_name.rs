//! STONE-the-fence-refuses-what-it-cannot-prove — negative controls for BOTH halves.
//!
//! C: an inline `match` in a `:then` is refused (even exhaustive). Un-arm C and
//!    `exhaustive_match_in_then_is_refused` compiles again (RED).
//! A: `:wat::rete::core::variant-name` / `:wat::core::variant-name` render a variant.
//!    Un-arm A and `enum_name_renders_the_variant` fails (RED).

use wat::assertion::AssertionPayload;
use wat::freeze::{startup_from_file, FrozenWorld, StartupError};
use wat::runtime::{apply_function, RuntimeError, RuntimeErrorKind, Value};

fn compile_message(world_path: &str, fn_name: &str) -> Result<Value, StartupError> {
    let world: FrozenWorld = startup_from_file(world_path)?;
    let func = world.symbols().get(fn_name).unwrap_or_else(|| panic!("no entry fn {fn_name:?}")).clone();
    let sym = world.symbols();
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        apply_function(func, vec![], sym, wat::rust_caller_span!())
    })) {
        Ok(res) => res.map_err(|e| StartupError::Runtime(Box::new(e))),
        Err(panic_payload) => {
            let (message, actual, expected) = match panic_payload.downcast_ref::<AssertionPayload>() {
                Some(p) => (p.message.clone(), p.actual.clone(), p.expected.clone()),
                None => {
                    let message = panic_payload
                        .downcast_ref::<String>()
                        .cloned()
                        .or_else(|| panic_payload.downcast_ref::<&str>().map(|s| (*s).to_string()))
                        .unwrap_or_else(|| "panic-opaque".to_string());
                    (message, None, None)
                }
            };
            Err(StartupError::Runtime(Box::new(RuntimeError::new(
                wat::rust_caller_span!(),
                RuntimeErrorKind::AssertionFailed { message, actual, expected },
            ))))
        }
    }
}

fn run_string(world_path: &str, fn_name: &str) -> String {
    let world: FrozenWorld = startup_from_file(world_path).expect("world must load");
    let func = world.symbols().get(fn_name).unwrap_or_else(|| panic!("no entry fn {fn_name:?}")).clone();
    let v = apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
        .unwrap_or_else(|e| panic!("run {fn_name}: {e}"));
    match v {
        Value::String(s) => (*s).clone(),
        other => panic!("expected String, got {other:?}"),
    }
}

#[test]
fn exhaustive_match_in_then_is_refused() {
    let r = compile_message(
        "tests/rete/probe_then_match_is_refused.wat",
        ":user::run-compile",
    );
    let StartupError::Runtime(e) = r.expect_err("an exhaustive match in a :then must be refused") else {
        panic!("expected Runtime assertion from the fence");
    };
    let RuntimeErrorKind::AssertionFailed { message, .. } = e.kind() else {
        panic!("expected AssertionFailed, got {:?}", e.kind());
    };
    // rune:lint(loose-assert) — the pin is the AXIS, not a frozen sentence that
    // cannot name the recommended op. Must say form-level vs head-level.
    assert!(
        message.contains("form-level") && message.contains("head-level"),
        "must name the axis (form-level exhaustiveness vs head-level totality):\n{message}"
    );
    // rune:lint(loose-assert) — names match, not a generic "not allowed".
    assert!(
        message.contains("match"),
        "must name match:\n{message}"
    );
}

#[test]
fn cond_with_else_in_then_still_works() {
    let s = run_string(
        "tests/rete/probe_then_cond_still_works.wat",
        ":user::run",
    );
    assert_eq!(s, "bb");
}

#[test]
fn cond_without_else_is_refused_at_expand() {
    let err = startup_from_file("tests/rete/probe_then_cond_no_else.wat.bad")
        .expect_err("cond without :else must fail at expand");
    let rendered = format!("{err}");
    // rune:lint(loose-assert) — expand diagnostic; path in Span.
    assert!(
        rendered.contains("non-exhaustive") && rendered.contains(":else"),
        "must be the expand-time cond diagnostic, not the fence:\n{rendered}"
    );
}

#[test]
fn enum_name_renders_the_variant() {
    let via_then = run_string("tests/rete/probe_enum_name.wat", ":user::via-then");
    let direct = run_string("tests/rete/probe_enum_name.wat", ":user::direct");
    assert_eq!(via_then, "Bb", "rete variant-name in a :then");
    assert_eq!(direct, "Bb", "core variant-name outside a rule");
}
