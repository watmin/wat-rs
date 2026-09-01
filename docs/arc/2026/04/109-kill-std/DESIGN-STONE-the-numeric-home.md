# DESIGN — STONE 1 of 2: `src/numeric/` — the tower gets a home, split by CONCERN

> **Builder, 2026-09-01:** *"src/intrinsic/ is meant to wire up into the registry… the edge of wat's
> kernel. The actual implementations live in some proper home… impl and (registration, delegation)
> interface are not the same thing."*
>
> And the requirement that shapes it: *"we will add direct support for all of rust's numerics…
> prepare the file system such that these additions become trivial once we're ready."*

## ⛔ THE BOUNDARY I HAD BACKWARDS

I proposed moving verb bodies INTO `src/intrinsic/record.rs` to "kill the delegate-back." **That was
wrong and would have destroyed the boundary.** A delegate-back is the INTERFACE, not coupling:

```
src/intrinsic/<domain>      EDGE — registration + delegation. The kernel's rim.
src/<domain>/               IMPL — the actual work.
```

Already built seven times: `intrinsic/collection.rs → src/collection/`, `intrinsic/edn.rs →
src/edn/`, `intrinsic/holon/ → src/holon/`, `intrinsic/kernel/ → src/kernel/`, `rete`, `stream`,
`string`. Two edges are directories *because of size* — the edge may split; that is not the impl.

**Nothing is wrong with the delegation. What is wrong is where it POINTS**: at `runtime.rs`, because
these domains never got a home. Measured, excluding shared evaluator primitives (`eval_inner`,
`apply_function` — an edge calling those is legitimate and permanent):

```
69 domain impls still squatting in runtime.rs, behind 11 homeless edges.
41 of them are the numeric tower:  i64 14 · f64 14 · rational 7 · bigint 6
```

## ★★★ WHY THE NUMERIC TOWER IS FIRST, AND WHY ITS SHAPE IS THE WHOLE STONE

The builder's ruling turns a relocation into a **structural** requirement. Rust's numeric set is
i8/i16/i32/i64/i128/isize · u8/u16/u32/u64/u128/usize · f32/f64 — plus this substrate's `BigInt` and
`Rational`: **~16 types** against today's 5.

⛔ **Every numeric mechanism here is currently written PER ORDERED PAIR, which is quadratic:**

```
ordering    src/value/numeric_order.rs — ONE door, but a per-pair match.
            Its header says "promotes to the narrowest EXACT common representation" — the CONCEPT
            is a lattice; the CODE is same-type fast paths, then exact-integer pairs, then
            any-exact-vs-f64 both directions. 5 types ≈ 15 arms. 16 types ≈ 256.
arithmetic  eval_i64_arith · eval_f64_arith · eval_bigint_arith · eval_rational_arith — one per type.
conversion  eval_i64_to_f64 · i64_to_bigint · i64_to_rational · bigint_to_f64 · bigint_to_rational ·
            rational_to_f64 · f64_to_i64 · … — NINE numeric pairs for FOUR types.
```

★ **So "prepare the file system" cannot mean "move the files."** A relocation that preserves the
per-pair shape makes additions no more trivial than they are today. The file system's job is to put
each mechanism where it can become linear.

## THE ONE CONTRACT DECISION — pinned

**`src/numeric/` splits by CONCERN, never by TYPE.**

```
src/numeric/arith.rs     312 lines today
src/numeric/convert.rs   247
src/numeric/compare.rs    34
src/numeric/cast.rs   +  mod.rs — the tower's shape and its rank/promotion vocabulary
```

★★ **That is the property the builder asked for, stated exactly:** 16 types against per-TYPE files is
16 files and 16 edits to add one; against per-CONCERN files it is **four files, and adding a type
touches each concern once.** Per-type files would encode the very growth this stone exists to absorb.

## What ships in STONE 1 — and what does NOT

**Ships:** the 24 numeric impl fns (**778 lines**) move out of `runtime.rs` into `src/numeric/`,
split by concern. The four edges — `src/intrinsic/{i64,f64,bigint,rational}.rs` — re-point from
`crate::runtime::` to `crate::numeric::`. **Behaviour unchanged. No verb gains or loses anything.**

