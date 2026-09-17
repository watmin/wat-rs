# DESIGN — a negative fixture that fails for its own reason makes 200 assertions falsifiable at once

## Why

Work-list **C18**: *"`assert!(!ok)` is unfalsifiable in this repo's negative-fixture idiom."* The row
called it *"a sweep, bigger than any one strike"* and named 5 fixtures. **Driven at HEAD
`545771b2f`, the row is wrong in its mechanism and right in its alarm.**

### What the row says, and what the disk says

| the row | measured |
|---|---|
| *"**Every** `.wat.bad` fixture ends `(:user::main [] -> nil nil)`"* | **6 of 281.** But **230 of 281 have NO `:user::main` at all**, and that is the same failure by a different door |
| *"5+ named fixtures"* | **200 of 283** tests driving a `.wat.bad` assert **only** `is_err()`/`!ok` |
| — | **17 fixtures fail with `MainSignatureError` TODAY** — they die *before* reaching the wall they exist to prove |

Both doors are startup failures, driven:

```
file with (:user::main [] -> nil nil)  ->  #wat.macro/MainSignatureError (UselessMain), exit 4
file with no :user::main at all        ->  #wat.kernel/MainSignatureError "not defined", exit 4
```

### The decisive drive

`probe_arc294_9a_kwargs_ctor_bad.wat.bad` exists to prove bare-positional construction is refused.
Its test is one line: `assert!(r.is_err(), …)`. I made the offending construct **valid**
(`(:Pair 1 2)` → `(:Pair :a 1 :b 2)`) and ran it:

```
[#wat.kernel.LociDiedError/MainSignature ["…:user::main not defined — a wat program needs an entry point…"]]
```

**Still fails.** So the test cannot distinguish *"the wall fired"* from *"there is no entry point"*,
and mutating the wall away leaves it green. That is the class, and it is 200 tests wide.

## ⛔ AND SEVENTEEN FIXTURES ARE ALREADY DEAD, SIX OF THEM SAYING SO IN THEIR OWN HEADERS

The 17 that fail with `MainSignatureError` today never reach their wall at all. Six carry an
**expired premise** and announce it:

> `probe_arc237_8a_no_implicit_coercion_arith_f64_i64.wat.bad:1` —
> *"arc 300 C4: f64 + i64 now COERCES to f64 (mixed contagion; 237.8a's reject retired). **Type-checks.**"*

**That is a `.wat.bad` that is no longer bad.** The behaviour it was written to reject is legal now.
It sits in the negative corpus, green, passing solely on a missing entry point. The whole
`arc237_8a/8b/8c/8d` family is in that state — six files.

A seventh shape: `probe_diagnostic_non_keyword.wat.bad` says *"type-checks but errors at EVAL (not
callable)"*. **With no main, nothing ever evals.** It cannot be testing what it claims.

Two are legitimate — `wat_arc170_slice_1e_user_main_nil_*` test the main signature *itself*.

## The contract decision, pinned

**Do not rewrite 200 assertions. Make the corpus incapable of failing for the wrong reason.**

A gate drives every `.wat.bad` and **refuses any whose failure is `MainSignatureError`**, unless it
carries a per-file rune naming why the main signature IS its subject. That is one gate over a
discovered population, and it converts the 200 `assert!(is_err())` tests from unfalsifiable to
falsifiable **without touching them** — because once a fixture fails only for its own reason,
removing its wall makes the file *succeed*, and `is_err()` goes red on its own.

This is the extirpare ladder's middle rung — *a check that fires at construction* — reached by
changing the corpus rather than the 200 call sites. The top rung (a helper that cannot express a
kindless negative assertion) is **affirmatively deferred to a named successor**, not left implied:
it is a 200-site migration and belongs behind this gate, not in front of it.

## Out of scope = REJECTED

- **Rewriting the 200 `assert!(is_err())` call sites.** The gate makes them falsifiable; migrating
  them to a kind-checking helper is a separate, larger strike and must not be smuggled in here.
- **Deciding the fate of the six expired-premise fixtures by guess.** Each is a real question — is
  the behaviour legal now, and should the fixture become positive, or be deleted? The strike
  resolves them with evidence, one at a time, or STOPs.
- **Widening to `.wat` fixtures that are expected to PASS.** Different population, different question.
