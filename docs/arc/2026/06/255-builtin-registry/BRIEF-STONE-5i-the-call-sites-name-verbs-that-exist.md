# BRIEF — STONE ⑤-i: the call sites name verbs that EXIST

Groups **B, C and D** of `DESIGN-STONE-5-the-blanket-dies-closing-the-seven.md`. **Read that design
first** — every replacement below is measured there and re-stated here. One concern: *call sites
naming verbs that do not exist, or that were retired.* All of it lands GREEN under the blanket; the
deletion is a later stone.

## B · `:wat::kernel::abort` — a PHANTOM in three fixtures

`grep -rn "kernel::abort" src/` → **nothing**. No dispatch arm, no scheme, no row. It type-checks
only because the blanket accepts any `:wat::*` head.

```
tests/reflection/wat_arc201_holon_ast_accessors_children_parametric.wat:16
tests/reflection/wat_arc201_holon_ast_accessors_children_sig.wat:11
tests/reflection/wat_arc201_holon_ast_accessors_first_err_empty.wat:9
```

Each is `(:wat::kernel::abort "<message>")` in a diverging `match` arm.

```
FROM  (:wat::kernel::abort "<message>")
TO    (:wat::kernel::assertion-failed! :message "<message>")
```

**MEASURED** in that exact position (a diverging `match` arm in a typed fn): `--check` exit 0.
Same disposition and same shape as stone ④'s `:wat::kernel::panic!`. Keep each message verbatim.

## C · `:wat::core::vector` — a PHANTOM, and fixing it DE-VACUIFIES its test

`tests/types/probe_arc256_generic_defclause_c05.wat:5`

```
FROM  (:user::len-of (:wat::core::vector 1 2 3))
TO    (:user::len-of (:wat::core::Vector :- [:wat::core::i64] 1 2 3))
```

**MEASURED:** the replacement runs and prints `"[1, 2, 3]"`, exit 0. ⚠ Bare
`(:wat::core::Vector 1 2 3)` is REFUSED — the head requires its `:- [T]` param-spec first, so the
param-spec is not optional decoration.

★★ This is not cosmetic. `c05_parametric_container_clause_checks` asserts *"a clause over a
parametric container head (Vector T) … dispatch matches by head."* Today it feeds a phantom, whose
type the checker resolves to a FRESH VARIABLE, which unifies with `(Vector :- [T])`
unconditionally — **the test passes whether or not head-matching works.** After this edit it tests
its own claim. ⚠ The fixture's header still says *"RED at HEAD"*; that is stale (the `.rs` asserts
`is_ok()`). Correct it while you are there.

## D · retired `i64` spellings — 23 sites in `src/resolve/mod.rs` unit tests

`src/remedy/retirement.rs:190` — `retired: ":wat::core::i64::+"` → `replacement: ":wat::i64::+"`.
Both replacement targets are REGISTERED (`src/intrinsic/i64.rs`: `:wat::i64::+`, `:wat::i64::*`).

```
wat.core/i64::+        →  wat.i64/+
wat.core/i64::*        →  wat.i64/*
:wat::core::i64::+     →  :wat::i64::+
:wat::core::i64::*     →  :wat::i64::*
```

These are unit tests using a namespaced symbol as a SAMPLE to measure normalize's data-vs-code
boundary; the name's identity is incidental to what they assert, which is why this is mechanical.

★ **Their subject was itself a phantom.** `normalize_skips_match_pattern_but_rewrites_body` asserts
*"the BODY symbol must be rewritten to its keyword FQDN"* — true under the blanket for ANY name.
After this, the assertion is about a name that exists.

⛔ **TWO THINGS THE SWEEP MUST NOT TOUCH:**

1. **`wat.core/+` at `src/resolve/mod.rs:462` and `:465`.** It sits in a QUASIQUOTE TEMPLATE and
   line 465 asserts it STAYS a `Symbol` — it is data, deliberately never rewritten. Leave it. (The
   `wat.core/i64::*` on the same line sits inside an `unquote`, is CODE, and DOES migrate.)
   Lines 63 and 112 are doc-comment prose using `wat.core/+` as an illustration — leave those too.
2. **`src/resolve/mod.rs:320-321`'s comment**, which currently reads:
   > *"`wat.core/i64::+` is a reserved prefix → normalize rewrites it to `:wat::core::i64::+`;
   > resolve then accepts it as a known builtin."*

   It is **FALSE and it is the blanket's own rationale written into a test**: that name was never a
   builtin; resolve accepted it because the prefix was blanket-accepted. **Rewrite the comment to
   say what is true after the migration** — the name IS a registered builtin now, and that is why
   it resolves. Do not delete the comment; correct it.

## Blast radius

`src/resolve/mod.rs` (its `#[cfg(test)]` module only) · 4 `.wat` fixtures under `tests/`.
**No production `src/` behaviour changes. No change to `walk.rs`, `normalize.rs`, or the registry.**

## Acceptance

```
cargo nextest run --release -E 'test(resolve::tests)'                            all pass
cargo nextest run --release -E 'test(probe_arc256_generic_defclause)'            all pass
cargo nextest run --release -E 'test(wat_arc201_holon_ast_accessors)'            all pass
target/release/wat tests/types/probe_arc256_generic_defclause_c05.wat            exit 0
grep -rn "kernel::abort" tests/ | wc -l                                          → 0
```

## STOP triggers — each is a REJECTION. Ship nothing; report.

**STOP-1.** If `wat.core/+` at `mod.rs:462/465` changes, or its test stops asserting the symbol
SURVIVES — STOP. That row measures the quote boundary; migrating it destroys what it measures.

**STOP-2.** If a replacement target turns out not to be registered — STOP and report which. Do NOT
pick a different verb to make a test pass.

**STOP-3.** If any `assert!` in `resolve::tests` has to be WEAKENED to stay green — STOP with the
assertion. These migrations must not change what a test claims, only the name it claims it about.

**STOP-4.** If you find yourself editing `walk.rs`, `normalize.rs`, or anything under
`src/intrinsic/` — STOP. Out of scope; a later stone owns those.

**STOP-5.** If `probe_arc256_generic_defclause_c05` goes RED after the fixture edit — **that is a
FINDING, not a failure to fix.** It would mean head-matching does not actually work and the phantom
was hiding it. STOP and report the verbatim error.

## Tier

You edit and report. Run the five acceptance commands, foreground, and nothing else. **Do NOT run
`scripts/floor.sh` or `cargo clippy`** — the orchestrator runs those centrally, once, on a
quiescent tree. **Do NOT commit.**

Work in `/home/john/work/holon/wat-rs`; verify with `pwd` first. Any path containing
`.claude/worktrees/` is harness state and must not be operated on.
