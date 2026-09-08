# BRIEF — STONE N RELAND 3: the string exception, and one site the scream named

> RELAND 2 landed the per-file fmap and — by REMOVING the fold — found the real producer:
> `eval_tail` TCO bypassed the map-ctor intercept. This is the last stone before green.

## THE TRAJECTORY, MEASURED CENTRALLY AT EACH STEP

```
M            2447 passed · 2773 failed · 18 TIMED OUT     (nothing ran; stdlib would not load)
N            4765 passed ·  473 failed ·  0 timed out
N RELAND-1   4947 passed ·  291 failed ·  0
N RELAND-2   5110 passed ·  128 failed ·  0
```

## THE 128 — TWO POPULATIONS, BOTH NAMED

**(1) ~94 — wat embedded in RUST STRING LITERALS.** `@example` doc-tests in `src/intrinsic/**`,
inline test programs in `src/runtime.rs` (`:14450` among them), `src/reflect/*.rs`. **164 references
across ~10 files.** The `.wat` codemod never saw them and never could.

★ **R21 does not govern here and this is the ONE place hand-editing is correct.** These are Rust
string literals, not `.wat` forms. Arc 109's arm campaign hit and named this exact class — *the
string exception* (RELAND 2). Do not build a codemod for it; do not leave it for one.

⛔ **RENAME AND WRAP MUST LAND IN THE SAME EDIT:**

```
(:wat::core::Some 3)        ->  (:wat::core::Option::Some {:value 3})
[:wat::core::Some v  … ]    ->  [:wat::core::Option::Some {:value v} … ]
[:wat::core::None {} … ]    ->  [:wat::core::Option::None {} … ]
(:wat::core::Ok  x)         ->  (:wat::core::Result::Ok  {:value x})
(:wat::core::Err e)         ->  (:wat::core::Result::Err {:error e})
```

Rename alone yields `(:wat::core::Option::Some 3)`, which then fails **positional** instead of
**bare-spelling** — the same site, a new error, and zero progress. That is the trap that turns one
sweep into two.

**The 22 `non-exhaustive: open-typed match needs at least one hash-destructure` are DOWNSTREAM of
the same sites** — `detect_match_shape` (`src/check.rs:6808`) recognises Option/Result only from the
QUALIFIED spelling, so a match with bare arms falls to `MatchShape::Open` and then wants a wildcard
it never needed. They should evaporate when the arms are qualified. **If any survive, that is a
FINDING** — name it with the arm and the verbatim block; do not absorb it.

**(2) 25 failures from ONE site.** `tests/macros/probe_arc278_macro_generates_service.wat:43` —
`(:probe::Echo::EchoResponse::Ok c)` inside a **defmacro body**. The codemod declined it and
SCREAMED (`[positional-ctor] UNRESOLVED`), which is the instrument working exactly as RELAND 2 built
it to.

★ **A scream is where the tool hands over to judgement.** R21 forbids hand-editing a `.wat` corpus
MIGRATION; it does not forbid resolving a single site the tool has explicitly declined and reported.
Fix it by hand, and say in the SCORE why the codemod declined — same class as M2's generated
unquote heads.

## THE DONE-WHEN

```bash
./scripts/floor.sh          # ORCHESTRATOR'S. Do not run it.
./target/release/wat --check "$f" 2>&1 | grep -o "\"$f\"" | wc -l      # 0
cargo clippy --release --all-targets --workspace                        # 0 errors
```

Your acceptance: **0 fixture-local errors**, the five probe rows green, and — stated plainly —
whether any `non-exhaustive` failures survived the qualification.

## STOP TRIGGERS

- **STOP-1 — rename without wrap.** Same site, new error. They are one edit.
- **STOP-2 — a codemod is built for the Rust strings.** They are not `.wat` forms; the string
  exception is the recorded disposition.
- **STOP-3 — a `.wat` corpus site is hand-edited.** R21 still governs `.wat`. The ONE authorised
  hand-edit is the defmacro-body site the tool declined and named.
- **STOP-4 — a surviving `non-exhaustive` is called residue.** It is a finding. Name it.
- **STOP-5 — the wall is weakened.** 128 real sites remain; the wall is correct.