**Does NOT ship — stone 2:** the promotion lattice. Replacing the per-pair matches with a rank-based
`promote(a, b) -> CommonRepr` so ordering/arith/conversion each become *"promote once, then do the op
once."* **That** is what makes adding `i8` a row. It is a behaviour-preserving rewrite of real
algorithms and deserves a red that points at one thing.

⚠ Stone 1 is what makes stone 2 tractable: a tower spread through a 34,142-line megafile cannot be
sanely restructured. **Relocate, then reshape.**

## ★ THE PREDICTION — falsifiable

```
runtime.rs                     34,142  ->  ~33,360   (-778)
intrinsic/{i64,f64,bigint,rational}.rs   crate::runtime:: -> crate::numeric::, 41 sites
check.rs -> crate::intrinsic             0, unchanged
new cycle?                     src/numeric/ must NOT reference crate::intrinsic
behaviour                      every numeric verb identical, before and after
```

⚠ **`src/numeric/` referencing `crate::intrinsic` would create the cycle this whole architecture
exists to avoid.** The impl must not know about its own edge. That is the acceptance row that matters
most.

## Out of scope = REJECTED (not deferred)

- **The promotion lattice.** Stone 2, named above.
- **`src/value/numeric_order.rs`'s home.** It is the tower's ordering door but lives under `value/`
  and is reached through `Value` comparison. Moving it is defensible and is **a practitioner's call
  this stone does not make** — it ships where it is, and stone 2 decides.
- **The other 28 homeless impls** (`record` 10 · `result` 4 · `stat` 4 · `reflect` 3 · `option` 3 ·
  `math` 2 · `keyword` 2). Same shape, later stones; the numeric tower goes first because it is the
  largest and because the builder's ruling makes its shape urgent.
- **`u8`.** Already in `Value`, zero registered verbs. It moves with the tower; it does not get a
  surface here.

## THE FOUR QUESTIONS — flat YES/NO

| option | Obvious? | Simple? | Honest? | Good UX? | verdict |
|---|:---:|:---:|:---:|:---:|---|
| **`src/numeric/` split by CONCERN; lattice deferred to stone 2** | YES | YES | YES | YES | ✅ **ADMITTED** |
| relocate + build the lattice in one stone | YES | **NO** | YES | — | ⛔ **DISQUALIFIED** |
| `src/numeric/` split by TYPE (`i64.rs`, `f64.rs`, …) | YES | YES | **NO** | — | ⛔ **DISQUALIFIED** |
| move the bodies into `src/intrinsic/*.rs` | **NO** | YES | **NO** | — | ⛔ **DISQUALIFIED** |
| leave them in `runtime.rs`; add new types beside them | YES | YES | **NO** | — | ⛔ **DISQUALIFIED** |

- **one-stone Simple? NO** — a 778-line relocation plus a rewrite of ordering, arithmetic and
  conversion algorithms. A red could not be attributed to either.
- **split-by-TYPE Honest? NO** — it claims to prepare for 16 types with a layout that grows one file
  per type. It makes the addition *look* trivial while multiplying the surface, which is the opposite
  of the ask.
- **bodies-into-`intrinsic` Obvious? NO / Honest? NO** — this was MY proposal and the builder
  corrected it: it collapses edge and impl into one file, and the seven existing pairs are the
  counter-example. `[[NOTE-the-crate-boundary-is-the-real-cut-and-eight-homes-are-cyclic]]`
- **leave-it Honest? NO** — adding twelve integer types to a per-ordered-pair tower inside the
  largest file in the tree is the situation `extirpare` says not to construct.

## Acceptance

| what | command | expected |
|---|---|---|
| ★ the impl does not know its edge | `grep -c "crate::intrinsic" src/numeric/*.rs` | **0** — no new cycle |
| the edges re-point | `grep -c "crate::runtime::" src/intrinsic/{i64,f64,bigint,rational}.rs` | 0 for domain impls |
| the megafile sheds it | `wc -l src/runtime.rs` | ~33,360 (−778) |
| ★ split by concern, not type | `ls src/numeric/` | arith · convert · compare · cast · mod — **no per-type file** |
| behaviour unchanged | every numeric verb, before and after | identical |
| `check.rs` untouched | `grep -c "crate::intrinsic" src/check.rs` | still 0 |
| floor | `scripts/floor.sh`, exit read UNPIPED | 5114/5114, 0 failed |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | 0 |
