# wat-rs arc 072 — Lexer tracks `<>` depth in type-keyword brackets — INSCRIPTION

**Status:** shipped 2026-04-27. One commit, ~30 minutes — substrate
fix surfaced by proof 018's walker rewrite (after arc 071 closed
the first flaw blocking it).

Builder direction (2026-04-27, after the second flaw revealed
itself behind the first):

> "this is an arc — yes — incredible. how long has this been
>  here?... what test didn't we write?"

> "we haven't shipped a bug in a long time"

The flaw was real, narrow, and ancient — but the DESIGN's
diagnosis named the wrong layer. The infra session's reproduction
traced the actual mechanism + shipped a one-call lexer fix that
turns an opaque downstream error into a clean lex-layer
diagnostic.

---

## The actual mechanism (vs. the DESIGN's diagnosis)

The DESIGN named the flaw as "let*-binding annotation doesn't
propagate to subsequent match arms" with the expected fix being a
type-checker constraint addition. Reading the type checker
confirmed: the propagation logic is correct. `infer_let_star`
inserts the binding's annotated type into locals; `infer_match`
unifies the scrutinee's type with a fresh `Result<:T, :E>`; the
shape is re-substituted post-unification; pattern_coverage
threads the substituted T into the pattern bindings. All correct.

The actual bug was one layer up — in the **lexer**. The keyword
tokenizer tracked `()` depth so `:fn(:(T,U),V)->R` lexes as one
token despite internal whitespace, but it ignored `<>` depth.
Source like:

```scheme
((wrapped :Result<(i64,i64), i64>) (Ok ...))
                              ^^^space-after-comma
```

…tokenized the type annotation as `:Result<(i64,i64),` (lexer
broke at whitespace inside the unclosed `<`) plus a separate
`i64>` symbol. The type parser saw a malformed Result with one
arg; the rest dropped silently. The type checker downstream saw a
fresh var `T` that nothing constrained; eventually surfaced as
`:wat::core::second: parameter #1 expects tuple or Vec<T>; got
:?71` at the pattern-arm body — *the symptom the DESIGN was
diagnosing as a propagation gap*.

The fix is in the lexer:

```rust
let mut paren_depth = 0i32;
let mut angle_depth = 0i32;        // NEW

while ... {
    if c.is_whitespace() {
        if paren_depth > 0 || angle_depth > 0 {  // NEW: angle too
            return Err(LexError::UnclosedBracketInKeyword(i));
        }
        break;
    }
    match c {
        '(' => { paren_depth += 1; out.push(c); }
        ')' => { ... }
        '<' => {
            // Disambiguate type-head `<` from operator `<`.
            // Operator `<` follows `::` (last emitted char is `:`).
            // Type-head `<` follows an alphanumeric (Result<, Vec<).
            let prev_alpha = out.chars().last()
                .map(|ch| ch.is_ascii_alphanumeric() || ch == '_')
                .unwrap_or(false);
            if prev_alpha {
                angle_depth += 1;
            }
            out.push(c);
        }
        '>' => {
            if angle_depth > 0 { angle_depth -= 1; }
            out.push(c);
        }
        ...
    }
}
```

