# SCORE — STONE: the first layout rules, driven end to end

No commit. No `--fmt` flag. Print to stdout. Rules assert `Break`, never text.

## What shipped

```
src/intrinsic/ast.rs     ADD :wat::core::read-string-with-comments  (read-string UNCHANGED)
src/edn/render.rs        eval_read_string_with_comments; ast->source now calls
                         write_wat_source_with_comments(&[ast], &[], …) — 13 expect(dead_code) GONE
wat/fmt.wat              Break, Comment, Parsed, emit, format-source (baked stdlib)
wat-scripts/fmt/rules/defn.wat       R1
wat-scripts/fmt/rules/siblings.wat   R11 — a NEW FILE
wat-scripts/fmt/run.wat              driver, R1 only
wat-scripts/fmt/run-r11.wat          driver, R1+R11
```

`grep -c 'expect(dead_code' src/edn/render.rs` → **0**.

## The verb (rows 2–3)

`read-string-with-comments` → `Result` of `:wat::fmt::Parsed` (forms + comments). `read-string` handler: git diff is one added fn after it.

`wat/io.wat`: **FORMS=3 COMMENTS=28** (printed). Comments ≥ 10.

## R1 (rows 5–6)

`defn-multi.wat`:

```
(:wat::core::defn :fix::add
  [x <- :wat::core::i64
   y <- :wat::core::i64]
  -> :wat::core::i64
  (:wat::core::+ x y))
```

Arg-spec own line, one arg per line, ret own line, body own line.

`defn-empty.wat`: `[]` on its **own line**.

## Comments survive the formatter (row 7)

`wat/io.wat`: 28 source comments, 28 in output, **same order** (counted). COMMENTS=28 printed.

## Idempotence (row 8)

Driver always prints `IDEMPOTENT=true|false` from `fmt(fmt(x)) == fmt(x)`. Every run this stone: **true** (multi, empty, io, half-broken, R1 and R11).

## ★★ THE ACCEPTANCE (rows 9–10)

R1-only on half-broken match (inside a `defn`):

```
(:wat::core::match x (n n) (_ 0))
```

R1+R11 (`run-r11.wat` loads `siblings.wat`; **no edit to fmt.wat or defn.wat**):

```
(:wat::core::match
  x
  (n n)
  (_ 0))
```

Output **changes**. `grep siblings wat/fmt.wat` empty. `grep siblings wat-scripts/fmt/rules/defn.wat` empty. No Rust rebuild to add R11.

R11 excludes `:wat::core::defn` by head-symbol dispatch **inside siblings.wat** — otherwise pass 2 would break R1's "name stays on line 1" (source lines differ after R1). That exclusion is the composition, not an engine edit.

## Commands

| command | result |
|---|---|
| `cargo build --release` | clean |
| `cargo test --release -p wat-reader` | **106** + **2** totality |
| `cargo nextest run --release -E 'test(every_wat_scripts_file_loads)'` | **1 passed** |
| `cargo nextest run --release --test lint` | **118/118** |

Floor and clippy `--all-targets -D warnings` are the orchestrator's.

## What surprised me / STOP-5

The first half-broken fixture used keyword variants that do not type-check. The load gate went **RED** (1 of 597). Captured, not re-run; fixture rewritten to a legal `match` with `_`. Then 118/118.

`println` EDN-quotes the formatted source (newlines as `\n`). Layout is in that string; the driver does not write files.
