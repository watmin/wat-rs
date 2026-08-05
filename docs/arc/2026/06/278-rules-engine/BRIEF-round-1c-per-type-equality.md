# BRIEF — #57 round 1c: per-type equality, the closed set

Anchor at `/home/watmin/work/holon/wat-rs/`; verify with `pwd`; `git -C …` for git reads. Tree clean
at HEAD.

## The work in one paragraph

`=` and `not=` are the most common things a rule says (45 occurrences in the `where` corpus) and the
rete surface has neither. The surface is **per-type by ruling**, so this is ten `Alias` rows — `=`
and `not=` across the five types `ParamType` can spell (`i64`, `f64`, `String`, `bool`, `keyword`).
No core changes, no audit, no new mechanism: the same shape as round 1a, one round larger.

## ⛔ READ THIS FIRST — what this round is NOT

The stone calls 1c *"an audit — which types is unmeasured."* **That framing is retired.** It made the
corpus the instrument that decides the mint list, and the builder ruled that instrument out in R60:
a census of our own benchmark files is *"a record of what happened to compile"*, not evidence of what
a rule author will write — *"you have no fucking clue what our users are going to do."*

**The set is closed by the type system, not by the corpus:** every type the surface can name gets an
equality. That is the stone's own *"the closed set is a BASIS, not a ceiling."* The corpus audit
still matters later, for the **migration** worklist (which spellings must be rewritten before
arming), and that is a legitimate use of it — but it does not gate this mint and must not be waited on.

## Read in order

1. `docs/arc/2026/06/278-rules-engine/DESIGN-STONE-where-admits-only-rete-ops.md` — the
   `★★ RULED — THE RETE SURFACE IS PER-TYPE, PERIOD` section. Its argument is **totality**:
   *"Generic `>` is PARTIAL. Its domain hole is 'these two operands are not comparable.'
   Monomorphising … deletes the domain hole."*
2. `src/rete/vocabulary.rs` — `RETE_OPS` (35 rows), `ParamType` (5 variants after 1a/1b).
3. The nine rows round 1a added — your exemplar. Copy their shape exactly.
4. `src/rete/purity.rs:307-308` and `:511-512` — where generic `=`/`not=` are classified.

## The ten rows

All `class: OpClass::Alias`, all `params: &[X, X]`, all `ret: ParamType::Bool`,
all `type_params: &[]`.

| rete_name | core_name | params |
|---|---|---|
| `:wat::rete::i64::=` | `:wat::core::i64::=` | `[I64, I64]` |
| `:wat::rete::i64::not=` | `:wat::core::i64::not=` | `[I64, I64]` |
| `:wat::rete::f64::=` | `:wat::core::f64::=` | `[F64, F64]` |
| `:wat::rete::f64::not=` | `:wat::core::f64::not=` | `[F64, F64]` |
| `:wat::rete::String::=` | `:wat::core::=` | `[String, String]` |
| `:wat::rete::String::not=` | `:wat::core::not=` | `[String, String]` |
| `:wat::rete::bool::=` | `:wat::core::=` | `[Bool, Bool]` |
| `:wat::rete::bool::not=` | `:wat::core::not=` | `[Bool, Bool]` |
| `:wat::rete::keyword::=` | `:wat::core::=` | `[Keyword, Keyword]` |
| `:wat::rete::keyword::not=` | `:wat::core::not=` | `[Keyword, Keyword]` |

**Why `String`/`bool`/`keyword` point at the GENERIC core op and that is not a cheat.** Core has no
`String::=`. It does not need one, because **totality here is delivered by the SIGNATURE, not the
implementation**: a row declaring `[String, String] -> Bool` makes an incomparable pair a *type error
before anything runs*, which is exactly the domain hole the per-type ruling exists to delete. The
routine underneath can safely be the shared generic kernel — the stone's own implementation law,
*shared kernel, two surfaces*. Minting six new public core verbs to answer a rete-internal question
would be the tail wagging the dog.

`i64` and `f64` point at their real per-type core ops because those exist; per-type is preferred
where it is available.

