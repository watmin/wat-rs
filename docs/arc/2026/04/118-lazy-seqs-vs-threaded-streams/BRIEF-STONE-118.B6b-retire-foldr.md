# BRIEF — STONE 118.B6b · retire `foldr`, and make two capability headers honest

`:wat::core::foldr` is `reverse` + `foldl` wearing a name borrowed from Haskell, where the verb is
distinct only because it is **lazy** — a property strict wat cannot have. Delete it. The operation is
spelled `(reduce f init (reverse coll))` from verbs that already exist.

Read `DESIGN-STONE-118.B6b-retire-foldr.md` first: it carries the ruling, the prior art, and three traps.

## ⛔ YOU DO NOT RUN THE FLOOR

**You MAY**, in the FOREGROUND: `cargo build --release`, `./target/release/wat --check <file>`, a
`.wat` probe, and a SCOPED `cargo nextest run --release -E 'test(<pattern>)'`.
**You may NOT** run `scripts/floor.sh` or an unscoped `cargo nextest`. The orchestrator measures
centrally, once, on a quiescent tree. Ask in your report if you want the full picture.

## Read in order

1. **`src/collection/transform.rs:635–680`** — `eval_vec_foldr`. The whole verb is
   `xs.iter().rev()` then accumulate. Delete it and its four container arms.
2. **`src/rete/vocabulary.rs:919`** — `foldr`'s `ReteOp`, `class: OpClass::Redispatch`, ruled
   pure/deterministic/total. ★ **This row must go too**, and it is the one most likely to bite.
3. **`wat/seq.wat:308`** — `reduce`, a two-arm defclause already delegating to native `foldl`. This
   is the replacement; **do not touch it**.
4. **`src/collection/seq_container.rs:250–285`** — the `mappable()` and `ordered()` headers. Both
   are wrong after this stone; see step 5.

## The strike path

**1 — delete the verb.** `eval_vec_foldr` and its dispatch arm (`runtime.rs`), `infer_foldr`
(`collection/infer.rs`), the `check.rs` registrations, `collection/mod.rs`'s export.

**2 — delete it from THREE separate ledgers that do not know about each other.** Removing it from one
does nothing for the others; expect a red from whichever you miss:
- `is_pure_total` — `src/macros/eval.rs` (1 hit)
- `intrinsic_meta` — `src/rete/purity.rs` (3 hits)
- the rete vocabulary — `src/rete/vocabulary.rs` (4 hits)

★ Stone B4-0's rider discovered `rete::purity::completeness_gate::every_dispatched_verb_is_classified_or_disposed`
the hard way, by going red mid-strike. **Expect a vocabulary-side gate of the same shape.** Look for
one before assuming the `ReteOp` row is free to remove.

**3 — migrate the 4 test call sites.** `tests/rete/probe_arc278_seq1b_list_hofs.rs` (×2),
`tests/collection/probe_arc278_0d_transform_dispatch_parity.wat`,
`tests/collection/probe_arc278_0c_persistent_parity.wat`. **Rewrite, do not delete** — each measures
a right fold and still should, now spelled `(reduce f init (reverse coll))`. Ask what each one
MEASURES and preserve that. `[[feedback_ask_what_a_test_measures_before_fixing_how_it_measures]]`

**4 — the rename-table entry.** `wat-scripts/fixes/rete-where-per-type-spelling.wat:83` holds the
pair `(":wat::core::foldr" ":wat::rete::core::foldr")`. It is a *string literal in a migration table*,
not a call. Remove the pair.

**5 — make BOTH capability headers honest** (`src/collection/seq_container.rs`):
- `mappable()`'s doc says it gates `foldl`/`foldr`/`reverse`/`concat`. Drop `foldr`.
- ★ `ordered()`'s doc (`:277`) says it gates `reverse`/`take`/`drop`/`concat`. **Measured today: it
  has exactly two live consumers** — `concat` (`collection/eval.rs:763`) and `reverse`
  (`collection/transform.rs:51`). `take`/`drop` were moved to `extract_lazyable_elem`'s fixed set in
  118.2a; `collection/infer.rs:1070` records it: *"classification no longer routes through
  `ordered()`."* Correct the header to name its two real consumers.

## Blast radius

`src/collection/transform.rs`, `src/runtime.rs`, `src/check.rs`, `src/collection/infer.rs`,
`src/collection/mod.rs`, `src/collection/seq_container.rs`, `src/macros/eval.rs`,
`src/rete/purity.rs`, `src/rete/vocabulary.rs`, the 4 test files, and the one codemod line.
**43 `src/` sites measured.** `wat/seq.wat` is NOT touched — `reduce` and `foldl` both stay exactly
as they are.

## STOP triggers — ship nothing further, report the gap, stop

**STOP-1** — a rete gate refuses the vocabulary removal, or a completeness check you cannot satisfy
without changing what another verb is ruled to be. Report the payload verbatim.

