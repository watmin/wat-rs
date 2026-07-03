# BRIEF — Stone A: rationals in wat-edn (the EDN data layer; closes the ward)

**The work (one paragraph).** `clojure.edn` reads `<int>/<int>` as a rational; wat-edn doesn't. Add a
rational value + lexing + normalization + writing to `wat-edn`, matching clj exactly. This closes the last
`clj:OK/wat:ERR` gap (the ward's ratio rows) — parity becomes complete except the intentional eager-tag
superset. Backing type: `num_rational::BigRational` (arbitrary precision; **zero new transitive deps** —
`num-bigint`/`num-integer`/`num-traits` are already in the tree). This is `wat-edn` (data) ONLY; the
language runtime is Stone B.

The spec is two committed RED probes — turn them green:
- `crates/wat-edn/tests/clj_oracle_parity.rs` — the ward, now RED on 8 ratio rows (`1/2`, `4/2`, `6/3`, `1/1`, `0/5`, `-3/4`, `-6/4`, `10/4`).
- `crates/wat-edn/tests/rational.rs` — the value/normalization contract (round-trip) + zero-denominator refusal.

## Deps — WORKSPACE deps (builder-directed: one version, no drift)

- Root `Cargo.toml` `[workspace.dependencies]` (line ~50): add `num-bigint = "0.4"` and `num-rational = "0.4"`.
- `crates/wat-edn/Cargo.toml`: change the existing direct `num-bigint = "0.4"` → `num-bigint.workspace = true`, and add `num-rational.workspace = true`. (Mirror the existing `uuid.workspace = true` line.)

## The value contract — Clojure-faithful normalization (grounded vs Clojure 1.12.4)

`<int>/<int>` → **reduced to lowest terms, sign on the numerator, denominator > 0**, AND:
- **denominator reduces to 1 → an Integer, NOT a Ratio**: `4/2`→`2`, `6/3`→`2`, `1/1`→`1`, `0/5`→`0`
  (clj yields a Long, not a Ratio). Use `Value::Integer(i64)` when it fits, else the existing `BigInt` path.
- else a `Value::Rational`: `1/2`→`1/2`, `-3/4`→`-3/4`, `-6/4`→`-3/2`, `10/4`→`5/2`.
- **zero denominator → ERROR** (`1/0`, `-5/0`): clj says "Divide by zero". Return a clean `Err`, never panic.

## Rooms

- `crates/wat-edn/src/value.rs` — add `Value::Rational(Box<BigRational>)` (+ PartialEq/Eq/Hash/Clone; a
  `type_name` arm — "rational"). `BigRational` normalizes on construction (`BigRational::new` reduces).
- `crates/wat-edn/src/lexer.rs` — `lex_number` (~line 622): after the integer digit run, if the next byte
  is `/` followed by a digit, lex the denominator and produce a rational token; on `/0` return
  `InvalidNumber("divide by zero")`. (The `N` suffix / float paths are unaffected — a ratio has no `.`/`e`/`N`.)
- `crates/wat-edn/src/parser.rs` — wire the rational token → `Value`, applying the den==1→Integer reduction
  (place it wherever cleanest — lexer or parser — but the emitted `Value` must be `Integer` for `4/2`).
- `crates/wat-edn/src/writer.rs` — `Value::Rational` → `"<num>/<den>"`.

## STOP triggers (halt + report)

- STOP if `4/2`/`6/3`/`1/1`/`0/5` would produce a `Rational` rather than an `Integer` — clj reduces them to
  integers; the round-trip probe pins it.
- STOP if a zero denominator (`1/0`) is accepted or panics — it must be a clean `Err`.
- Do NOT change existing integer/float/BigInt/`N`-suffix parsing.
- Do NOT touch the runtime, `wat-reader`, `src/`, or `wat/` — that's Stone B. Scope: `crates/wat-edn/` + root `Cargo.toml`.

## HOW TO WORK

`cargo test -p wat-edn`. Iterate `--test rational` and `--test clj_oracle_parity` first (the spec). Then the
full `-p wat-edn` suite for regressions; `cargo clippy -p wat-edn`.

## Done = green

- `cargo test -p wat-edn --test rational` → green (normalization + zero-denom).
- `cargo test -p wat-edn --test clj_oracle_parity` → green (all 8 ratio rows parse; only the eager-tag exemption remains).
- `cargo test -p wat-edn` → no regression; clippy clean.

Report: the `Value::Rational` shape, where den==1→Integer normalization lives, the lexer branch, the writer
format, the workspace-dep change, files changed, and any STOP hits.
