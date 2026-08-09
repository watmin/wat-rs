# BRIEF — a registered Rust opaque READS IMPURE, and it enrolls itself

> **Builder-ruled 2026-08-08:** *"this is unacceptable … i want to attack this immediate symptom now …
> if its a foreign symbol we just deny it?… that feels like a correct, cheap fix?"*
>
> Correct — **scoped to foreign OPAQUES.** Denying unknown symbols *generally* is a 2713-test cascade
> (measured, below). Denying **registered Rust opaques** is zero-blast (measured, below). This brief
> builds the second, which is the builder's fix made precise.

## The work, one paragraph

`is_pure_type` (`src/check.rs:12848`) decides a type's purity. For a **Rust opaque** its knowledge is
two hand-written lists — eight `TypeExpr::Path` names, and a `TypeExpr::Parametric` head list — and
anything absent reads **pure**. So a live, thread-owned `:wat::cache::Lru<String,i64>` can be declared
as a field of a pure record, or in a defservice's `:durable`, and it compiles. Every `#[wat_dispatch]`
opaque **already self-registers its path** into `RustDepsRegistry.types` (`src/rust_deps/mod.rs:202`),
and that registry is reachable from the checker (`crate::rust_deps::registry()`, already called at
`check.rs:14751`). Make `is_pure_type` consult it: **a path in the Rust-opaque registry is impure.**
Self-enrolling, one source of truth, no new hand list.

## Why "impure, period" and not "projected from `scope`"

`#[wat_dispatch]` exists to expose a **live Rust value** to wat. There is no `#[wat_dispatch]` type
that is pure EDN data — that is what `defrecord` is for. All three scopes (`shared`, `thread_owned`,
`owned_move`, `crates/wat-macros/src/lib.rs:136`) wrap a Rust handle. So the rule needs no `scope`
threading and no new attribute field: **registered as a Rust opaque ⇒ impure.** Simpler, and it cannot
drift out of step with the macro.

## Read in order

1. `src/check.rs:12848` — `is_pure_type`, the whole fn. Note **both** arms that admit an opaque:
   - `TypeExpr::Parametric` → head match, then `_ => args.iter().all(is_pure_type)`
   - `TypeExpr::Path` → hardcoded names, then `types.get(p)`, then `None => true`
2. `src/rust_deps/mod.rs:141` (`RustTypeDecl`), `:202` (`register_type`), `:266` (`registry()`).
3. `src/check.rs:14751` — a live example of reaching the registry from the checker.
4. `crates/wat-macros/src/codegen.rs:75-79` — where each opaque emits its `register_type` call.
5. `docs/arc/2026/06/293-struct-record-symmetry/NOTE-containment-wall-blind-to-rust-opaques.md` —
   the full grounding, both arms, and the measured blast radius.

## Implementation sketch

In `is_pure_type`, before each fallthrough, ask the registry:

```rust
// A path registered by #[wat_dispatch] is a live Rust handle — never EDN-representable.
fn is_registered_rust_opaque(path: &str) -> bool {
    let bare = path.strip_prefix(':').unwrap_or(path);
    let reg = crate::rust_deps::registry();
    // ⚠ NORMALISATION: the registry stores the attribute's literal `path` (LEADING COLON,
    // e.g. ":rust::cache::Lru"); a TypeExpr head is bare ("rust::cache::Lru"). Compare
    // ONE normalised form on BOTH sides — see STOP-1.
    reg.has_type(bare) // add this accessor if absent; do NOT reach into the field directly
}
```

- **Parametric arm** — consult it on `head` *before* the `_ => args…` fallthrough.
- **Path arm** — consult it *before* `types.get(p)`; leave `None => true` **UNTOUCHED**.

## Blast radius — MEASURED, and it is zero

