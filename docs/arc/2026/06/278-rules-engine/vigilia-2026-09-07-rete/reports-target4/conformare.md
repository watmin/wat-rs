# CONFORMARE — Cast Report (TARGET 4 — the test corpus) — **FINDINGS**

> Returned verbatim, before synthesis. Orchestrator verification follows the ward's text.

## Zero re-derived first (per trap 8)

```
grep -rhnE 'enum [A-Za-z]*Error' tests/rete/ src/rete/kernel/tests/ --include='*.rs' | wc -l   → 0
find tests/rete -name '*.wat' | wc -l        → 144
find tests/rete -name '*.rs' | wc -l         → 100
find tests/rete -name '*.edn' | wc -l        → 23
find tests/rete -name '*.wat.bad' | wc -l    → 19
find src/rete/kernel/tests -name '*.rs' | wc -l → 20
```
Matches the brief's manifest exactly. Confirmed: **no `enum *Error` in this target.** Taking the translated surface.

## FINDING (in-target): the 460:10 convention has no structural backing

`call_beside_value(...) -> Result<Value, RuntimeError>` (`src/freeze.rs:1163`) is called from ~470 sites across the corpus. The overwhelming convention is `.expect("<context>")`:

```
grep -rn "call_beside_value(.*)\.expect(" tests/ --include='*.rs' | wc -l   → 460
grep -rn "call_beside_value(.*)\.unwrap()" tests/ --include='*.rs' | wc -l  → 10
```

All 10 bare `.unwrap()` calls sit in exactly one file, in-target: `tests/rete/probe_arc278_8i_accumulator_folds.rs:26,32,38,44,50,56,63,70,77,83`. Every sibling test in the same corpus (e.g. `tests/rete/probe_arc278_5b_collect_rules.rs:16,22,28,34,36`) uses `.expect("eval")`-style context on the identical call. Nothing in the type signature of `call_beside_value` or in `RuntimeError` distinguishes these ten call sites from the 460 — the convention is pure hand-discipline: a new test author copying this one file, rather than any of the other 470 sites, silently drops the diagnostic-context string and the type system raises no objection.

