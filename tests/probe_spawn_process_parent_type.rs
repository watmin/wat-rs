//! Arc 170 slice 3 Gap F-3 — regression probes for parent type registry
//! inheritance to spawn-process child.
//!
//! Three probes confirm that types declared at parent top-level (and NOT
//! referenced in the spawn-process fn signature or body AST) are visible
//! in the hermetic child subprocess's TypeEnv, so that `edn::read` can
//! deserialize tagged EDN forms whose type is only known at the parent level.
//!
//! Gap F-3 fix: `extract_closure` propagates the parent's full user type
//! registry to the child by including all non-reserved user types as AST
//! forms in the closure prologue (in addition to the types already captured
//! via reference-walking the fn signature + body).
//!
//! All three probes FAIL before Gap F-3 ships (child exits non-zero because
//! `edn::read` encounters an `UnknownTag` error for the type declared only
//! in the parent). All three probes PASS after the fix.
//!
//! ## Why string literals trigger the gap
//!
//! The closure-extraction reference walker (`walk_free_symbols`) classifies
//! `WatAST::StringLit(..)` as a leaf — no type discovery. A fn body that
//! calls `(:wat::edn::read s)` where `s` is a string literal containing a
//! tagged EDN form naming a parent-declared type does NOT cause that type to
//! be captured into the closure's prologue type set. The child's TypeEnv
//! lacks the type. `edn::read` returns `EdnReadError::UnknownTag` →
//! `RuntimeError::MalformedForm` → child exits non-zero.
//!
//! Probe 1: parent-declared struct used only via `edn::read` in the child.
//! Probe 2: parent-declared enum used only via `edn::read` in the child.
//! Probe 3: parent-declared parametric struct used only via `edn::read`.
//!
//! ## Note on raw-string delimiters
//!
//! WAT string literals in these probes begin with `"#...` (tagged EDN). The
//! `"#` sequence would terminate a Rust `r#"..."#` raw string. All probes
//! therefore use `r##"..."##` as the Rust raw string delimiter.

use std::sync::Arc;
use wat::ast::WatAST;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;
use wat::runtime::{eval, Environment, ProgramHandleInner};

// ─── helpers ────────────────────────────────────────────────────────────────

fn freeze_ok(src: &str) -> wat::freeze::FrozenWorld {
    match startup_from_source(src, None, Arc::new(InMemoryLoader::new())) {
        Ok(w) => w,
        Err(e) => panic!("freeze should succeed; got: {}", e),
    }
}

/// Drain the stderr field (index 2) of a Process Struct value.
fn drain_stderr(process: &wat::runtime::Value) -> String {
    match process {
        wat::runtime::Value::Struct(s) if s.type_name == ":wat::kernel::Process" => {
            match &s.fields[2] {
                wat::runtime::Value::io__IOReader(rdr) => {
                    let mut all = String::new();
                    while let Ok(Some(line)) = rdr.read_line(wat::span::Span::unknown()) {
                        all.push_str(&line);
                    }
                    all
                }
                _ => "<stderr field not IOReader>".into(),
            }
        }
        _ => "<not a Process Struct>".into(),
    }
}

/// Evaluate `(:my::launch)` in the frozen world, fork the child, wait for
/// it to exit, and return (exit_code, stderr_text).
fn run_launch(world: &wat::freeze::FrozenWorld) -> (i64, String) {
    let call = WatAST::List(
        vec![WatAST::Keyword(
            ":my::launch".into(),
            wat::span::Span::unknown(),
        )],
        wat::span::Span::unknown(),
    );
    let env = Environment::new();
    let process = eval(&call, &env, world.symbols()).expect("launch should evaluate");
    let handle = match &process {
        wat::runtime::Value::Struct(s) if s.type_name == ":wat::kernel::Process" => {
            match &s.fields[3] {
                wat::runtime::Value::wat__kernel__ProgramHandle(h) => h.clone(),
                other => panic!("expected ProgramHandle field at index 3; got {:?}", other),
            }
        }
        other => panic!("expected Process Struct from launch; got {:?}", other),
    };
    // Wait for child exit BEFORE draining stderr so the child has finished
    // writing. Child's pipes are buffered; join-first is safe for small
    // assertion bodies (same pattern as run-hermetic-driver in test.wat).
    let exit_code: i64 = match handle.as_ref() {
        ProgramHandleInner::Forked(child) => child.wait_or_cached(),
        other => panic!("expected Forked handle; got {:?}", other),
    };
    let stderr = drain_stderr(&process);
    (exit_code, stderr)
}

// ─── Probe 1 — parent-declared struct visible in child via edn::read ────────

/// The child fn body calls `(:wat::edn::read s)` where `s` is a string
/// literal containing a tagged EDN form `#test.proto/Point {:x 3 :y 4}`.
/// The type `:test::proto::Point` is declared at parent top-level but NOT
/// referenced by any keyword in the fn body AST (no `/new` constructor,
/// no type annotation). Before Gap F-3, the child's TypeEnv lacks
/// `:test::proto::Point` and `edn::read` returns `UnknownTag` →
/// `RuntimeError::MalformedForm` → child exits non-zero. After Gap F-3,
/// the type is in the prologue and the child exits 0.
#[test]
fn probe_spawn_process_inherits_parent_struct() {
    let src = r##"
        (:wat::core::struct :test::proto::Point
          (x :wat::core::i64)
          (y :wat::core::i64))

        (:wat::core::define
          (:my::launch -> :wat::kernel::Process<wat::core::nil,wat::core::nil>)
          (:wat::kernel::spawn-process
            (:wat::core::fn
              [_rx <- :wat::kernel::Receiver<wat::core::nil>
               _tx <- :wat::kernel::Sender<wat::core::nil>]
              -> :wat::core::nil
              (:wat::core::let
                [s "#test.proto/Point {:x 3 :y 4}"
                 _ (:wat::edn::read s)]
                :wat::core::nil))))

        (:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)
    "##;
    let world = freeze_ok(src);
    let (exit_code, stderr) = run_launch(&world);
    assert_eq!(
        exit_code, 0i64,
        "child should exit 0 (struct type in child TypeEnv); stderr:\n{}",
        stderr
    );
}

