# BRIEF — migrate `Value::wat__core__PersistentMap` to `PMap`

Design: `DESIGN-STONE-promoting-map.md`. The type is built, landed and proven: `src/value/pmap.rs`
(`1236f6ad`), five laws green. **This brief is the migration only.**

## The work, in one paragraph

`Value::wat__core__PersistentMap` holds a raw `rpds::HashTrieMapSync<Value, Value>`
(`src/value/value.rs:105`). Change it to hold `crate::value::pmap::PMap`, then follow the compiler
to every site. `PMap` already has `new`/`from_pairs`/`get`/`contains_key`/`len`/`is_empty`/`iter`/
`assoc`/`dissoc`/`keys`/`values`/`is_trie`. The cascade will ask for a few more — add them to
`pmap.rs`; do not scatter conversions at call sites.

## The measured shape of the cascade — this is a MAP, not a warning

I ran the swap and reverted it. At the moment the variant changes you get **43 errors**:

| kind | n | what it is |
|---|---|---|
| `E0308` mismatched types | 26 | a `HashTrieMapSync` being wrapped in the variant, or unwrapped and used as one |
| `E0599` no method | 12 | `size` (5), `filter_map` (3), `cloned` (2), `insert` (1), `remove` (1) |
| `E0614` deref | 3 | — |
| `E0277` trait | 1 | `Bindings` is not implemented for `PMap` |

By file: `rete/kernel.rs` 29, `value/value.rs` 22, `rete/matcher.rs` 7, `collection/eval.rs` 6,
`edn_shim.rs` 1, `closure_extract.rs` 1. **The fail-count is the progress meter**
(`docs/SUBSTRATE-AS-TEACHER.md`); watch it waterfall, do not panic at 43.

## Read in order, and why you are being sent there

1. **`src/value/pmap.rs`** — whole file. The type and, more importantly, the five laws in its test
   module. They define what must stay true.
2. **`src/value/value.rs:105`** — the variant. **`:613`** — the `PartialEq` arm: it reads
   `(PersistentMap(a), PersistentMap(b)) => a == b` and **keeps working unchanged**, because `PMap`
   implements `PartialEq` as entry-set comparison. **`:787`** — the `Hash` arm: replace the inline
   sorted-pair-hash body with `m.hash(state)`; `PMap`'s `Hash` is that same routine, moved, so one
   routine now covers both arms.
3. **`src/rete/matcher.rs:86-116`** — the `Bindings` trait and its three impls. Add a fourth for
   `PMap`. ⚠ Read the comment above the trait first: it says *"must NEVER grow an `insert`"* and
   explains why `Token.bindings` stays a trie.
4. **`src/collection/eval.rs`** — the `PersistentMap/*` op implementations, the 6-error cluster.

## Two things I got wrong when I tried this — do not repeat them

**1. A blanket regex over `Value::wat__core__PersistentMap(...)` made it WORSE — 37 errors became
56.** It rewrote sites that were already correct. Fix sites the compiler names, one at a time.

**2. Wrapping an existing trie as `PMap::Trie(t)` is the subtle wrong answer.** A 2-entry map
arriving that way keeps its HAMT forever and silently opts out of promotion — the exact thing the
type exists to prevent. Add and use:

```rust
/// Adopt an existing trie, CHOOSING THE ARM BY SIZE.
pub fn from_trie(t: rpds::HashTrieMapSync<Value, Value>) -> Self;
/// The trie view, materialising one from the array arm when a reader genuinely needs rpds.
pub fn to_trie(&self) -> rpds::HashTrieMapSync<Value, Value>;
```

## ★ THE ROW THAT MUST LAND FIRST — it is currently unproven and it is the only silent failure

The instant the variant holds a `PMap`, add this to `pmap.rs`'s test module **before** fixing the
rest of the cascade:

> Build a ≤8-entry map two ways — small-build (lands **Array**) and build-past-8-then-`dissoc`-back
> (stays **Trie**, because promotion is one-way). Put one in a map **as a key**. Look it up with the
> other. Both directions. Then the same through `edn::read` → `write` → `read`.

Why it is not optional: **`{{:some :map} :as-a-key}` is legal EDN and round-trips correctly today**
— verified this session — so a peer can send one over the wire. If cross-arm key lookup breaks, the
map simply **misses**. No error, no panic, a wrong answer. Every other failure in this stone
produces a visibly wrong value; this one is silent.

## Blast radius

`src/value/value.rs`, `src/value/pmap.rs`, `src/rete/kernel.rs`, `src/rete/matcher.rs`,
`src/collection/eval.rs`, `src/edn_shim.rs`, `src/closure_extract.rs`. **Nothing under `wat/`** — no
`.wat` file changes, no new wat verb, no surface change. This is invisible to every wat program.

## STOP triggers — rejection criteria. Ship nothing; report the gap.

**⚠ STOP-1 WAS WRONG — corrected 2026-08-01, after the migration landed.** It is left verbatim
below because the rider obeyed it and the record should show what it obeyed. The builder overturned
it: *"the entire array up to 8, trie for rest came from this exact situation… and chose not to use
it?"* Our own `element-bindings-array.md:94` says *"The 8-crossover stays interesting for
`Token.bindings`"* — that stone NAMED Token as where this mattered, and ruled the other way only
because the choice was then binary (array XOR trie, once, globally). A promoting map is the third
option it never had. `DESIGN-STONE-token-bindings-promoting.md` is the correction.

**STOP-1 (as written, and obeyed).** If `Token.bindings` (`kernel.rs:49`) appears to need changing — STOP. It stays a raw
`HashTrieMapSync` by ruling (`DESIGN-STONE-element-bindings-array.md` measured build/lookup/clone/
drop at every width and kept the trie). Convert at the boundary with `from_trie`/`to_trie`; do not
migrate Token.

**STOP-2.** If any cross-arm law in `pmap.rs`'s test module goes red — STOP and report which. Those
laws are the wall; a migration that loosens one has broken the stone rather than landed it.

**STOP-3.** If a site seems to need `PMap` to expose which arm it is holding (beyond the existing
`is_trie()`, which exists ONLY so a test can prove promotion fired) — STOP. Representation must be
unobservable; a caller branching on the arm is the leak.

## The gate — run these yourself, in the FOREGROUND

1. `pmap.rs`'s laws, including the new map-as-key row.
2. `cargo nextest run --release` — the Summary line, 4259+ passing, 0 failed. **Never read a piped
   exit code**; `grep -c` returns 1 on a zero count and will look like a failure.
3. `cargo clippy --release --all-targets` — 0.
4. The nine grid axes: `:accuracy :match`, `:derived` byte-identical.
5. `{{:some :map} :as-a-key}` still round-trips through `edn::read`/`write` — it does today, and a
   regression there is the wire breaking.

## You are a rider, not the orchestrator

**Ending your turn ENDS you** — it does not suspend you, and nothing will wake you. There is no
notification coming. Run every verification in the FOREGROUND and block on it: your turn ends when
the numbers are in your hands, not when the command is launched. Do not commit, do not push, do not
stash. Report what you ran and what it said.
