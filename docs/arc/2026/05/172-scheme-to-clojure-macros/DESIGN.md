# Arc 172 — Scheme → Clojure macro flavor swap

**Status:** opened 2026-05-10 per arc 170 REALIZATIONS pass 14
decision. Sized at author time per pass 14 disposition.

**Sibling-coordinated with arc 171** (`:foo,N` → `:foo'N`).
Both arcs together complete the lexical EDN-compliance pivot
that slice 1f-W opened.

## Motivation

Per REALIZATIONS pass 14 (lock-in 2026-05-10):

> *"if yes [transmit macros over the wire].. we need to remove
> the comma... we have scheme macros now.... we need to swap
> to clojure macros..."*

The current substrate uses Scheme/CL-style quasiquote tokens:
- `` ` `` — quasiquote
- `,foo` — unquote (substitute foo's bound value)
- `,@list` — unquote-splicing (splice list's elements)

**This contradicts EDN-compliance.** EDN treats commas as
whitespace; wat's lexer at `src/lexer.rs:234` makes commas
into meaningful unquote tokens. The mismatch means wat source
can't round-trip through EDN parsers without lossy
re-interpretation.

The locked Clojure shape (pass 14):
- `'foo` — quote (equivalent to `(quote foo)`)
- `` `foo `` — syntax-quote (Clojure's quasiquote with
  auto-namespace)
- `~foo` — unquote
- `~@foo` — unquote-splicing
- `name#` — auto-gensym (within a syntax-quote scope)
- `gensym` — manual fresh symbol generator
- `&form` / `&env` — implicit macro arguments (the calling form
  + lexical env)
- **Commas as whitespace** (per EDN)

After this arc ships, **wat is lexically EDN-compliant.**

## Scope — substantial

This arc reshapes the macro layer of wat's substrate. Most
existing wat-side macros (defmacro bodies in `wat/*.wat`,
`wat-tests/*.wat`, `examples/*/wat/*.wat`) use comma-unquote;
they all migrate.

### Substrate edits

**Lexer change** (`src/lexer.rs`):
- Retire `,` as a token-producing character; comma becomes
  whitespace per EDN
- Recognize `~` as new Unquote token; `~@` as new
  UnquoteSplicing
- Recognize `name#` (trailing `#`) as auto-gensym in
  syntax-quote scope
- `'foo` already works as `(quote foo)` in arc 010's variadic
  quote per the existing precedent (verify before arc kickoff)

**Parser change** (`src/parser.rs`):
- Token-renaming flows from lexer; parser dispatch shape
  unchanged (still produces `:wat::core::unquote` /
  `:wat::core::unquote-splicing` reader macros)

**Macro evaluator change** (`src/macros.rs`):
- Auto-gensym handling: `name#` inside syntax-quote scope
  resolves to a fresh symbol consistent across all uses within
  the same syntax-quote
- `&form` / `&env` implicit args populated at macro expansion
  time
- `gensym` primitive added as a non-macro-time function

**Consumer sweep — broad** (~20-30 wat files based on
2026-05-10 grep):
- `wat/edn.wat`, `wat/runtime.wat`, `wat/test.wat`,
  `wat/core.wat`, `wat/console.wat`, `wat/list.wat`,
  `wat/stream.wat`, `wat/kernel/channel.wat`
- All `wat/holon/*.wat` files
- `wat-tests/*.wat` (numerous)
- `examples/*/wat/*.wat` (with-loader, with-lru,
  console-demo, interrogate)
- Possibly `crates/wat-lru/wat/`, `crates/wat-telemetry/wat/`
  (verify scope at slice author time)

Pattern: `,name` → `~name`; `,@list` → `~@list`. Mechanical.

### What does NOT change

- Quasiquote semantics (substitute / splice / nesting) —
  unchanged; only the lexical surface
- `:wat::core::unquote` / `:wat::core::unquote-splicing`
  reader-macro names — unchanged at the AST level (only the
  source-level tokens that produce them change)
- Macros that don't use unquote (just emit literal forms) —
  unaffected

## Slicing plan

Predicted: 3-6 hours total across multiple slices.

### Slice 1 — Lexer + parser swap (minimum viable)

- Lexer: `,` → whitespace; `~` → Unquote; `~@` → UnquoteSplicing
- Parser: token rename only; macro semantics unchanged
- Tests: verify the new tokens produce the same reader-macro
  AST; verify comma-as-whitespace doesn't break other parsing
- During this slice: existing macro bodies break (their
  commas no longer parse as unquote). The slice is intentionally
  BROKEN-WORKSPACE because slice 2 sweeps fixes.

