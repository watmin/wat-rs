# BRIEF — STONE 251.8d-ii (SIXTH DRAW): the acc-fold head

**Drawn 2026-09-22 against `main` @ `3068ea139`.** Landable floor 6002/6002, clippy 0, census
`no STOP-8`, delta **3 / RECOVERY 0**, ⭐ **heresy ledger 229** (shape **B = 3**).
Converted floor: **283**. Trajectory **3 747 → 416 → 283**.

## ⭐ THE ADDRESS — READ BY THE ORCHESTRATOR THIS TIME

⛔ **The two previous draws both convicted the wrong address** (a file with no keyword comparison;
then a dead `_ =>` arm behind a Keyword-only match guard). **This one was read, not inferred:**

`src/rete/kernel/arm.rs::compile_acc_fold`

```rust
// ~:250 — reads BOTH payloads RAW: shape B
let head = match items.first() {
    Some(WatAST::Keyword(k, _)) => k.as_str(),
    Some(WatAST::Symbol(s, _))  => s.as_str(),      // ⛔ no door
    …
// :282 — then keys on the KEYWORD spelling
Ok(match head {
    ":wat::rete::acc::count" => AccFold::Count,
    ":wat::rete::acc::sum"   => AccFold::Sum(var_key()?),
    …8 arms…
    _ => { /* "user acc fold" — demands a compiled Program */ }
})
```

⚠ **A SECOND dual-raw read is ~15 lines above, inside the `var_key` closure** (`Symbol(s) =>
s.as_str().to_string()` / `Keyword(k) => …`). ⭐ **255.13's ledger named BOTH; they are the two
remaining shape-B sites in the crate outside `purity.rs`.**

## The evidence — measured by the orchestrator from the fifth draw's converted log

| | |
|---|---|
| `acc::count: expected 1 argument(s); got 0` | **98** |
| `acc::sum: expected 2; got 1` | **66** |
| `acc::max: expected 2; got 1` | **41** |
| `acc::min: expected 2; got 1` | **29** |
| `acc::all: expected 1; got 0` | **5** |
| ⭐ **uniform off-by-one across all five verbs** | **239 total** |
| `:wat::rete::i64::+: parameter #1 expects i64; got :wat::core::keyword` | **29** — ⚠ **a LEAD, not attributed**: consistent with args shifting once one is dropped, **not proven** |

⭐ **The source is a ZERO-ARG call in BOTH spellings** — `(:wat::rete::acc::count)` →
`(wat.rete.acc/count)`. **The missing argument is INJECTED by the builtin arm**, and the converted
head misses that arm, falls to `_`, and is compiled as a **user fold**.

## ⛔⛔ PERMISSIVE DIRECTION — AGAIN. THIS IS THE THIRD IN A ROW.

Teaching this match a second spelling makes a head that was treated as a **user fold** become a
**builtin**. ⭐ **255.12 and the fifth draw are both precedent: 255.12's adversarial row caught its
own first draft.**

⭐ **NON-VACUITY, MANDATORY, IN ONE TEST:**
1. `(wat.rete.acc/count)` compiles to `AccFold::Count`, same as the keyword spelling.
2. ⛔ **A GENUINE USER FOLD STILL COMPILES AS A USER FOLD** — it must not be swallowed by a builtin arm.
3. ⛔ **AN UNKNOWN head with no compiled Program is STILL REFUSED**, with the same located reason.
4. ⭐ `var_key`'s site too: a `?var` argument still resolves in both spellings.

## ⛔⛔ CURING THIS BREAKS THE LEDGER'S OWN CALIBRATION — RE-ANCHOR IT, DO NOT WEAKEN IT

255.13's shape-B calibration was **re-anchored onto `arm.rs::compile_acc_fold`** when the previous
anchor was cured. ⭐ **Curing it will red `the_discriminator_separates_the_cured_from_the_open`.**

⛔ **That test failing is CORRECT and is NOT permission to delete the row.** Move the anchor to a
**still-open shape-B site** (the ledger will name what remains), exactly as the fifth draw did, and
⭐ **add `compile_acc_fold` to the CURED side** so the calibration keeps proving both directions.
⛔ **A calibration with an empty open-side proves nothing.**