**STOP-2** — a test cannot be rewritten without changing what it measures. Name it and quote its
assertion. Do not weaken one to clear the deletion.

**STOP-3** — anything outside the listed files needs to change. That is a consumer nobody counted;
report it rather than migrating it.

**STOP-4** — a scoped `nextest` fails outside the tests you rewrote.

## Your report

1. The three ledgers, each with what you removed and what (if anything) fired.
2. For each of the 4 rewritten tests: what it measures, and the new spelling.
3. Both capability headers, before and after.
4. Everything you ran; state plainly that you did not run the floor.
5. Honest deltas, line counts, wall-clock against a 45–70 minute prediction.

---

## ⛔ AMENDED 2026-08-18 (post-compaction, pre-strike) — THE CENSUS ABOVE MISSED THREE ROOMS

Re-measured against the disk before release. The 43 `src/` sites and the 5 `:wat::core::foldr`
call sites above are all **correct**. Three things they do not cover:

### A — `src/runtime.rs` holds TWO unit tests, and one of them IS this stone's acceptance row

Both sit inside the 7 `runtime.rs` hits counted as blast radius, and step 1 above says only
"delete it and its dispatch arm." Measured, they are not the same thing:

- **`:37866 foldr_is_right_associative`** — asserts **2** for `1-(2-(3-0))`. That is
  **EXPECTATIONS row 2, verbatim, already on disk.** ★ **REWRITE it** to
  `(reduce f init (reverse coll))` and keep the assertion at `2`. Do not delete it: the row that
  proves the capability survives the retirement is this test.
- **`:37882 foldl_vs_foldr_differ_on_nonassoc_op`** — measured: its body calls **only `foldl`**
  and asserts **−6**. The `foldr` half lives in the NAME and the comment, never in a call. So it
  is a **rename + comment fix**, not a rewrite — and it is the negative control that keeps
  discriminating a left fold from a right one, so it stays.
  `[[feedback_a_negative_control_that_can_be_kept_must_be_kept]]`

### B — the refusal does not name a replacement unless you add a retirement-table row

`src/remedy/retirement.rs` is the substrate's explicit memory of its own evolution — an exact
static table, `(retired, replacement, note)`, consulted by `remedies_for`. Deleting the dispatch
arm alone yields a generic unknown-form error, which **fails EXPECTATIONS row 1** ("the message
names the replacement"). **Add the row**, following the shape of the `:wat::core::vec` entry
(which carries a two-part note):

```rust
RetirementEntry { retired: ":wat::core::foldr", replacement: ":wat::core::reduce",
    note: Some("wat is STRICT, so a right fold is `(:wat::core::reduce f init (:wat::core::reverse coll))` — `foldr` was `reverse`+`foldl` wearing a name borrowed from Haskell, where the verb is distinct only because it is LAZY (arc 118.B6b)") },
```

The module's header carries an arc-history table; append the row there too — that is the
convention every prior entry followed.

### C — a `wat-scripts/` file calls the RETE spelling, and wat-scripts must LOAD

`wat-scripts/scratch-pad/probe-arc278-57-round1b-parametric-and-hof.wat:42–46` defines
`:probe-foldr` via **`:wat::rete::core::foldr`**. The census above grepped `:wat::core::foldr`
and never saw it. `wat-scripts/` is loader-gated (`every_wat_scripts_file_loads`), so deleting the
vocabulary row in step 2 takes this file red. **Delete the `:probe-foldr` def**, and record why
INLINE — that file already documents its own `PersistentVector/get` conversion in exactly that
style, four comment lines above the fold defs. The file's header says it probes "the five
Redispatch-class higher-order combinators"; it now probes four. Correct the header.

### The gates to expect, by name — do not discover these the hard way

```
src/rete/vocabulary.rs:1524  every_rete_row_is_total
src/rete/vocabulary.rs:1561  every_row_is_admitted
src/rete/vocabulary.rs:1626  every_rete_name_is_unique
src/rete/purity.rs:2168      completeness_gate::every_dispatched_verb_is_classified_or_disposed
```

### ★ A FINDING TO REPORT, NOT TO ACT ON — there is no `:wat::rete::core::reverse`

Measured: the rete vocabulary has `foldl`, `foldr`, `map`, `filter`, **`reduce`** — and **no
`reverse`**. So core's replacement composition `(reduce f init (reverse coll))` is spellable in
wat and **NOT spellable in a `where`**: removing `foldr`'s row leaves the rete sub-language with
no right fold at all.

Removing the row is still **forced** — it is a `Redispatch` alias whose `core_name` points at a
verb that will not exist, i.e. a dangling declaration. Whether to mint a `reverse` row alongside
is a **language-surface addition and the builder's ruling**. ⛔ **Do not mint it.** Report the
hole; the orchestrator carries it up.
