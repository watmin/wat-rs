# REFUTE — the floor is RED, and the cause is a layering inversion (2026-09-05)

**Two failures. Not re-run.** `scripts/floor.sh` captured them; both arms verbatim below.

```
Summary [121.622s] 5179 tests run: 5177 passed, 2 FAILED, 18 skipped
```

## ARM 1 — `intrinsic::tests::checker_skip_debt_is_named_and_frozen`

```
thread 'intrinsic::tests::checker_skip_debt_is_named_and_frozen' panicked at src/intrinsic/mod.rs:1351:9:
checker-skip DEBT LEDGER drifted from the measured population.

NEW — registered but absent from `CheckEnv` (`check_env.get` returns `None`), NOT on the frozen
ledger — `doc_arg_ret_types_match_checker_scheme` is silently skipping these and verifying nothing
about their `@arg`/`@ret` docs. Add each to `FROZEN_CHECKER_DEBT_LEDGER` (or register it in
`register_builtins`, `src/check.rs`, to remove it from the ledger entirely):
[":wat::core::read-string-with-comments"]
```

## ARM 2 — `intrinsic::tests::all_see_fqdns_resolve_to_registered_intrinsics`

```
thread 'intrinsic::tests::all_see_fqdns_resolve_to_registered_intrinsics' panicked at src/intrinsic/mod.rs:2008:9:
Found 1 dangling @see reference(s) in the intrinsic corpus:
dangling @see `:wat::fmt::emit` on `:wat::core::read-string-with-comments`
```

## ⛔ ONE ROOT CAUSE, AND IT IS NOT A TYPO

Both reds say the same thing from two directions: **a `:wat::core::` intrinsic was given a
`:wat::fmt::` surface.**

```
:wat::core::read-string             -> :wat::core::ReadOutcome    ← registered in RUST, src/types.rs:1268
:wat::core::read-string-with-comments -> :wat::fmt::Parsed        ← a defrecord in wat/fmt.wat:18
```

`read-string`'s return type is a Rust-side registered type, which is exactly why it *can* carry a
`TypeScheme` and be checked. The new verb returns a record defined in a stdlib `.wat` file that loads
**after** `grep.wat`. The gate constructs `CheckEnv::with_builtins_and_types(&TypeEnv::new())` — a
FRESH TypeEnv, no stdlib loaded — so `:wat::fmt::Parsed` cannot be in it, and **no `TypeScheme` for
this verb is expressible.** `@see :wat::fmt::emit` dangles for the same reason: it names a wat-level
`defn`, not an intrinsic.

**So a core verb depends on a late-loading stdlib file. That is the defect.**

## ⛔ WHY THE LEDGER IS THE WRONG FIX — the gate offers it, and it should be REFUSED

The error message offers two doors. Take the second, not the first. **Every existing ledger entry has
the same reason, and it is NOT ours:**

> *"`and`/`or` are registered special forms with a real check impl (`infer_boolean_shortcircuit`) but
> NO `env.register()` TypeScheme"* · *"`Option/expect` is checked FOR REAL by a hand-written
> `check_call` arm … Retires when a scheme registers — **not by weakening the gate**."*

Those rows say *real checking happens; only the DOC comparison is missing.*
`read-string-with-comments` has **no scheme AND no hand-written inference — it is genuinely
unchecked.** Putting it on the ledger would not record existing debt; it would **admit a new class of
it**, and the ledger's own precedent forbids that reading.

## THE FIX — mirror `read-string`, which is what the BRIEF asked for

The BRIEF's room #1 was *"`read-string`'s handler and its `#[wat_intrinsic(...)]` attribute. **Your
new verb is this, mirrored.**"* The mirror was taken on the handler and dropped on the TYPE.

```
1  Define the parsed-with-comments type in RUST, beside :wat::core::ReadOutcome (src/types.rs:1177-1268).
   Name it in :wat::core::, not :wat::fmt:: — a core verb's surface belongs to core.
2  Register its TypeScheme in register_builtins (src/check.rs, beside :wat::core::read-string
   at :19491) so check_env.get() answers and the debt ledger stays byte-identical.
3  Point @see at something registered, or drop it. :wat::fmt::emit is a wat defn, never an intrinsic.
4  wat/fmt.wat consumes the core type; it does not define the verb's return type.
```

★ **`:wat::fmt::Comment` and the rest of `fmt.wat` are NOT in question** — only the type a
`:wat::core::` verb hands back. Everything downstream of the verb may stay wat-side.

## WHAT IS NOT DISPUTED — the stone's substance stands

- **The acceptance passed in letter.** R11 shipped as a new file; `wat/fmt.wat` and `defn.wat` are
  untouched by it; no Rust rebuild was needed to add it.
- **Idempotence held** — `IDEMPOTENT=true` on every input, printed by the driver.
- **Comments survive the formatter** — 28 in, 28 out, same order, count printed.
- **R1 is right**, including `[]` on its own line.
- **The 13 `expect(dead_code)` are gone**, and the wiring is real: `wat/fmt.wat:140` calls
  `ast->source`, which now routes through the comment-aware printer. **One renderer, not two** — that
  is better than the brief asked for, because it removes the drift risk of a second printer.
- **STOP-5 was honoured exactly.** A fixture went RED on the load gate; it was captured, not re-run,
  and the fixture was corrected. That is the protocol working.

## ⚠ AND A SECOND FINDING, SEPARATE FROM THE RED — R11 CARRIES A HAND-LIST

`wat-scripts/fmt/rules/siblings.wat` contains:

```
(:wat::rete::where (:wat::rete::string::not= ?hn ":wat::core::defn"))
```

R11 excludes `defn` by name so it will not fight R1. The acceptance passed — that exclusion lives in
R11's own file. **But it makes rule N+1 an edit to rule N's file:** adding R3 (`let`) means adding
`not= ?hn ":wat::core::let"` here, and R4 (`match`) another. That is O(N) edits to an EXISTING rule
file per new rule — the extensibility claim failing one step later than the acceptance looks.

⭐ **The DESIGN predicted this exactly**, and named the cure before the stone was drawn:

> *"⚠ The one place ordering is real: the **default** rule for a form no rule names. It must be a
> fallback consulted when head dispatch misses, **never a competing rule** — otherwise it races every
> specific rule and the exclusivity argument collapses."*

R11 **is** the default rule, and it was built as a competing rule with an exclusion list.

**The shape that fixes it, and it is stratifiable:** a specific rule asserts `Claim {form-id}`
alongside its `Break`. R11 fires only where **no `Claim` exists** for that form. `Claim` is produced
by specific rules and consumed by R11 — no cycle, so no stratification refusal
(`[[NOTE-width-is-a-fact-not-a-rule]]` is about a rule aggregating over its OWN output; this is not
that). Then a new rule adds its `Claim` and R11 steps aside automatically, touching nothing.

**Fix it now at two rules, not at fifteen.** `[[feedback_a_gate_over_two_hand_lists_is_a_hand_list]]`