## The work

1. **One door at the head read**, through `edn::render::canonical_identity` — ⭐ **Symbol arm only;
   leave the Keyword arm byte-identical** (255.13's DUAL-ARM RULE, as the fifth draw did).
2. **The `var_key` site.** ⚠ **Measure whether it needs the same cure or is already reached
   canonically — do not assume.**
3. The four non-vacuity rows.
4. **Re-anchor the ledger calibration** and re-freeze `LEDGER_TOTAL`.
5. ⭐ **Re-measure the conversion:** convert live `wat/` (64 explicit paths), build, floor.

## The gate

- ⭐ **Ledger drops and is RE-FROZEN in the same commit.** Shape **B 3 → fewer**; ⛔ **report the
  shape mix, not just the count** — the fifth draw proved a count-only ratchet can be walked past
  sideways (`Bx8 → Ax8` was invisible until it was fixed).
- ⭐ **Converted floor: 283 → ?** ⛔ **Report it even if unchanged.** ⚠⚠ **DO NOT PROMISE 283 − 239.**
  The 283 are **classified, not diagnosed**; the third draw's *"323 tests bought by two sites"* and
  the fifth draw's own warning both say the classes interact. ⭐ **A `comm` against the fifth draw's
  failing set (FIXED / NEW) is worth more than the total** — the fifth draw's was a strict subset.
- The four non-vacuity rows, in one test.
- `scripts/floor.sh` green on the **landable** state; clippy 0 (⚠ **not cached — touch first or check
  elapsed time**); census `no STOP-8`; `scripts/replay/delta.sh` — baseline **3**, ⛔ **RECOVERY
  non-zero is a STOP**.
- ⭐ **LAND THE CONVERSION ONLY IF THE CONVERTED FLOOR IS GREEN.** Otherwise `git checkout -- wat/`,
  rebuild, report. **Four draws have stopped correctly; all four were accepted.**

## ⛔ Operational traps — all have cost this arc time

- ⛔⛔ **NEVER `git add -A`, NEVER invoke cargo, while a conversion is writing `wat/`.**
- ⛔ `git ls-files 'wat/**/*.wat'` → 31 of 64. Use `git ls-files | grep -E '^wat/.*\.wat$'`.
- ⛔ The stdlib is `include_str!`'d — **rebuild between steps and say you did.**
- Conversion ~22 min; Bash caps at 600 s — **background it.**
- ⚠ **A fast floor is a symptom** (24 s = stdlib did not load; ~330 s healthy; ~450 s converted-but-red).
- ⚠ `cargo clippy` can report rc 0 **from cache in 0.07 s**.
- ⚠ **`ARM.txt` REPEATS each failure** (1 248 lines for 416). ⭐ **For FIXED/NEW use `clean.log`, and
  strip the whole `FAIL […] (n/m)` prefix — a padded index `( 167/6002)` broke the orchestrator's own
  regex and manufactured 138 phantom regressions.**

## Doctrine

- ⛔ **THERE IS NO KNOWN FLAKE. A RED IS A RED.** ⚠ One bookkeeping exception: `harvest_wrap_split`
  (`278-rules-engine/FINDING-harvest-cost-is-not-a-gate.md`). **Cite and report; never a pass, never
  re-run to clear. Name it either way.**
- ⛔ **REPORT WHAT A DOOR CANNOT EXPRESS.** Seventeen stones running.
- ⛔ **If this brief contradicts the code, THE CODE WINS.** ⭐ **Twenty-three corrections across
  twenty-one stones.** **Assume a twenty-fourth.**
- Work only in `/home/john/work/holon/wat-rs`. **Do not push.**

## Out of scope

Shape **E** — ⛔ **125 sites, 78 in `check.rs`, UNTOUCHED, and the terminal cut needs A+E at zero,
not just heads** · `spawn::thread/init`'s leaf-join disagreement · `classify_expr`'s keyword-only
quote/data guard (**strict, not loose — builder's call**) · `is_quasiquote_form` · `Ngram` ·
variant tags in declarations · 8d-iii.
