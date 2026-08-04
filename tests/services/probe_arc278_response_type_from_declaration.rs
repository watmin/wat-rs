//! Arc 278 #74 — `<Op>Response` is LAW (builder ruling, 2026-08-05: "convention is law —
//! enforce it… services are our OOP layer, we make requests to them and get responses back.").
//!
//! ★ INVERTED. This file used to be the acceptance case for "the response type's name is READ,
//! not guessed": `:probe::Odd::Verdict` (deliberately NOT `PutResponse`) exercised three call
//! paths — `op-methods`' own generated client method, `serve-op-arms`' size guard, and its shape
//! guard — each proving its `RequestTooLarge`/`RequestMalformed` ctor was built by reading the
//! DECLARED response type, never by guessing `<OpPascal>Response`.
//!
//! Under the ruling that proposition is FALSE: an op's response type IS `<Op>Response`, checker-
//! enforced at `defsurface` registration (`synthesize_surface_protocol`, `src/types.rs`). So
//! `:probe::Odd::put -> Verdict` is no longer a legal declaration at all — the file cannot even
//! FREEZE, let alone reach any of the three call sites the old tests drove. There is nothing left
//! to read a name from, because there is no longer a way to declare a wrong one and have it run.
//!
//! This file's subject is now the wall itself: `:probe::Odd` stays non-conforming (`Verdict`, not
//! `PutResponse` — the .wat is UNCHANGED, on purpose) and the ONE test below asserts the surface
//! is REFUSED, located, naming BOTH the declared name and the required one. The three original
//! probes collapse into this one because the refusal fires at registration, before op-methods,
//! serve-op-arms, or any generated code exists to reach — there is no code left to distinguish.
//!
//! The fixture is `.wat.bad` (not `.wat`), the established convention for "this file must NOT
//! load" (see `tests/services/probe_arc170_c2_d_bodiless_edge.wat.bad`) — it can no longer be a
//! plain `.wat` sibling loaded via `startup_beside`, which asserts success by construction.

use wat::freeze::{startup_from_file, StartupError};
use wat::types::TypeErrorKind;

/// `:probe::Odd::put -> Verdict` must be REFUSED at `defsurface` registration — located, naming
/// both the declared name (`:probe::Odd::Verdict`) and the required one
/// (`:probe::Odd::PutResponse`) — never silently accepted and never reached by any downstream
/// codegen (op-methods, serve-op-arms' size guard, serve-op-arms' shape guard all die upstream
/// of this check, which is exactly the point: the wrong name has no representation anymore).
#[test]
fn odd_verdict_response_name_is_refused_at_registration() {
    match startup_from_file("tests/services/probe_arc278_response_type_from_declaration.wat.bad") {
        Ok(_) => panic!(
            "expected `:probe::Odd::put -> Verdict` to be REFUSED at registration (arc 278 #74 \
             — `<Op>Response` is law); it froze clean instead, which means the law is not being \
             enforced"
        ),
        Err(StartupError::Type(e)) => match e.into_kind() {
            TypeErrorKind::MalformedDecl { head, reason } => {
                assert_eq!(head, ":wat::core::defsurface");
                assert_eq!(
                    reason,
                    "op `put` in surface :probe::Odd: response type name is LAW — declared \
                     `:probe::Odd::Verdict`, required `:probe::Odd::PutResponse` (arc 278 #74, \
                     builder ruling 2026-08-05: an op's response type IS `<Op>Response`; rename \
                     the declaration to match)"
                );
            }
            other => panic!(
                "expected MalformedDecl (the #74 response-name-law refusal); got {other:?}"
            ),
        },
        Err(other) => panic!(
            "expected StartupError::Type (the #74 response-name-law refusal); got {other:?}"
        ),
    }
}
