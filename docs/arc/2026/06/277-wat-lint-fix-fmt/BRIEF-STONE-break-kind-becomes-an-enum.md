# BRIEF — STONE: Break.kind becomes an enum

Turn `:wat::fmt::Break.kind` from a free-form `String` into a two-variant enum, and let the checker's
exhaustiveness replace the runtime raise that currently defends it. Read
`[[DESIGN-STONE-break-kind-becomes-an-enum]]` first — it carries the site census, the three probes
that prove every composition, and the one wall that must SURVIVE.

## READ IN ORDER

1. **`wat/fmt.wat:9-13`** — the `Break` record and the comment that justifies the String. **The
   comment is FALSE and the DESIGN shows the probe that refutes it.** Delete it with the String.
2. **`wat/fmt.wat:298-312`, `pad-break`** — the `if`-with-a-raise that this stone retires into an
   exhaustive `match`. **This is the point of the stone**, not a side effect.
3. **`wat/fmt.wat:975-999`, `breaks-map`** — its value type becomes the enum, and the
   `"conflicting Breaks"` raise at 995 **STAYS** (a different invariant, with a sabotage test).
4. **`wat-scripts/fmt/rules/defn.wat:30,40,52`** — three of the 23 assert sites; the shape to rewrite.
5. **`wat-scripts/fixes/positional-to-kwargs.wat`** (or any sibling) — **the codemod shape to copy.**
   `wat/fix.wat`'s `fix-source` walks the form tree; edits are span-faithful.
6. **`wat-scripts/scratch-pad/277-can-a-rete-rhs-carry-an-enum.wat`** — the proven RHS pattern.
   **Mirror it; it is evidence, not an assertion.**

## SKETCH

```wat
;; in wat/fmt.wat, beside Break:
(:wat::core::defenum :wat::fmt::BreakKind :wat::enum::Pure
  :Block []
  :Align [])

(:wat::core::defrecord :wat::fmt::Break
  [id   <- :wat::core::i64
   kind <- :wat::fmt::BreakKind])

;; a :then site — PROVEN to compile and fire (see the probe):
:then [(:wat::fmt::Break :id ?c :kind (:wat::fmt::BreakKind::Block))]

;; pad-break — the raise becomes exhaustiveness. NO else arm, NO `_` wildcard:
(:wat::core::match bk
  ((:wat::fmt::BreakKind::Block) …(:wat::i64::+ indent 2)…)
  ((:wat::fmt::BreakKind::Align) …(:wat::i64::+ open-col 1)…))
```

## BLAST RADIUS

```
wat/fmt.wat                     the enum · Break.kind · breaks-map's value type · pad-break
wat-scripts/fixes/<name>.wat    NEW — the recorded codemod
wat-scripts/fmt/rules/*.wat     8 files, 23 sites — rewritten BY the codemod, never by hand
```

**No Rust. No new fact record. No new rule file. No layout change of any kind.**

## STOP TRIGGERS

- **STOP-1 — the 23 rule sites are a CODEMOD, never hand edits and never sed.** R21. Write
  `wat-scripts/fixes/<name>.wat`, **dry-run it on a `/tmp` copy and `diff`**, then apply and commit
  it as the recorded migration. A second run must report **0 changes**.
- **STOP-2 — the CONFLICTING-Breaks wall at `fmt.wat:995` must SURVIVE.** It is a different
  invariant from the unknown-kind check. If the migration cannot keep it firing, STOP and report.
- **STOP-3 — `pad-break` gets an exhaustive `match` with NO `_` wildcard and NO else arm.** A
  wildcard re-opens exactly the hole the enum closes. If exhaustiveness cannot be reached, STOP.
- **STOP-4 — THE FORMATTED OUTPUT MUST BE BYTE-IDENTICAL.** Capture the five real files' output
  BEFORE you start, and diff after. Any drift is a red, not a rounding.
- **STOP-5 — do NOT touch `Node.kind`.** 62 sites, 12 files, a Rust-free producer at
  `grep.wat:194` — it is the NEXT stone and the builder sequenced it second.
- **STOP-6 — do NOT touch `if`, `cond`, defect E, or the `RhsUnresolvableOperand` diagnostic.**
- **STOP-7 — if any existing test goes red, STOP.** Capture the whole block verbatim; do not re-run.

## ⚠ TRAPS

- **`wat/*.wat` is FROZEN into the release binary.** `cargo build --release` between an emitter edit
  and any measurement, or you are measuring the old world. Rule files under `wat-scripts/` are read
  from disk.
- **A wide red mid-migration is EXPECTED** — a String field and an enum field are different types, so
  the checker is the cascade and the fail count is the progress meter (`docs/SUBSTRATE-AS-TEACHER.md`).
  Do not stash, do not revert.
- **The match pattern for a nullary variant is `((:Ns::Enum::Variant) body)`** — parenthesised. A bare
  `:Ns::Enum::Variant` is refused: *"is a tagged variant; pattern must be (`…` binders...)"*.
- Every fixture must `wat --check` clean before the floor.

## THE FLOOR IS MINE

`scripts/floor.sh` and `cargo clippy --all-targets -D warnings` are the orchestrator's. Run the cheap
targeted checks — `--check`, the census, the fixtures, the byte-diff — and report the numbers.
