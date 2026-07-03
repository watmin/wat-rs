# DESIGN — Stone C3: i64 arithmetic overflow → clean error, not silent wrap

**Thesis.** wat's `i64 + - *` currently `wrapping_add/sub/mul` — silently returns a wrong value on overflow
(`(+ i64::MAX 1)` → `i64::MIN`, reported `OK`). That is the substrate's own doctrine violated: a wrong
state produced silently instead of made loud. clj's default `+` **throws** on overflow ("long overflow").
The builder's ruling: **don't wrap, error.** C3 changes the 6 i64 arith closures from `wrapping_*` to
`checked_*` and returns a clean, distinct **overflow error** — "not a jvm, do what rust does": `checked_add`
→ `None` → a Rust `Result` error, not a JVM exception.

## Grounded contract (clj 1.12.4, verified this session)

```clojure
(+ Long/MAX_VALUE 1) => THROW "long overflow"    ; default + is CHECKED — never wraps, never auto-promotes
(* Long/MAX_VALUE 2) => THROW "long overflow"
;; wat today (the bug C3 fixes):
(:wat::core::+ 9223372036854775807 1) => OK -9223372036854775808   ; SILENT WRAP — the dishonest state
;; wat after C3:
(:wat::core::+ 9223372036854775807 1) => ERR (i64 overflow)        ; clean error, honest
```

**Not the `'` auto-promote path.** clj's default `+` throws; `+'` promotes to BigInt. The builder's ruling
is the *checked* semantics (error), not promotion. wat has `bigint` (C1) as an explicit wider type the
caller chooses; C3 does NOT auto-promote i64 overflow to bigint. Error, then the caller decides.

## The pinned contract

- `i64 + - *` overflow → a **distinct `IntegerOverflow` error** (NOT `DivisionByZero` — conflating them is
  dishonest). Message names it honestly (e.g. `"i64 overflow: <a> <op> <b>"`), mirroring clj's "long overflow".
- **Division stays as-is for `/0`** (`DivisionByZero`), but the `i64::MIN / -1` overflow edge (if a `/` path
  exists on i64) also errors via `checked_div` — do not wrap or panic.
- No auto-promotion to bigint. No behavior change to `f64`/`bigint`/`rational` arithmetic (bigint/rational
  are arbitrary precision — they never overflow; f64 is IEEE — it goes to ±inf, unchanged).

## Rooms (grounded file:line)

```clojure
{:eval-closures  "runtime.rs:4288 (+), :4291 (-), :4294 (*) — |a,b,_| Ok(a.wrapping_add(b)) → checked"
 :inner-closures "runtime.rs:9230 (+), :9231 (-), :9232 (*) — |a,b| Ok(a.wrapping_add(b)) → checked"
 :impl-fns       "eval_i64_arith + arith_i64_i64_inner — the closure Err channel currently maps to
                  DivisionByZero; enrich it to distinguish Overflow vs DivByZero (a small error-kind on the
                  closure's Err, or checked_*().ok_or(overflow) then the impl fn maps the kind)"
 :error-kind     "a new RuntimeErrorKind::IntegerOverflow (or an arith-error kind) — honest, distinct from
                  DivisionByZero; if wat's errors-as-records surface requires a #wat.*/… record, add it (conformare)"
 :division-edge  "the i64 / path (if present) — i64::MIN / -1 → checked_div → error, never panic/wrap"}
```

## Blast radius + the wrap-relying-test cascade

This CHANGES existing behavior, so the compile+test cascade may surface tests that ASSERT wrapped values
(e.g. `(+ i64::MAX 1)` == `i64::MIN`). Those tests were asserting the dishonest behavior — **update them to
expect the overflow error** (or delete if they were only probing wrap). Surface every such site; do not
preserve wrap to keep a test green. Blast radius: `src/runtime.rs` (the 6 closures + 2 impl fns + the error
kind) + any wrap-asserting test.

## Out of scope

- `+'` auto-promote (not a wat concept — the caller chooses `bigint`).
- `f64`/`bigint`/`rational` arithmetic (unchanged).
- Mixed-float contagion (C4).

## STOP triggers

- STOP if `(+ 9223372036854775807 1)` still returns `OK` (wrapped) — it MUST error.
- STOP if the overflow error is reported as `DivisionByZero` — it must be a distinct honest `IntegerOverflow`.
- STOP if `(/ 1 0)` no longer gives `DivisionByZero` (that path stays).
- STOP if a test that RELIED on wrap is "fixed" by keeping wrap — surface it and flip it to expect the error.

## RED spec

`tests/value/probe_rational_C3_i64_overflow.rs`: `(+ i64::MAX 1)` → err, `(* i64::MAX 2)` → err,
`(- i64::MIN 1)` → err; `(/ 1 0)` → still err (DivisionByZero, distinct); a normal `(+ 1 1)` → `2` (no
regression). RED at HEAD: the overflow cases return `OK` with a wrapped value.
