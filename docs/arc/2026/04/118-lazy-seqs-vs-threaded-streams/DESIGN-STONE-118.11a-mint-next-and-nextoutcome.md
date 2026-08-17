# DESIGN STONE — 118.11a · mint `next` + `NextOutcome`. Additive. The memo stays.

**Builder, 2026-08-17: *"ok... let's build next... make it a stone."***

This is **stone A of two**, and the split is load-bearing: **A is purely additive** — a new enum, a
new native verb, zero call sites changed, nothing can regress. **B** migrates the walkers and *then*
deletes the memo, because the memo cannot die while anything still walks with three calls.

## What it is

```wat
(:wat::stream::next s) -> :wat::stream::NextOutcome<T>
   :Item      [value <- T  rest <- :wat::stream::Stream<T>]
   :Exhausted []
```

One call. One force. Value and tail together. A named end.

## Why (the measured chain, in one line each)

- The walk protocol is `empty?` → `first` → `rest` — **three forces of one cell**.
- Without a cache that calls **user code 3× per element** — measured: **15 calls for 5 elements**.
- The cache patches it — measured: restores exactly **5 for 5**.
- The cache links every cell to its tail, so the head pins the realized chain — measured:
  **+297 B/element**, linear, and it is the *entire* lazy overhead (memo-off matches eager `mapv`
  within 200 KB on 326 MB).
- **Neither arm ships:** memo-on is silently wrong for effectful `f`; memo-off OOMs.

`next` is the only shape that gets both: one force **structurally**, so there is nothing to dedupe
and no cache to retain.

Full evidence: `MEASURED-118.8`, `DESIGN-118.10`.

## The four questions

- **Obvious? YES** — one call returns what you need to continue; the end is a named arm. It is
  `Iterator::next`, Ruby's `next`, and the house's own ten `*Outcome` verbs.
- **Simple? YES** — one enum, one native verb. Additive; no call site moves in this stone.
- **Honest? YES** — it faces the domain hole `first` answers with a bare `nil` today. And it stops
  the substrate depending on a cache to keep user code from running three times.
- **Good UX? YES** — one `match`, both halves bound, and the three-call sequence becomes
  unwritable *in stone B* because there is nothing to sequence.

## Rooms

| what | where | note |
|---|---|---|
| declare the enum | `src/types.rs` — exemplar `RecvOutcome`'s variants at **`types.rs:1662`** (`name: "Message".into()`) | parametric in `T`, like `RecvOutcome<O>` |
| build a variant from Rust | `builtin_enum_variant_names` — **`runtime.rs:21990`**; a wrap site at **`runtime.rs:7058`** | the door; do not hand-roll |
| the force itself | **`src/stream/mod.rs:158`** `realize` | `next` = realize to WHNF, then destructure Cons |
| dispatch arm | `src/runtime.rs` beside the other `:wat::stream::*` arms | |
| `TypeScheme` | `src/check.rs` | `Stream<T> -> NextOutcome<T>`, `type_params: vec!["T".into()]` |

★ **`next` does NOT need a surface.** Only `Stream` is ever pullable — a user's own sequence
implements `Seqable`/`seq` and builds a `Stream` from `lazy`/`cons`, getting pull for free. Proven:
`:my::countdown` runs today with no container behind it.

## The gate

| # | assertion |
|---|---|
| 0 | ★ **NON-VACUITY:** `:wat::stream::next` does not exist before the change — capture the unknown-verb error verbatim |
| 1 | `(next <3-element stream>)` → `NextOutcome::Item` with `value` = first element |
| 2 | `(next <exhausted stream>)` → `NextOutcome::Exhausted` |
| 3 | ★★ **ONE FORCE PER CALL** — with a printing `f`, a single `next` on `(map f v)` prints **exactly one** line. This is the whole stone; if it prints 2+, the verb is walking more than one cell |
| 4 | pulling `rest` from row 1 and calling `next` again yields the **second** element |
| 5 | the memo is **untouched** — `git diff src/stream/mod.rs` shows no change to `forced` |
| 6 | every existing stream verb byte-identical — `map`/`filter`/`keep`/`into`/`doall` unchanged, floor proves it |
| 7 | a **kept** test covers rows 1–4 |
| 8 | floor GREEN via `scripts/floor.sh` — the Summary line |
| 9 | `cargo clippy --release --all-targets` → **0** |
| 10 | `#[ignore]` count **13** |

Row 3 is the stone. Rows 5 and 6 are what make it additive.

## Out of scope — affirmative cuts

- **Deleting the memo.** Stone B. It cannot die while `empty?`/`first`/`rest` walkers exist.
- **Migrating the 7 twins and the drain verbs.** Stone B.
- **Fixing `dorun`** (builds a Vector and bins it), **`length`** (type-checks then raises),
  **`first`** (bare `nil`). All consequences of B, not A.
- **`Seqable`.** Downstream of both — minting the interface before the protocol is fixed would
  freeze the broken shape into a user-facing type.
- **Naming `:Exhausted` vs `:Done` vs `:End`.** `Exhausted` matches the substrate's outcome
  register; not worth a round trip. Say so and move.

## What A does NOT prove

That the fused pull reaches O(1) — **the memo is still in place**, so memory is unchanged by this
stone and row 6 requires that. The O(1) claim is stone B's acceptance test, and it is still a
**prediction**: my earlier prediction that removing the memo alone would reach O(1) was wrong (it
reached eager parity). Predictions in this area have a poor record; B measures.
