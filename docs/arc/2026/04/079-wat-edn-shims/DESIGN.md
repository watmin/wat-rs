# Arc 079 — `:wat::edn::*` shims (render any wat value as EDN/JSON)

**Status:** PROPOSED 2026-04-29. Pre-implementation reasoning artifact.

**Predecessors:**
- `crates/wat-edn/` ships a complete EDN parser/writer/JSON-bridge in Rust today (313 Rust tests + 39 Clojure tests, all green). Workspace member; no wat consumers yet.
- Arc 064 — `:wat::core::show` polymorphic renderer (existing primitive that walks any wat value and produces a human-readable string).

**Surfaced by:** The user's data-not-text directive (2026-04-29):

> "no free form log lines... no rando (println! ...) bullshit.... the users must operate on data at all times"

The Console destination in arc 081 (telemetry::Console) needs to render structured records as EDN-per-line OR JSON-per-line. The renderer is a substrate primitive — useful far beyond logging.

---

## What this arc is, and is not

**Is:**
- Rust shims that expose `wat-edn`'s `write` / `write_pretty` / `to_json_string` to wat through `#[wat_dispatch]`.
- A surface map between wat values and `wat_edn::Value`. Walks any wat value; produces a `wat_edn::Value`; calls the renderer.
- One-line wat surface: `(:wat::edn::write v)` → `:String`, `(:wat::edn::write-json v)` → `:String`.

**Is not:**
- A parser surface in slice 1 (read-back lands in slice 2 if a consumer surfaces a need).
- A new Value type. Existing wat values walk through; the renderer translates at the boundary.
- A logging primitive. This is a value-renderer; logging is downstream.

---

## Surface

```scheme
;; Compact single-line EDN
(:wat::edn::write
  (v :Any)
  -> :String)

;; Multi-line pretty EDN (stable indentation)
(:wat::edn::write-pretty
  (v :Any)
  -> :String)

;; JSON (round-trip-safe via sentinel-key tagged objects per wat-edn's design)
(:wat::edn::write-json
  (v :Any)
  -> :String)
```

`:Any` here means "any wat value the runtime can walk." The Rust side dispatches on `wat::Value` variants and emits `wat_edn::Value`; rendering happens in Rust. Wat callers see strings.

---

## Mapping wat → wat-edn::Value

| wat | wat-edn::Value |
|---|---|
| `()` (unit) | `Nil` |
| `bool` | `Bool` |
| `i64` | `Integer` |
| `f64` | `Float` (NaN/Inf via sentinel tags per wat-edn convention) |
| `String` | `String` |
| `:keyword` | `Keyword` (canonical EDN keyword form) |
| `:ns::path` | `Keyword` with namespace |
| `Vec<T>` | `Vector` |
| `(a, b, c)` (tuple) | `Vector` (no tuple distinction in EDN) |
| `Option<T>::Some(v)` | `v` (transparent) |
| `Option<T>::None` | `Nil` |
| `(:wat::core::struct ...)` | `Map` with `:keyword` keys = field names |
| `(:wat::core::enum ::Variant ...)` | `Tagged` with `#namespace/Variant` for variant identity |
| `:wat::holon::HolonAST` | structural — leaf primitives directly; composites as `Tagged` |
| `:wat::holon::Vector` | `Tagged #wat-edn.holon/Vector [...]` (or compact `:bytes` form) |

The mapping is a one-time decision per type. It's the substrate's "show this thing as data" translator.

---

## Slice plan

Single slice. The crate exists; the shim is mechanical.

1. New file `src/edn_shim.rs` — `eval_edn_write`, `eval_edn_write_pretty`, `eval_edn_write_json` functions; one walker `value_to_edn(value: &Value) -> wat_edn::Value` shared across them.
2. Register the three primitives in `runtime.rs`'s op-dispatch table.
3. Type schemes in `check.rs`: each takes `:Any` and returns `:String`.
4. Add `wat-edn = { path = "crates/wat-edn" }` as a dep in `wat-rs/Cargo.toml` (it's already a workspace member, just needs to be on wat-rs's path).
5. Tests at `wat-tests/edn/render.wat`:
   - Compact: `(write [1 2 3])` → `"[1 2 3]"`
   - Pretty: a struct + nested vec produces multi-line output
   - JSON: round-trips through wat-edn's JSON surface
   - Sentinel cases: `f64::NAN`, `Vec<()>`, empty struct

---

## Open questions

### Q1 — Polymorphic `:Any` argument

Wat's type system doesn't have an `:Any` parameter today. Two options:
- Add `:Any` as a substrate-recognized "I'll dispatch at runtime" marker — opens the door for other render-style primitives.
- Make these primitives a special-case in the type checker (similar to `assert-eq`'s polymorphic-equality dispatch).

Default: special-case in slice 1; revisit if a third polymorphic-render primitive surfaces. Per "verbose is honest" — don't introduce `:Any` until two consumers need it.

### Q2 — HolonAST rendering shape

HolonAST has 11 variants (5 leaves + 5 composites + Atom-wrap). The renderer needs to pick a form. Options:
- **Tagged-EDN per variant** — `#wat-edn.holon/Bind {:role <r> :filler <f>}`. Round-trippable.
- **Lispy form** — `(Bind <r> <f>)`. Reads cleanly; loses tag distinction from list.
- **Hybrid** — primitives render as their EDN equivalent; composites render as tagged.

Default: hybrid. Read-friendly without losing identity.

### Q3 — Vector (HD vector) rendering

Vectors are large (10000-D). Options:
- Render as `Tagged #wat-edn.holon/Vector "<base64-bytes>"`.
- Render as `Tagged #wat-edn.holon/Vector {:dim 10000 :sha256 "..."}` (header only).
- Special-case at the consumer level (telemetry rarely renders raw vectors).

Default: header-only. Logs don't need the bytes; consumers that need the bytes use `vector-bytes` + hex.

---

## Test strategy

- `wat-tests/edn/render.wat` — six deftests covering compact/pretty/json across the substrate's value categories.
- The render output is asserted as exact-string. EDN's deterministic write means string equality is the right test.

---

## Dependencies

**Upstream (must ship before this arc lands):** none. `wat-edn` crate is complete; wat-rs already pulls it transitively through workspace.

**Downstream (this arc unblocks):**
- Arc 081 (telemetry::Console) — the renderer is the boundary that turns TelemetryEntry into a stdout line.
- Future debugging primitives (`:wat::dev::dump value`).
- Future cross-process IPC where wat values travel as EDN bytes.

**Independent of:** Arc 080 (Telemetry substrate promotion) — that work doesn't need rendering. Arc 082 (docs) — purely educational.

PERSEVERARE.
