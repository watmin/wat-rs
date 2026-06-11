# DESIGN — Stone 251.3b: the faithful type-NAMESPACE fix (intueri-named)

**Status: SHIPPED 2026-06-10 (`f11d2bd6`). Sonnet-built (4-way ladder + DRY delegation + rename),
orchestrator-weighed + hardened.** The weigh caught the sonnet build's two STOP guards implemented
as `panic!` on shapes that are REACHABLE (`parse_type_expr` accepts `:foo::` and `:Stream<i64>`) on
a runtime-verb path → hardened `type_expr_to_clojure_form` to return `Result`, the two unmodeled
shapes return a clean `MalformedForm` (probe C08/C09). Probe `probe_arc251_type_namespace_fix` 9/9;
`keyword/to-type-form` 9/9 (incl. flipped `:56`); types 83/0 + lib 949/0 serial. Prereq to Strike 4
cleared. Home: `src/edn_shim.rs`.

## The bug (confirmed)
`type_expr_to_faithful_watast` (`edn_shim.rs:601`) renders EVERY named type as
`wat.type/<last-::-segment>` (`:610` Path arm, `:619` Parametric head — both `rsplit("::").next()`).
This LIES (intueri Level 1): `:wat::kernel::services::StdErrService::Req` and
`…StdInService::Req` both collapse to `wat.type/Req`. The doc at `:593` ("simple name only") and the
test assertion at `tests/probe_arc251_keyword_to_type_form.rs:56` (`:wat::holon::HolonAST` →
`wat.type/HolonAST`) **enshrine the lie**. Probe `probe_arc251_type_namespace_fix` RED at HEAD: C03
(`:String`→bare `String`), C04/C05 (the `Req` collision), C06 (`HolonAST` flattened).

## The named scheme (intueri, builder-ratified)
Core/scalar types live in a flat **reserved** `wat.type/` namespace; user/library types KEEP their
namespace. The discriminator on sight: **is the namespace exactly `wat.type`?** yes → core; no
namespace + Uppercase → type-var; otherwise → user type.

| rendered | kind |
|---|---|
| `wat.type/i64`, `wat.type/Vector`, `wat.type/Tuple` | core / built-in |
| `wat.kernel.services.StdErrService/Req`, `wat.holon/HolonAST` | user / library |
| `T`, `K`, `V` (bare, Uppercase) | type-var |

## The disk (grounded — reuse, don't re-encode)
- `wat_keyword_to_clojure_symbol` (`edn_shim.rs:1806`) — the SOUND, namespace-preserving inverse
  (`:wat::a::b::C::Req` → `wat.a.b.C/Req`). The type renderer must DELEGATE to it for user types
  (DRY — one path-splitting truth; the renderer currently re-implements a broken subset).
- `BARE_PRIMITIVES` (`check.rs:1650`) = `:i64 :f64 :bool :String :u8` and `BARE_CONTAINER_HEADS`
  (`check.rs:1665`) = `Option Result HashMap HashSet Vec`(→`Vector`). The substrate's existing
  primitive knowledge — REUSE it (make accessible to edn_shim, or a shared helper), do NOT hardcode
  a second list.
- **Empirical (probe):** `parse_type_expr` does NOT canonicalize bare primitives — `:i64` →
  `Path(":i64")`, `:String` → `Path(":String")` (bare, **Uppercase**), `:T` → `Path(":T")`. So the
  renderer DOES see bare primitives, and `:String` is bare+Uppercase — indistinguishable from a
  type-var by case alone. The primitive SET is the only sound discriminator.

## The strike — the 4-way discriminator (Path arm; Parametric head mirrors it)
For a `Path(s)` (strip leading `:` → `body`):
1. `body` starts with `wat::core::` → **core FQDN**: `wat.type/{body after "wat::core::"}`.
2. `:{body}` ∈ `BARE_PRIMITIVES` → **bare legacy primitive**: `wat.type/{body}`.
3. `body` contains `::` (and not core) → **user type**: `wat_keyword_to_clojure_symbol(":{body}")`.
4. else (no `::`, not a primitive) → **type-var**: bare `Symbol(body)`.
Order matters: 1+2 (core) BEFORE 3 (the `::` delegation), else `wat::core::i64` wrongly delegates.

Parametric `head` (stored without `:`): same ladder — `wat::core::` prefix → `wat.type/{tail}`;
`head` ∈ `BARE_CONTAINER_HEADS` → `wat.type/{canonical}` (note `Vec`→`Vector`); `::`-bearing →
delegate; recurse on args. `Fn`/`Tuple`/`Var` arms unchanged (`Tuple` stays `wat.type/Tuple` — core).

## Also (intueri)
- Rename `type_expr_to_faithful_watast` → `type_expr_to_clojure_form` (names the target; update
  callers — small, internal `pub(crate)`).
- Fix the lying doc at `:593` to describe the 4-way scheme.
- Flip the lying test assertion `probe_arc251_keyword_to_type_form.rs:56` →
  `:wat::holon::HolonAST` ⇒ `wat.holon/HolonAST`.

## Out of scope
- The `<T,U>` suffix drop + declaration migrator → Strike 4 / 4.1 (this only makes the renderer
  sound). Higher-kinded type-var heads (a bare type-var as a Parametric head) — not in the model;
  if the corpus has none, no handling needed (STOP + surface if found).