// ─── Probe 2 — parent-declared enum visible in child via edn::read ──────────

/// Same shape: parent declares `:test::proto::Color` (unit-variant enum).
/// The fn body calls `edn::read` on a string with a unit-variant EDN form
/// `#test.proto.Color/Red nil`. The type `:test::proto::Color` is NOT
/// referenced in the fn body AST.
///
/// EDN format for enum unit variants (from `value_to_edn_with`'s Enum arm):
///   type_path = `:test::proto::Color`, variant = `Red`
///   → tag_name = `:test::proto::Color::Red`
///   → rfind `::` → ns = `test.proto.Color`, name = `Red`
///   → `#test.proto.Color/Red nil`
/// On read: `reconstruct_enum_unit("test.proto.Color", "Red", types)`
///   → `ns_to_enum_path("test.proto.Color")` = `:test::proto::Color`
///   → `types.get(":test::proto::Color")` → must be present in child TypeEnv.
#[test]
fn probe_spawn_process_inherits_parent_enum() {
    let src = r##"
        (:wat::core::enum :test::proto::Color
          :Red
          :Green
          :Blue)

        (:wat::core::define
          (:my::launch -> :wat::kernel::Process<wat::core::nil,wat::core::nil>)
          (:wat::kernel::spawn-process
            (:wat::core::fn
              [_rx <- :wat::kernel::Receiver<wat::core::nil>
               _tx <- :wat::kernel::Sender<wat::core::nil>]
              -> :wat::core::nil
              (:wat::core::let
                [s "#test.proto.Color/Red nil"
                 _ (:wat::edn::read s)]
                :wat::core::nil))))

        (:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)
    "##;
    let world = freeze_ok(src);
    let (exit_code, stderr) = run_launch(&world);
    assert_eq!(
        exit_code, 0i64,
        "child should exit 0 (enum type in child TypeEnv); stderr:\n{}",
        stderr
    );
}

// ─── Probe 3 — parent-declared parametric struct visible in child ────────────

/// Same shape with a parametric (generic) struct `:test::proto::Wrapper<E>`.
///
/// Parametric types are stored in TypeEnv by base name (without `<E>` suffix)
/// per `parse_declared_name`'s `stored_name = format!(":{}", base)` path.
/// The edn::read reconstruct path calls `ns_to_wat_path("test.proto", "Wrapper")`
/// = `:test::proto::Wrapper` — same key — so the lookup succeeds once the type
/// is in the child's TypeEnv.
///
/// The fn body calls `edn::read` on `#test.proto/Wrapper {:label :empty :value 42}`.
/// `:test::proto::Wrapper` is NOT referenced in the fn body AST.
#[test]
fn probe_spawn_process_inherits_parametric_type() {
    // Note: WAT's struct parser requires field types to be keywords. The
    // type parameter `E` is a bare symbol and causes a parse error.
    // Instead we declare the struct with a concrete type for the `value`
    // field (:wat::core::i64) — this is still a parametric struct in the
    // sense that it's declared with a type-param name in the registry key
    // (`:test::proto::Wrapper` without `<E>`) but the field type is
    // resolved at declaration time. The probe verifies that the TYPE
    // ITSELF (the struct name / registry entry) is inherited — not that
    // the parametric instantiation mechanism works.
    //
    // Correct parametric syntax: WAT structs with type params store the
    // type parameter in StructDef.type_params but field types must be
    // keywords (type-exprs). Bare `E` is not a keyword and fails parse.
    // The Gap F-3 concern (registry inheritance) is orthogonal to
    // parametric field-type resolution; this probe covers the registry
    // key `:test::proto::Wrapper` (base name stored in TypeEnv per
    // `parse_declared_name`'s `stored_name = format!(":{}", base)` path).
    let src = r##"
        (:wat::core::struct :test::proto::Wrapper<E>
          (label :wat::core::String)
          (value :wat::core::i64))

        (:wat::core::define
          (:my::launch -> :wat::kernel::Process<wat::core::nil,wat::core::nil>)
          (:wat::kernel::spawn-process
            (:wat::core::fn
              [_rx <- :wat::kernel::Receiver<wat::core::nil>
               _tx <- :wat::kernel::Sender<wat::core::nil>]
              -> :wat::core::nil
              (:wat::core::let
                [s "#test.proto/Wrapper {:label :empty :value 42}"
                 _ (:wat::edn::read s)]
                :wat::core::nil))))

        (:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)
    "##;
    let world = freeze_ok(src);
    let (exit_code, stderr) = run_launch(&world);
    assert_eq!(
        exit_code, 0i64,
        "child should exit 0 (parametric type in child TypeEnv); stderr:\n{}",
        stderr
    );
}