Every `#[wat_dispatch]` opaque in the tree: `:rust::cache::Lru`, `:rust::sqlite::Connection`,
`:rust::sqlite::ReadConnection`, `:rust::lru::LruCache` (study-only crate oracle). A whole-corpus sweep
for fields typed by any of them across `wat/ wat-tests/ wat-scripts/ tests/ crates/` = **18 sites, and
not one is an illegal aggregate field** — all are fn parameters, correct `:ephemeral` slots, or
`:wat::cache::HolographicLru`, which is already a `defstruct`. **Expect the floor to stay 4376/0.**

## ⛔ STOP triggers

1. **STOP-1 — NORMALISATION.** The registry key and the `TypeExpr` head almost certainly differ by a
   leading `:`. This arc's own recurring defect is *"a string comparison with one side normalised and
   the other not"* — three instances in arc 278 alone. **Prove the comparison matches by a RUN** (the
   acceptance probe below going RED is that proof), not by reading. If it does not match, the probe
   stays green and you have shipped nothing.
2. **STOP-2 — DO NOT touch `None => true` in the Path arm.** Measured: flipping it turns **2713 of
   4376** tests red, because that arm is load-bearing for (a) formal type parameters (`:K`) and (b)
   six of our own unregistered core types (`PersistentMap`, `PersistentVector`, `WatAST`, `HolonAST`,
   `time::Instant`, `time::Duration`). That is a separate, larger stone — see the tail of this brief.
   If you find yourself editing that arm, STOP.
3. **STOP-3 — DO NOT delete the eight hardcoded Path names.** Only four paths are `#[wat_dispatch]`;
   `IOWriter`/`Hologram`/`Engram`/etc. register by another route and are NOT in this registry.
   Deleting them would re-open a hole this brief is closing. They dissolve under arc 255, not here.
4. **STOP-4 — if the floor moves off 4376/0, STOP and report the failing test's whole block verbatim.**
   The measurement says it should not move. A move means the registry contains more than expected, and
   that is a finding, not something to code around.

## The acceptance gate — a probe that must FLIP

`wat-scripts/scratch-pad/probe-293w-durable-admits-unenrolled-opaque.wat` **exists and loads GREEN
today, and that is the defect.** It declares `:durable [cache <- :wat::cache::Lru<String,i64>]` on a
defservice.

- **Before your change:** `./target/release/wat --check <that file>` → exit 0.
- **After your change:** it MUST go **RED** with `ImpureFieldInPureAggregate`.

That flip is the whole proof. When it flips, **delete that scratch file** (its header says to) and put
a permanent negative fixture in its place under `tests/` in the house style, so the wall keeps a
standing gate. Also add a positive control in the same fixture: a pure `:durable [count <- i64]` that
still loads, so the gate cannot pass by refusing everything.

## Weigh

`./scripts/floor.sh` → the **Summary line** (never a piped exit). Expect **4376 passed / 0 failed**.
Then `cargo clippy --release --all-targets` → 0.

## ── NOT IN SCOPE (the larger stone this measurement uncovered) ──

Flipping `None => true` to deny-by-default is the real wall, and the checker has already enumerated its
worklist. Under the experiment, the distinct offending field types were:

| type | hits | class |
|---|---|---|
| `:wat::core::PersistentMap` (+ parameterised) | 435 + 81 | our own core type, genuinely PURE, unregistered |
| `:wat::WatAST` (+ inside `PersistentVector`) | 246 + 187 | ” |
| `:wat::time::Instant` | 112 | ” |
| `:wat::holon::HolonAST` | 93 | ” |
| `:wat::time::Duration` | 92 | ” |
| `:wat::core::PersistentVector` | 83 | ” |
| `:K`, `Vector<K>`, `Vector<Entry<K,V>>` | 88 + 95 + 85 | **formal type parameter** — must never deny |

**Six core families to enrol as pure, plus a type-parameter question.** Not one is a foreign opaque —
which is why the naive flip costs 2713 tests and buys nothing this brief does not already buy. That
enrolment is arc **255**'s registry work (`255/NOTE-purity-is-definition-time-queryable-metadata.md`
already files this as the class's THIRD instance), or a smaller dedicated pass. **Do not start it here.**