## `meta` — transcribe, do not decide

All ten: `OpMeta { pure: true, deterministic: true, total: true }`, transcribed from generic
`=`/`not=` (`purity.rs:307-308` pure∧det, `:511-512` total).

**⚠ One thing you will notice and must NOT act on:** `:wat::core::i64::=` and `:wat::core::f64::=`
appear **nowhere** in `purity.rs` — the per-type equalities that exist in core are unclassified by
the fence. That is a real gap and it is **not this round's**. It does not affect these rows: a rete
row carries its **own** `meta`, which is what the fence consults for the rete spelling. Report it in
your findings; do not add classifications to `purity.rs`.

## ⛔ STOPs — rejection criteria

- **⛔ STOP-1 — mint EXACTLY these ten.** No `<`/`>`/`<=`/`>=` (already present per-type for i64,
  and the generic comparison family is round 2's business). No container equality.
- **⛔ STOP-2 — do NOT add anything to `src/rete/purity.rs`.** Including the unclassified
  `i64::=`/`f64::=` you will notice. Report, do not fix.
- **⛔ STOP-3 — do NOT mint any new `:wat::core::` verb.** If you conclude a core `String::=` is
  needed, STOP and report — that conclusion contradicts this brief's reasoning and the orchestrator
  owns the re-scope.
- **⛔ STOP-4 — do NOT arm anything.** Core spellings keep working identically.
- **⛔ Do not add a `_` wildcard arm on an enum scrutinee.**
- **⛔ Do not commit, stash, push, or touch git.**

## Verify — FOREGROUND, block, and run the suite SOLO

```
cargo build --release
cargo nextest run --release          # no other cargo process alive
cargo clippy --release --all-targets
./wat-scripts/perf/grid/check-where-shapes.sh
```

Floor **`4348 / 4348 / 0 / 262`**; gate **9 pairs / 98 rows**. Vocabulary only — no derived fact,
no test, may move.

---

## EXPECTATIONS — written before the strike

| # | what | expected |
|---|---|---|
| 1 | row count | **45** (35 + 10) |
| 2 | ★ a rete equality runs | `(:wat::rete::String::= "a" "a")` → `true`; `"a" "b"` → `false` |
| 3 | ★ **the domain hole is deleted by the signature** | `(:wat::rete::String::= "a" 1)` is a **type error at `--check`**, not a runtime surprise — this is the whole justification for per-type, so it is the load-bearing row |
| 4 | ★ **non-vacuous** | a bogus `:wat::rete::String::=X` raises a located `UnknownFunction` **at runtime** (`--check` does not validate `:wat::*` heads — measured 2026-08-05) |
| 5 | ★ f64 equality works and is not special-cased | `(:wat::rete::f64::= 1.5 1.5)` → `true`. It belongs; the doubt was apparatus-raised and is retracted |
| 6 | i64/f64 route to their per-type core ops | read the rows |
| 7 | ★ nothing armed | core `=` still works everywhere; where-corpus unchanged |
| 8 | ★ floor | `4348 / 4348 / 0 / 262` exactly |
| 9 | ★ gate | `9 pair(s), 98 rows — wat == Clara on every shape` |
| 10 | clippy | clean |
| 11 | `purity.rs` untouched | `git diff --stat -- src/rete/purity.rs` → empty |

Rows 2, 3, 4, 5, 8, 9 re-run by the orchestrator by hand.

**Runtime prediction: 20–35 minutes.** Smaller than 1b — no new enum variants, no new class, no
checker routing. Ten rows in one table. Time-box 70.

**Trap doors:**
1. **Minting a core `String::=`** because the row "should" point at a per-type op. The signature does
   the totality work; the kernel is shared. STOP-3.
2. **"Fixing" the unclassified `i64::=`/`f64::=` in `purity.rs`** while you are in there. STOP-2.
3. **Trusting `--check` for row 4** — it does not validate `:wat::*` heads at all.
4. **Reaching for the corpus** to decide which types to mint. That instrument is ruled out for this
   question; the set is closed by the type system.