Predicted: 60-90 min opus.

**Atomic-commit pair with slice 2.** Per recovery doc § "Atomic
commit across coordinated sweeps" — slice 1's output is the
broken intermediate; slice 2 fixes it; commit both together
when workspace is green.

### Slice 2 — Consumer sweep

- Mechanical `,name` → `~name`, `,@list` → `~@list` across
  ~20-30 wat files
- Verify cargo test workspace returns to baseline post-sweep
- Sonnet-tier — pattern is verbatim

Predicted: 60-120 min sonnet.

### Slice 3 — Auto-gensym + `&form` / `&env`

- Macro evaluator: implement `name#` auto-gensym in
  syntax-quote scope
- Populate `&form` and `&env` as implicit args in defmacro
  bodies
- Add `gensym` primitive (Rust eval arm + check arm)
- Tests covering the new features

Predicted: 90-150 min opus.

### Slice 4 — Closure

- INSCRIPTION + 058 row + USER-GUIDE + ZERO-MUTEX cross-ref
  (the macro substrate doesn't touch concurrency, but the
  closure paperwork captures the lexical pivot)
- Memory amendment: wat-rs lexically EDN-compliant; macros use
  Clojure flavor

Predicted: 30-45 min sonnet.

## Dependencies

- Arc 170 slice 1f-W shipped (`4278c4d`) — wire encoding rule
  already locked; this arc extends EDN-compliance to the
  source surface
- Arc 171 should ship FIRST (or concurrently): arc 171 retires
  `,` in dispatch position; arc 172 retires `,` as token
  entirely. If arc 172 ships first and `wat/core.wat:96`
  still has `,2`, parsing breaks
- Independent of arc 170's slice 1f-β-i+ (the wat-side service
  work) IF those slices author no new macros. Slice 1f-β-i's
  service implementation (per BUILD-PLAN) uses enum + typealias
  + fn + match + let — no quasiquote needed. Confirmed safe to
  parallel-track

## Risks

- **Macro-evaluator subtlety** — auto-gensym + `&form`/`&env`
  require careful scope handling. Worth referencing Clojure's
  reference implementation behavior (or substrate-testing
  cross-language for known-good idioms)
- **Consumer sweep size** — ~20-30 files is large enough that
  honest deltas could surface (a wat file using a Scheme-only
  idiom). Sonnet surfaces; orchestrator decides scope
- **Risk of breaking macro tests** — `wat/test.wat`'s deftest
  macros use comma-unquote heavily. The sweep fixes them; but
  if tests rely on Scheme-specific semantics (e.g., `,,foo`
  for double-quote-depth, per arc 029), they need
  per-occurrence translation. Worth a pre-grep audit
- **Lab consumers** — `holon-lab-trading/` likely has macro
  consumers; surface scope at slice 2 author time

## Ship criteria (whole-arc)

- Lexer: `,` is whitespace; `~`/`~@` produce Unquote /
  UnquoteSplicing tokens
- All wat source migrated to `~name` / `~@list`
- Auto-gensym + `&form`/`&env` work per Clojure spec
- Workspace cargo test green (no regressions; all macro
  consumers compile + run)
- INSCRIPTION ships per FM 11 discipline (no "deferred to
  future" language)

## Cross-references

- Arc 170 REALIZATIONS pass 14 (the decision that opened this
  arc)
- Arc 170 slice 1f-W (`4278c4d`) — the FIRST thread of
  EDN-compliance to ship
- Arc 171 — sibling arc (sweep `:foo,N` → `:foo'N`); ships
  before this arc to avoid `,` as token
- Arc 010 (variadic-quote) — quasiquote primitive precedent
- Arc 029 (nested-quasiquote) — depth-tracking semantics this
  arc preserves
- Arc 091 slice 8 — runtime quasiquote + struct→form;
  consumer this arc sweeps

## What comes after

When arc 172 closes, wat-rs is lexically EDN-compliant.
Three follow-on benefits surface:

1. **wat source files round-trip through EDN parsers.** External
   tooling that reads EDN (editors, formatters, syntax-aware
   tools) can consume wat source without special-casing
   commas.

2. **Macros can ship over the wire as EDN values.** The pass-14
   motivation ("if yes [transmit macros over the wire]") is
   unlocked — macros become serializable data.

3. **One less special-case in the substrate.** Commas being
   whitespace is one less rule to teach + maintain. Per
   `feedback_verbose_is_honest.md`: fewer special chars =
   smaller surface = honest naming.
