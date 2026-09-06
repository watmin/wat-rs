# BRIEF — STONE: Node.kind becomes an enum

Turn `:wat::grep::Node.kind` from a free-form `String` into a 14-variant enum mirroring `WatAST`,
map the Rust discriminant to it **once** at the producer, sweep the 49 consumers with a codemod, and
**gate the Rust↔wat correspondence so it cannot drift**. Read
`[[DESIGN-STONE-node-kind-becomes-an-enum]]` first — it carries the corpus census, the measured
variant-name refusal, and the reason the gate is not optional.

This is the same shape as `[[DESIGN-STONE-break-kind-becomes-an-enum]]`, seven times larger, plus a
boundary that stone did not have.

## READ IN ORDER

1. **`wat/grep.wat:36-40`** — the `Node` record. `kind <- :wat::core::String` is the field.
2. **`wat/grep.wat:188-195`** — the PRODUCER. `(:wat::core::ast-kind node)` returns a String and
   `:194` builds the fact. **This is the one place the map belongs.**
3. **`src/edn/render.rs`, `eval_ast_kind`** — the 14 arms and their 14 strings. **The source of
   truth. Read the arms, not the doc comment** — they agree, and the gate must key on the arms.
4. **`wat-scripts/fmt/rules/table.wat:10-11`** — `(:wat::rete::where (:wat::rete::string::= ?ak "list"))`.
   The comparison shape, in a rete `:where`.
5. **`wat-scripts/grep/head-position.wat`** — the same comparison in **wat-grep's own** rules, which
   are NOT the formatter and have their own CLI tests.
6. **`wat-scripts/fixes/break-kind-string-to-enum.wat`** — **the codemod shape to copy.** It is the
   sibling stone's recorded migration and it is proven idempotent AND effective.
7. **`tests/lint/one_name_grammar.rs`** — the `tests/lint/` gate idiom to copy for row 6.

## SKETCH

```wat
;; wat/grep.wat, beside Node — variant names MIRROR WatAST's, measured:
;; a variant named :String is REFUSED ("bare primitive type ':String' is retired").
(:wat::core::defenum :wat::grep::NodeKind :wat::enum::Pure
  :IntLit [] :FloatLit [] :RationalLit [] :BigIntLit [] :CharLit [] :BoolLit []
  :StringLit [] :NilLit [] :Keyword [] :Symbol [] :List [] :Vector [] :Set [] :Map [])

;; the ONE boundary. ast-kind returns a String, so this cannot be exhaustive on its
;; input — the :else is the ONLY raise this migration adds, and it replaces a latent
;; hole at 155 comparison sites.
(:wat::core::defn :wat::grep::kind-of
  [s <- :wat::core::String] -> :wat::grep::NodeKind
  (:wat::core::cond
    ((:wat::core::= s "int") (:wat::grep::NodeKind::IntLit))
    …
    (:else (:wat::kernel::assertion-failed! … s …))))

;; a consumer, in a rete :where — the PROVEN comparator (arc 278 #57):
(:wat::rete::where (:wat::rete::core::enum::= ?ak (:wat::grep::NodeKind::List)))
```

## BLAST RADIUS

```
wat/grep.wat                   NodeKind · Node.kind's type · kind-of at the producer
wat-scripts/fixes/<name>.wat   NEW — the recorded codemod
49 consumer .wat files         BY the codemod, never by hand:
                                 12 fmt rules · 7 grep rules · 11 recorded fixes
                                 3 cli tests · 6 scratch probes · 2 fixtures
tests/lint/<name>.rs           NEW — the sync gate. THE ONLY RUST IN THIS STONE.
```

## STOP TRIGGERS

- **STOP-1 — 49 files is a CODEMOD, never hand edits and never sed.** R21. Write the fix script,
  **dry-run on `/tmp` copies and `diff`**, then apply and commit it. A second run reports **0
  changes**, and a run against a pre-migration file reports **1**.
- **STOP-2 — THE SYNC GATE IS NOT OPTIONAL.** Without it this stone trades the old drift class for a
  new one: Rust gains a 15th `WatAST` variant, `eval_ast_kind` goes `E0004` so the author is forced
  to add an arm, and the wat side then raises at RUNTIME the first time a file carries that shape.
  If a `tests/lint/` gate over `eval_ast_kind`'s arms and `NodeKind`'s variants cannot be written,
  **STOP and report** — do not ship the enum without it.
- **STOP-3 — no `:Unknown` / catch-all variant.** It pushes the hole back out to all 155 sites, which
  is the thing being removed.
- **STOP-4 — exactly ONE raise, and it lives at `grep.wat`'s boundary.** If any consumer needs a
  raise, the design is wrong; STOP.
- **STOP-5 — `wat-grep` IS NOT THE FORMATTER.** 7 rule files and 9 `tests/cli/wat_grep__*` depend on
  `Node`. They are in scope and their tests are a row.
- **STOP-6 — the four corpus files this stone does not edit must stay BYTE-IDENTICAL.** Capture
  before you start. `wat/grep.wat` itself will differ; that is expected and is not a licence for the
  others to move.
- **STOP-7 — do NOT touch `ast-kind`.** It stays a String-returning intrinsic. Changing it is a far
  wider Rust blast radius and one mapping site is enough.
- **STOP-8 — do NOT touch `if`, `cond`, defect E, or `RhsUnresolvableOperand`.**
- **STOP-9 — if any existing test goes red, STOP.** Capture the whole block verbatim; do not re-run.

## ⚠ TRAPS

- **`:String` is a REFUSED variant name** — `"bare primitive type ':String' is retired (arc 109 slice
  1c)"`. Measured across all 14 short-name candidates; it is the only one that fails. Mirroring
  `WatAST` sidesteps it and makes the correspondence readable.
- **`wat/*.wat` is FROZEN into the release binary.** `cargo build --release` between an edit and any
  measurement. `wat-scripts/` is read from disk.
- **The 11 recorded migrations in `wat-scripts/fixes/` are in scope and must be rewritten.** They are
  live code under `every_wat_scripts_file_loads`; git preserves what they were.
- **A census on `"list"` will find phantoms** — it is also ordinary text and part of FQDNs like
  `:wat::core::list`. Anchor the count to the comparison SITE, not the substring.
- **A very wide red mid-migration is EXPECTED and is the progress meter**, not a crisis
  (`docs/SUBSTRATE-AS-TEACHER.md`). Do not stash, do not revert.
- The match pattern for a nullary variant is `((:Ns::Enum::Variant) body)` — parenthesised.

## THE FLOOR IS MINE

`scripts/floor.sh` and `cargo clippy --all-targets -D warnings` are the orchestrator's. Run the cheap
targeted checks — `--check`, the census, the CLI tests, the byte-diff, your new gate — and report the
numbers.
