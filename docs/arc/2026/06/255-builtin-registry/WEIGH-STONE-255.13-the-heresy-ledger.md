# WEIGH — STONE 255.13: ACCEPTED. **The ledger reads 232** — and my brief pointed at the wrong file.

Weighed against `c600c039f` by independent re-run. `src/` **byte-identical to HEAD**; 0 `.wat`
converted; the diff is one lint file, a dev-dep, and the SCORE. ⭐ **Nothing was cured — as instructed.**

## ⭐⭐ THE RATCHET WORKS — I PROVED IT MYSELF, IN A DIFFERENT FILE

I appended a synthetic dual-read to `src/closure_extract.rs` (the rider used `form_match.rs`):

```
+ src/closure_extract.rs  fn weigh_255_13_synthetic_heretic  1 site(s) [Bx1]
```

⭐ **It caught it AND classified it correctly as shape B** — dual-read then keyword-keyed, which is
exactly what I wrote. Reverted; tree clean; all 4 tests green again.

`LEDGER_TOTAL = 232` is frozen and ⭐ **fails in BOTH directions**, with a distinct message for
shrinkage telling the next hand to re-freeze. **That is a ratchet, not a number in a doc.**

## ⛔⛔ THE BRIEF WAS WRONG ABOUT THE ADDRESS — AND I VERIFIED THE CORRECTION

My brief convicted `src/collection/transform.rs:304-340`. ⭐ **Measured here:**

```
awk 'NR>=290 && NR<=360' src/collection/transform.rs | grep -c '":wat::'   →   0
```

⛔ **There is NO keyword comparison in that range at all.** It is `sort$native`'s **call** of the
purity gate; every `":wat::` in the file is an `OP` label, an attribute, or message text. **A
"keyword comparison" lint could not flag it at any quality, and I sent the rider there.**

⭐⭐ **The lint found the real site unaided, by DATA FLOW:** `rete/purity.rs::classify_expr` reads
`Keyword(k) => k.as_str()` / `Symbol(id) => id.as_str()` with **no door**, and carries that
provenance into `intrinsic_meta` (`matches!(head, ":wat::core::+" | …)`) and `effectful_by_prefix`
(`head.starts_with(":wat::kernel::")`), both keyword-keyed. **That is `wat.core/<`.**
⭐ **12 of the crate's 14 shape-B sites are in that one file — ONE door retires all 12.**

**8d-ii's fifth draw edits `src/rete/purity.rs`.** I had the defect right and the address wrong.

## ⭐⭐ THE DECOMPOSITION CHANGES THE STRATEGY — read this before the terminal cut

| shape | n | what it is |
|---|---|---|
| **E — type-path raw** | **125** | a `TypeExpr` compared without the denotation door (255.12's class) |
| **A — keyword-only** | **93** | a symbol-spelled name is invisible to it (255.9's class) |
| **B — dual-raw** | **14** | ⛔ *sees* the symbol and **mis-keys** it — **strictly worse than A** |
| C — symbol-raw | 0 | none |

⛔⛔ **THE MIGRATION IS ~54% A TYPE-PATH QUESTION, NOT A CALL-HEAD ONE.** The ruling is *"all call
heads are symbols"* — but **converting every head does not take this number to zero.** 125 of 232
are types, **78 of them in `check.rs` alone**, behind one door (`denoted_type_path` /
`parametric_heads_unify`). ⭐ **Shape E deserves its own campaign, and the terminal cut needs A+E at
zero, not just heads.**

⭐ **And the dangerous class is tiny and concentrated:** 14 shape-B, 12 in one file, already the next
stone's target. **Split the countdown — B is a DEFECT LIST, A+E is a MIGRATION SCHEDULE.**

## Calibration — a TEST, keyed per decision, not per function

All 4 lint tests pass here. ⭐ **Keyed on `(file, fn, literal)`, because `is_type_equatable` and
`validate_pure_total` each hold a cured decision AND a raw one** — a per-function key would have
scored both as clean. **13 cured rows absent; 4 open rows present**, and the test asserts
`intrinsic_meta`'s shape mix **contains `B`** rather than mere presence — ⭐ **presence alone would
prove nothing about the `wat.core/<` mechanism.**

The discriminator is `syn`, a real parser — ⭐ *"the alternative was another regex, and this arc has
paid for that twice."* The door set is **derived by fixpoint from two seeds**, never a hand-list,
with a census pinning six doors by name **and pinning that `identity_text` is NOT admitted.**

## Gates re-derived

| gate | mine |
|---|---|
| lint suite | **4/4 pass**; ratchet **verified by my own synthetic heretic** |
| floor | **3 failed → 1 failed → ⭐ 5 993/5 993 GREEN**, all three captured |
| clippy | ⚠ first run was **cached (0.07 s)** — I forced a real compile: **6.70 s, rc 0** |
| census / delta | `no STOP-8`, 213 = 213 · delta **3**, **RECOVERY 0** |
| `src/` changed | **0 bytes** |

⭐ **The three floor reds were all the rider's own, all captured, none re-run to clear** — its new
lint file tripped three *existing* lints and it cured them with the per-site runes those lints
prescribe, ⭐ **weakening none.** `harvest_wrap_split` did not fire, and it said so **because the
brief asked it to be named either way.**

⚠ **Two undisclosed partial floor dirs** (`20-43-12Z`, `20-43-39Z`) hold only fragments — aborted
runs the SCORE does not mention. Harmless, but unremarked.
⚠ IDE diagnostics again flagged dead code at line numbers that do not match the file. **Stale**, like
255.12's. Verified by fresh compile, not by trusting either.

## Three findings no stone had named — reported, not cured

1. It **independently rediscovered 255.12's own declared blind spot** (`subsume.rs:123`), which that
   SCORE called *"a reading, not a probe."* ⭐ **The instrument found by measurement what a stone had
   only suspected.**
2. `check.rs::is_type_equatable` **cured its `Path` table and left two residues beside it** —
   `p == ":wat::core::Value"` and a `Parametric` arm matching raw, **skipping `parametric_heads_unify`,
   the door 255.12 itself built.** Same in `is_type_orderable`.
3. ⛔ **`rete/kernel/arm.rs` has two shape-B sites that 255.9 dispositioned as *"already both — fine"*.**
   ⭐ **Reading both spellings is HALF the cure; the door is the other half.** A correction to a
   landed stone's disposition.

## ⚠ The caveat that will bite someone

⭐ **If a future hand widens the analyzer and resolves some of the 68 `D-unresolved`, the ledger can
RISE without `src/` changing** — and the ratchet will read it as a regression. **Re-freeze in the
same commit, and say that the instrument moved, not the tree.**

## VERDICT

**ACCEPTED.** Pushing. **Twenty-two corrections across twenty stones.**

**Next:** 8d-ii's fifth draw → `src/rete/purity.rs::classify_expr`'s head read (one door, 12 of 14
shape-B), then re-measure the 416. ⭐ **The ledger, not a 40-minute conversion cycle, is now how the
next target gets found.**
⛔ **Builder's call, unchanged:** `is_quasiquote_form` · `Ngram` · variant tags in declarations.
⭐ **And the terminal cut now has a number: it needs A+E (218) at zero — not just heads.**