- **Level:** L1 — cosmetic under this specific type (Rust panic output includes the source `file:line` of the `.unwrap()` call itself via `#[track_caller]`, so the 3am reader still lands on the exact assertion; only the human-authored "what was I testing" phrase is lost, not the location). Cost of the gap is real but small.
- **Shape failure axis:** Honest = NO (a caller CAN produce this failure path without supplying context) and Good UX = practitioner-enforced, not structural. Obvious/Simple are fine (there's exactly one path, `.expect`/`.unwrap`, both trivial to read).
- **Pattern verdict:** this is a call-site convention, not a type definition, so Patterns A/B/C (which retrofit an error *type*) don't apply. The structural fix a test corpus can actually take is a *lint*, mirroring `no_loose_string_assert.rs`: a `tests/lint/*.rs` walk that fails on a bare `.unwrap()` following `call_beside_value` (or more generally on any `Result`-returning call inside `tests/rete/` and `src/rete/kernel/tests/`) without a per-site rune. That converts "make the wrong shape uncompilable" into "make the wrong shape un-green," which is the achievable structural analogue in a test corpus (no enum to retrofit).
- **Cascade scope:** 10 sites, 1 file, 0 downstream consumers (these are leaf `#[test]` fns) — smallest possible retrofit.

Not adjacent to prior art: excusare weighed *runes*, not naked failure points with no rune at all; solvere/struere/sequi rows are about duplicated logic/positional args, not about this file's messaging omission specifically.

## The handed-down question — settled by reading, not running

**I re-implemented `tests_carry_no_loose_string_assert`'s exact algorithm** (same three roots, same `has_loose_string_match`/`is_assert_opener` string logic, same statement-scoped reset-on-`;{}`-skip-comment-lines state machine) in a read-only Python script (no cargo, no writes) and ran it against the live tree:

```
total .rs files under src/,tests/,crates/:  1028   (non-vacuity: > 500 holds; brief's "998" is 30 files stale, consistent with active development)
violations: 0
```

I then independently found the raw candidates with a separate grep (methodology different from the port, to cross-check it), restricted to this target:

```
17 same-line `assert!(...contains/starts_with/ends_with("<literal>")...)` hits in tests/rete/
```

I read all 17 in context (`sed -n` around each). **Every one of the 17 carries a `// rune:lint(loose-assert) — <reason>` on the line immediately above it**, inside the same statement (no intervening `;`/`{`/`}`), e.g.:
- `tests/rete/probe_arc278_then_operand_wall.rs:48-50` — rune present, reason: "the error embeds an absolute path (Span :file); an exact golden cannot be deterministic. Asserts the error KIND."
- `tests/rete/probe_constructor_meta_surface_audit.rs:112-113,114-115,116-117,118-119,146-147,148-149,150-151,152-153` — eight consecutive runed asserts, same reason class.

I also confirmed the rune count independently: `grep -rc "rune:lint(loose-assert)"` over the target sums to exactly **52**, matching the brief's figure precisely — my instrument and the brief's agree on this anchor.

**Verdict: (a) — the campaign was driven to zero; the gate is GREEN today; the "expected-red test" prose in `tests/lint/no_loose_string_assert.rs:14-15` is stale.** My reproduction found 0 violations repo-wide (1028 files), not just in-target, and the in-target 17 raw hits are fully accounted for by adjacent runes. This is consistent with — not proof against — the recorded `5480/5480` floor claim; nothing here suggests (b).

**But this IS a live doctrine hazard, independent of which of (a)/(b) is true, and it is the sharper part of the answer:** the header text (`:11-15`) is written as a standing, self-renewing exception — *"Until then this is an expected-red test... a SECOND red is a real regression"* — with no expiry condition tied to the violation count actually being checked. Since violations are now 0, that clause is inert prose describing a past campaign state, not a live exception — but it still *reads* as one to the next person who opens the file, and its very phrasing ("a SECOND red is a real regression") is structurally the same shape as the four struck licences in `wat-rs/CLAUDE.md` ("known flake," "first one doesn't count," a two-strikes framing) that the builder explicitly annihilated on 2026-08-05. If a future single new loose-assert violation lands and someone reads this header before checking `git blame`/history, the header invites exactly the dismissal the repo's own doctrine forbids by name. This is not the same finding as excusare's 4X1 (an unreasoned suppression) — it's stale doctrinal framing left standing in a gate's own header, in a file the ward explicitly sanctioned reading for context.

**What would make this certain rather than "strongly evidenced":** running `scripts/floor.sh` (forbidden this cast) and reading the `tests_carry_no_loose_string_assert` line in the Summary. I did not run it and neither should this report be read as claiming I did — the 0-violations result is from a faithful re-implementation I built and ran myself this session, cross-checked against two independent methods (full-port simulation + manual raw-grep + rune reading) and one hard anchor (the 52 count matching exactly), not from executing the actual gate.

## What I checked and found sound (verified negatives)

- `unwrap_err()` — 1 hit, `tests/rete/probe_arc278_then_user_forms.rs:116`, is legitimate (testing an expected-error path, not a diagnostic-completeness gap).
- The `[[test]] name = "lint" path = "tests/lint/mod.rs"` entry (`Cargo.toml:226-228`) is a normal harnessed test — no `harness = false`, no pre-exemption at the Cargo level that would make "expected-red" true by construction.
- File-count instrument was cross-checked twice (python os.walk port vs `find`) and agrees exactly with the brief's 144/100/23/19/20 breakdown for the target.

## Ranking

1. **Sharpest, in-target:** the 460:10 `.expect`/`.unwrap` asymmetry, entirely inside `probe_arc278_8i_accumulator_folds.rs` — smallest possible fix (10 sites, 1 file), directly answers conformare's literal question translated onto this corpus, and is not covered by any prior-art row.
2. **Sharpest, handed-down question:** the stale "expected-red" / two-strikes framing in `tests/lint/no_loose_string_assert.rs:11-15` — the gate itself is measured GREEN today, so the framing is inert but hazardous, and it is the one item in this cast that directly collides with `wat-rs/CLAUDE.md`'s named doctrine.

## Corrections made mid-cast

None to the instrument logic — but I built and cross-validated the loose-assert re-implementation two independent ways (full port + separate raw grep) before trusting the 0-violations result, given trap 8's explicit warning about zeros.

FINDINGS

---

# ⚖ ORCHESTRATOR VERIFICATION — my own read of the disk, this session

## ✅ EVERY NUMBER VERIFIES EXACTLY — 460, 10, all ten addresses, 52, 1028, and the header verbatim

```
grep -rn "call_beside_value(.*)\.expect(" tests/ --include='*.rs' | wc -l   → 460
grep -rn "call_beside_value(.*)\.unwrap()"  tests/ --include='*.rs' | wc -l →  10
```
All ten at `tests/rete/probe_arc278_8i_accumulator_folds.rs:26,32,38,44,50,56,63,70,77,83` — **every address checked individually, all ten correct.** `rune:lint(loose-assert)` in-target sums to **52**. `find src tests crates -name '*.rs'` → **1028**. The header at `:11-15` reads verbatim as quoted, ending *"Until then this is an expected-red test; nextest isolates it, so a SECOND red is a real regression."*

## ⛔⛔ I TRIED TO FALSIFY ITS "ALL 17 ARE RUNED" CLAIM AND **MY INSTRUMENT WAS THE THING THAT BROKE** — TWENTIETH ERROR, AND IT IS THE CORPUS PROPERTY THIS CAST HAS NOW PROMOTED THREE TIMES

I ran a one-line-lookback check over the 17 raw hits and got **6 apparently BARE** sites — which would have refuted the ward. I read all six before reporting. **Every one carries a MULTI-LINE rune comment**, the `rune:lint(loose-assert)` sitting 2–4 lines above the `assert!` with continuation comment lines between:

```
tests/rete/probe_arc278_then_user_forms.rs:117-120
    // rune:lint(loose-assert) — the diagnostic embeds an absolute file path (Span), which is
    // non-deterministic across machines/CI; assert the load-bearing SUBSTANCE (which type was
    // rejected), not the whole rendered blob.
    assert!(msg.contains("wat::core::i64"), …);
```

**The ward is right; all 17 are runed.** ⭐⭐ **And this is the SAME corpus property that broke `purgare`'s orphan scan and `intueri`'s name-extractor** — a comment sitting between a marker and the item it marks. Those two were `// rune:vocare(…)` between `#[test]` and `fn`; mine was a multi-line rune between the marker and the `assert!`. ⛔ **Three independent scanners, three casts, one property.** It is no longer a coincidence to note — it is a rule for this corpus: **any scanner that pairs a marker with its item here must tolerate an arbitrary run of comment lines between them.** The real gate does exactly that (its state machine skips comment lines); my throwaway did not, which is why the real gate is green and my check was not.

## ⭐ AND ITS "998 IS STALE" CORRECTION IS RIGHT — WITH A SHARPER EDGE THE WARD DID NOT TAKE

`no_loose_string_assert.rs:82-87` reads:

> *"The floor sits well under the **998** `.rs` file(s) this walk finds today — driven 2026-09-01, and the count comes from `every_walking_gate_declares_non_vacuity.rs`, **never from prose** — so it catches a walk gone blind … **without rotting as the tree grows**."*

The guard itself is `assert!(files.len() > 500, …)` — a **threshold**, so the ward is right that the mechanism does not rot. ⛔ **But the number written beside it did: 998 → 1028 in six days.** The comment claims non-rotting *and states a figure that has rotted*, in the same sentence. **The mechanism is sound and its own illustration is stale** — `[[a-right-number-vouches-for-a-wrong-label]]`, and a near-twin of `4R1`'s shape. Folded into `4F2` rather than rowed separately, because both defects are the same header's prose outliving its subject.

## ⚠ ON THE `#[track_caller]` REASONING BEHIND THE L1 GRADE

The ward graded `4F1` **L1** while arguing the cost is small, because Rust's panic output carries the `.unwrap()`'s own `file:line`. That reasoning is sound and I keep the grade — but note what it concedes: **the ward is grading the DEFECT L1 while arguing the CONSEQUENCE is minor.** Both are defensible and they are not the same axis; a later reader should not take "L1" here as equivalent in weight to `4Q2` or `4P1`.
