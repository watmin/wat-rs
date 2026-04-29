# Arc 079 — `:wat::edn::*` shims (render any wat value as EDN/JSON) — INSCRIPTION

**Status:** shipped 2026-04-29.

The substrate now ships three render primitives that walk any wat
value and produce a serialized EDN or JSON string. Built on the
existing `wat-edn` crate (313 Rust + 39 Clojure tests, all green).
The wat-side caller writes one form; the Rust shim does the walk;
the renderer emits the string.

This is the boundary that turns structured wat values into log
lines, IPC payloads, and dev-time `dump`-style diagnostics. The
data-not-text discipline begins here: every line that crosses a
process boundary goes through this primitive.

**Design:** [`DESIGN.md`](./DESIGN.md).

---

## What shipped

### Single slice — three primitives

`src/edn_shim.rs` (new module):
- `eval_edn_write(args, env, sym)` — compact single-line EDN.
- `eval_edn_write_pretty(args, env, sym)` — multi-line indented EDN.
- `eval_edn_write_json(args, env, sym)` — round-trip-safe JSON via
  wat-edn's tagged-object convention for non-JSON-native types.
- `value_to_edn(v: &Value) -> wat_edn::OwnedValue` — the single walker
  shared by all three. One mapping per Value variant.

`src/runtime.rs`:
- Op-dispatch table extended for the three primitive paths.

`src/check.rs`:
- Type schemes registered: each takes `:Any`-equivalent (fresh var)
  and returns `:String`.

`Cargo.toml`:
- `wat-edn = { path = "crates/wat-edn" }` already a workspace member;
  added to wat-rs's direct deps.

### Mapping wat → wat-edn (per DESIGN's table)

| wat | wat-edn::Value |
|---|---|
| `()` | `Nil` |
| `bool` / `i64` / `f64` / `String` | direct EDN scalars |
| `:keyword` / `:ns::path` | `Keyword` (with namespace where applicable) |
| `Vec<T>` | `Vector` |
| tuple `(a,b,c)` | `Vector` (no tuple distinction in EDN) |
| `Option<T>::Some(v)` | `v` (transparent) |
| `Option<T>::None` | `Nil` |
| struct | `Map` keyed by `:field-name` |
| enum variant | `Tagged` with `#namespace/Variant` |
| `:wat::holon::HolonAST` | hybrid — primitives direct, composites tagged |
| `:wat::holon::Vector` (HD vector) | `Tagged #wat-edn.holon/Vector` (header only) |

Per DESIGN's open-question Q3 default: HD vectors render header-only
(dim + sha256), not raw bytes. Logs don't need bytes; consumers that
do reach for `vector-bytes` + hex.

### Tests

`wat-tests/edn/render.wat` — **10 deftests** covering:
- compact EDN: scalar / vec / nested struct / enum
- pretty EDN: deterministic multi-line indentation
- JSON: round-trip via wat-edn's JSON shape
- sentinel cases: `f64::NAN`, `Vec<()>`, empty struct/enum

All 10 green; `cargo test -p wat` clean.

---

## Open questions resolved by what shipped

- **Q1 — Polymorphic `:Any` argument:** special-cased in the type
  checker (similar to `assert-eq`'s polymorphic-equality dispatch).
  No `:Any` type primitive added per the verbose-is-honest discipline;
  the renderer dispatches at runtime over `Value` variants.
- **Q2 — HolonAST rendering:** hybrid, per DESIGN default.
- **Q3 — HD Vector rendering:** header-only, per DESIGN default.

## What's still uncovered

- **Parser surface (read-back).** `wat-edn` has it; the wat shim
  doesn't expose it yet. Lands when a consumer surfaces a need (e.g.,
  cross-process IPC where wat values arrive as EDN bytes from
  outside).
- **`:wat::dev::dump` development helper.** The renderer is the
  primitive; a dev-side wrapper that pretty-prints to stderr at
  arbitrary points is a future convenience.

## Consumer impact

Unblocked:
- **Arc 081** (telemetry::Console) — built directly on top of these
  primitives. The Console dispatcher is render + send-tagged-stdout.
- **Future debugging primitives** — a single-call dump-to-stderr
  surface lands when a consumer needs it.

PERSEVERARE.
