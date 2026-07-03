# BRIEF — Stone C3: i64 arithmetic overflow → clean error, not silent wrap

**The work (one paragraph).** wat's `i64 + - *` use `wrapping_add/sub/mul` — `(+ i64::MAX 1)` silently
returns `i64::MIN` and reports `OK`, a wrong value with no signal. clj's default `+` **throws** on overflow
("long overflow"); the builder's ruling is **don't wrap, error**. Change the 6 i64 arith closures from
`wrapping_*` to `checked_*` and return a clean, **distinct `IntegerOverflow` error** (NOT `DivisionByZero`
— conflating them is dishonest; "do what rust does" — a Rust `Result` error, not promotion to bigint).
Turn `tests/value/probe_rational_C3_i64_overflow.rs` green.

## Read in order

1. `docs/arc/2026/07/300-wat-source-is-edn/DESIGN-STONE-rational-C3-i64-overflow.md` — design, contract,
   room table. **Read first.**
2. `tests/value/probe_rational_C3_i64_overflow.rs` — the RED spec (3 tests; the normal-arith one already
   passes, the 2 overflow ones are RED).
3. The existing **`DivisionByZero`** path in `arith_i64_i64_inner`/`eval_i64_arith` — the error the closure
   `Err` channel already produces (for `/0`). You enrich that channel to ALSO carry overflow, distinctly.

## The work

- **The 6 closures** — change `Ok(a.wrapping_add(b))` → checked, at:
  `runtime.rs:4288` (+), `:4291` (-), `:4294` (*) in `eval_i64_arith`; and `runtime.rs:9230` (+), `:9231`
  (-), `:9232` (*) in `arith_i64_i64_inner`. Use `a.checked_add(b)` (etc.) → on `None`, signal overflow
  through the closure's `Err` channel.
- **The error channel** — today the closure returns a `Result` whose `Err` maps to `DivisionByZero` in the
  impl fn. Enrich it so the impl fn can produce **either** `DivisionByZero` (the `/` `b==0` case, unchanged)
  **or** a new **`IntegerOverflow`** — e.g. give the closure's `Err` a small kind, or have `checked_*().ok_or(kind)`
  and map the kind in `eval_i64_arith`/`arith_i64_i64_inner`.
- **The error kind** — add `RuntimeErrorKind::IntegerOverflow` (honest, distinct), message naming it (e.g.
  `"i64 overflow"` — mirror clj's "long overflow"). If wat's errors-as-records surface requires a
  `#wat.*/…` record for it, add it the way the other arith errors are declared (grep `DivisionByZero`).
- **Division edge** — if the i64 `/` path can hit `i64::MIN / -1` (an overflow), route it through
  `checked_div` → the overflow error too (never panic/wrap). `/0` stays `DivisionByZero`.

## The wrap-relying-test cascade (expected)

This CHANGES existing behavior. The test cascade may surface tests that ASSERT the wrapped value (e.g.
`(+ i64::MAX 1)` == `i64::MIN`). **Those tests were asserting the dishonest behavior — flip them to expect
the overflow error (or delete if they only probed wrap).** Do NOT preserve wrapping to keep a test green.
Surface every such site in your report.

## How to work

Green at HEAD (post-C2). Make the change, follow the compile cascade, then:
`cargo test -p wat --test value probe_rational_C3_i64_overflow` (3/3), then the other rational probes
(`probe_rational_C2_arithmetic`, `_C1_bigint`, `_B_runtime_representation` — still green), then
`cargo test -p wat-edn`, then a broad `cargo nextest run` — **read the Summary; capture once to a file,
grep the file**. The suite must show **exactly ONE standing red**: `no_inlined_wat_in_tests` (the meter,
351) — UNLESS your change surfaces wrap-relying tests, which you FLIP to expect the error (they must end
green, not add to the red count). If `wat-cli sigterm…` trips, verify solo with `--test-threads=1` (the
arc-170 race). `deporder` is fixed (30s). Any other red is a real regression — halt + report. Do NOT commit.

## STOP triggers

- STOP if `(+ 9223372036854775807 1)` still returns `OK` (wrapped) — it MUST error.
- STOP if the overflow error is reported as `DivisionByZero` — it must be a distinct named `IntegerOverflow`.
- STOP if `(/ 1 0)` no longer gives `DivisionByZero`.
- STOP if you keep wrapping anywhere to make a test pass — flip the test instead.
- STOP if this needs changes to `f64`/`bigint`/`rational` arithmetic — those are unaffected (bigint/rational
  are arbitrary precision; f64 is IEEE).

## Done = green

- `probe_rational_C3_i64_overflow` 3/3; the other rational probes still green.
- `cargo build -p wat` clean; `cargo test -p wat-edn` green.
- Broad `cargo nextest run`: exactly one standing red (the 351 meter); any wrap-relying tests flipped to
  green; no new failures.

Report: files changed; the error-kind mechanism; every wrap-relying test you flipped; the Summary line; any STOP hits.