The disambiguation matters: `:wat::core::<` and `:wat::core::>=`
are valid keyword paths (operators). The `<` / `>` in those paths
follow `::`, so `out.chars().last()` is `:`, not alphanumeric —
angle_depth stays at 0, the `<` lexes as part of the keyword
path. Type-head `<` always follows an alphanumeric character (the
type's name: `Result<`, `Vec<`, `HashMap<`) — angle_depth
increments, whitespace-inside-brackets becomes a clean lex error.

The substrate's whitespace rule for type keywords stays strict
(no whitespace inside `:<...>` / `:(...)` / `:fn(...)` /
`:[...]`). With the canonical `:Result<i64,String>` form
(no space), the let*-bind-then-match flow already worked — the
type checker's propagation was fine.

---

## How long had this been here

Probably since `<>` syntax for parametric types was added — the
existing `()` tracking in `lex_keyword` was a focused fix for a
specific case. `<>` was simply never added. Type-keyword tests
all used canonical (no-whitespace) syntax, so the gap stayed
hidden. Proof 018's walker rewrite happened to write
`:Result<(i64,i64), i64>` with a natural space after the comma —
the way a developer would type it — and surfaced both the bug
and a misleading downstream error.

---

## Why the substrate's own tests didn't catch it

The substrate's test corpus uses canonical no-whitespace
syntax everywhere. `tests/wat_*.rs` and `wat-tests/*.wat` all
write `:Result<T,E>` not `:Result<T, E>`. There was never a
test that asserted "whitespace inside type-keyword brackets
produces a clean error." The whitespace rule lived in builder
intuition (the memory feedback `feedback_wat_keyword_whitespace.md`
captures it as "no spaces inside `:(...)`, `:fn(...)`, `:<...>`,
`:[...]`") — the substrate enforced the `()` half but
**silently truncated** for `<>`.

---

## What shipped

| Slice | Module | LOC | Tests | Status |
|-------|--------|-----|-------|--------|
| 1 | `src/lexer.rs::lex_keyword` — `angle_depth` counter alongside `paren_depth`. `<` after alphanumeric increments; `>` decrements when depth > 0; whitespace inside any unclosed bracket raises `LexError::UnclosedBracketInKeyword`. `src/types.rs::tests::malformed_parametric_name_rejected` — updated to accept either lex-layer or type-layer rejection (the test's invariant is rejection, just at a better layer now). | ~25 Rust | 4 new (`tests/wat_arc072_letstar_parametric.rs`: simple Result, Result-with-tuple-payload, whitespace-raises-clean-lex-error, operator-`<`-`>=`-still-lex) | shipped |

**wat-rs unit-test count: 721. Workspace: 1096 → 1100 (+4
regression tests). All passing.**

Build: `cargo build --lib` clean. `cargo clippy --lib`: zero
warnings.

---

## What this unblocks

- **Proof 018's walker rewrite** — the second flaw is gone. With
  arc 071's parametric variant constructors + arc 072's lexer
  bracket tracking, the trader-shape walker pattern lexes,
  type-checks, and runs end-to-end (using canonical
  no-whitespace type annotations).
- **Future debugging** — the next time someone writes
  `:Result<T, E>` (intuitive whitespace), the error points
  exactly at the offending byte instead of a fresh-var unsolved
  three layers downstream. That's the diagnostic improvement
  that takes proof-018-shape sessions from days to minutes.
- **Lab umbrella 059 slice 1** — can consume `walk` directly
  per arc 070's USER-GUIDE (which uses canonical syntax).

---

## What this arc deliberately did NOT do

- **Relax the whitespace rule** — `:Result<T, E>` (with space)
  remains rejected. Per memory feedback (the substrate's existing
  convention) and consistent with how `:(...)`, `:fn(...)`,
  `:[...]` already work. The arc fixes the diagnostic, not the
  rule.
- **Add type-checker propagation tests** — the DESIGN proposed
  six new tests for "let* → match → arm-binding → consumer of
  parametric structure." Reading the checker showed that
  propagation already works correctly; the tests would only
  reaffirm what already passes. Skipped.
- **Audit other parser/lexer corners for similar gaps** — the
  paren-depth tracking covers `(...)`. Angle depth now covers
  `<...>`. Square brackets `[...]` exist in some keyword forms;
  worth a follow-up audit if a similar surface bites. Not now.

---

## The thread

- **Arc 028** — Result wrap. The first parametric tagged enum.
- **Arc 048** — tagged-enum constructors + match. Parametric
  match patterns landed.
- **Arc 055** — recursive patterns. Sub-patterns work.
- **Arc 070** — `:wat::eval::walk` + `WalkStep<A>`. First
  consumer of parametric variant constructors at scale.
- **Arc 071** — lab-harness enum-method parity. Closed the
  *first* flaw blocking proof 018; also discovered the
  parametric-decl-type bug inside register_enum_methods. Plus
  the structural follow-up: substrate test path now type-checks.
- **Proof 018 walker rewrite (2026-04-27, post-arc-071)** —
  surfaced this arc with `:Result<(i64,i64), i64>` (intuitive
  whitespace).
- **DESIGN drafted (2026-04-27)** — diagnosis named "let*
  propagation gap" but the actual cause was one layer earlier
  (lexer ignoring `<>` depth).
- **Arc 072 (this)** — lexer tracks `<>` depth + whitespace-
  inside-brackets surfaces as clean lex error. Tests pass with
  canonical no-whitespace syntax. The arc-070 USER-GUIDE
  example becomes runnable in real proof contexts.
- **Next** — proof 018's walker rewrite ships. The lab probe
  at `experiment/099-walkstep-probe/probe.wat` needs its
  whitespace fixed (lab-side change); after that, the proof's
  walker collapses to the documented pattern.

PERSEVERARE.
