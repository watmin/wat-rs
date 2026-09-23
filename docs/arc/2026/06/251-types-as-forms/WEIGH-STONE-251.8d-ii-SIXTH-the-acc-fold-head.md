# WEIGH — 251.8d-ii (SIXTH): cure ACCEPTED, conversion STOP. ⭐ **283 → 161.**
# ⛔⛔ And my THIRD consecutive wrong attribution — the pattern is named below.

Weighed against `dae454d6a` by independent re-run. Tree clean, `wat/` byte-identical to baseline,
nothing pushed. Trajectory: **3 747 → 416 → 283 → 161.**

## ⛔⛔⛔ THE PATTERN IS MINE AND IT IS NOW THREE FOR THREE

| draw | what I convicted | what was true |
|---|---|---|
| 4th | `collection/transform.rs:304-340` | ⛔ **the range contains NO keyword comparison at all** |
| 5th | `purity.rs:1232`'s `_ =>` arm | ⛔ **DEAD CODE — behind a `Keyword`-only match guard** |
| 6th | `compile_acc_fold` → the 239 arity failures | ⛔ **right SHAPE, wrong PASS** |

⭐ **The rider drew the distinction exactly:** *"You verified the function SHAPE (a real dual-raw head
read, quoted correctly) and inferred the CAUSAL LINK from the error text naming `acc::count`."*

⛔⛔ **And this one was refutable in ten seconds.** Verified here:

```
compile_acc_fold(…) -> Result<AccFold, EvalBreak>      ← a FIRE-time error type
the 239 failures are  #wat.check/MalformedForm         ← a FREEZE-time namespace
```

**A function returning `EvalBreak` cannot raise `#wat.check/*`.** I never checked the error namespace
against the function's return type.

⭐⭐ **THE CURE FOR MY PATTERN, AND THE RIDER DEMONSTRATED IT: CONVERT ONE FILE.** It found the real
address by converting **`wat/rete/syntax.wat` alone (~2 s)** and rebuilding — an empirical isolation
costing ~25 s, against my inference from a log. ⛔ **I have a 22-minute whole-stdlib cycle and a
2-second single-file one, and I have been reaching for neither — I have been reasoning.**
**Every future brief names the site by SINGLE-FILE CONVERSION, or names it as a HYPOTHESIS.**

## The real address, and it is a better finding than mine

`defrule`'s template emits `(wat.core/quote ~when-vec)` once converted — a **Symbol-headed quote**.
All four `make-rule` descents tested it with a keyword-only `matches!`, so the `:when` argument was
neither recognised nor re-spelled, and ⭐ **the quoted condition vector was type-checked as CODE.**
A zero-arg acc-form is an arity error the instant it is read as a call. **One language fact, six
sites, every Keyword arm byte-identical.**

⭐ `var_key` needed **no** cure — *measured, not assumed*: `canonical_identity` is the identity
function on `?v`.

## Re-derived here

| | mine |
|---|---|
| converted floor | **161 / 6008** |
| landable floor | ⭐ **6008 / 6008 GREEN** |
| ⭐ **FIXED / NEW** vs the fifth draw | ⭐ **127 / 5** — exactly as reported |
| the class the brief was drawn for | ⭐⭐ **96 → 0** in `clean.log` — **GONE, not reduced** |
| ledger | **229 → 223**, 4/4 pass, ⭐ **calibration RE-ANCHORED** to `walk_rete_defn_callees`, not deleted |

⚠ **My own `ARM.txt` counts in the brief (98/66/41/29) were inflated** — `ARM.txt` repeats each
failure. The rider counted `clean.log` and got 75/48/41/29. **Same trap I put in its own brief as a
warning, and I had already fallen into it when writing the numbers.**

## ⭐⭐ THE 5 NEW ARE THE BEST JUDGEMENT CALL IN THE STONE

Not a strict subset, and **it did not classify-and-move-on.** Two reverts on the converted tree, each
rebuilt, isolate all five to **one** of the six cured sites: `expand_make_rule_then`'s guard.

⭐ **Mechanism: that cure RE-ARMS a `:then` macro expansion the conversion had silently switched
OFF.** The five tests were passing because a wall was disarmed.

⭐ **It kept the cure**, on two grounds I endorse: *a disarmed wall is worse than a red*, and shipping
`expand_make_rule_when` cured **with its identical twin uncured** is precisely the drift
`boundary.rs` exists to prevent. ⛔ **The 161 therefore contains 5 that are arguably failing
CORRECTLY** — and the landable floor is green, so nothing user-visible rides on it.

## The adversarial tier — 13 rows, two binaries, and the passes are the point

⭐ **PRE-CURE the unit tier reads `5 tests run: 2 passed, 3 failed`, and the TWO PASSES ARE THE
ADVERSARIAL WALL ROWS** — a genuine user fold is still a user fold (including `wat.rete.acc2/count`
and `wat.rete.acc/median`, ⭐ **near-misses that canonicalise TOWARD the builtin namespace but match
no arm**), and the malformed-shape diagnoses are unchanged. The unknown-head row asserts the **wall**
before the identity, with a positive control so it cannot pass on a door that refuses everything.

⭐ **And when one cure had no witness in either tier, it went and found one where it lives:** a census
over all 2 211 tracked `.wat` on the converted stdlib, twice, one line apart — **246 → 240 failing,
six files rc 1 → rc 0, none the other way.** ⭐ *"That is why it ships."*

## The floor red — its own, captured, and exquisitely ironic

2 failed at **one line**: `one_variant_separator` and `one_name_grammar` both fired on a
`head.rsplit("::").next().unwrap()` it had written **in a test helper.** ⭐ *"This stone is about a
name read two ways and my own fixture read one two ways."* Cured through
`wat_reader::identifier::leaf` with the rune the gate prescribes. **Not re-run to clear.**

## Disclosed

`compile_acc_fold`'s cured Symbol arm has **no wat-surface witness** — the wat-level fence
(`wat/rete/compile.wat:578`, keyword-only) refuses first, so all five rows are unit rows.
⛔ **The `is_where_form` guards are still keyword-only in all three descents.**
⭐ **38 of the remaining 161 are the `then-item-fence`** — the same wall the 5 NEW joined, and the
rider names it *"the seventh draw's highest-value question."* Candidate:
`rete/purity.rs::is_declaration_derived_construction`.

## VERDICT

**CURE ACCEPTED, CONVERSION STOP.** clippy 0 (not cached, 12.85 s), census `no STOP-8` 213 = 213,
delta **3 / RECOVERY 0**, 0 live `.wat`. **Twenty-four corrections across twenty-two stones.**

**Seventh draw:** the `then-item-fence` — ⭐ **38 of 161, and the 5 NEW point at the same wall.**
⛔ **Named by SINGLE-FILE CONVERSION, not by log inference.**
