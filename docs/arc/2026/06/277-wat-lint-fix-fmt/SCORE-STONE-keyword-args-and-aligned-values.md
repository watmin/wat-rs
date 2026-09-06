# SCORE — STONE: keyword arguments, one pair per line, values aligned

No commit. Floor and clippy left to the orchestrator. Level 2 (cross-sibling-call alignment) not attempted.

## The two approved shapes

**No positional** (`kwargs-none.wat`) — head alone, then one pair per line, values in one start:

```
(:wat::grep::Unreadable
  :file   path
  :reason msg
  :line   n
  :col    c)
```

`IDEMPOTENT=true`.

**A positional rides** (`kwargs-pos.wat`) — defservice's shape:

```
(f :wat-tests::recorder
  :satisfies :wat-tests::Recorder
  :durable   []
  :ephemeral []
  :impls     [])
```

`IDEMPOTENT=true`.

## Alignment is the emitter's

`AlignPairs {form}` — a rule asks; the emitter pads each broken child's first token to the widest first token among that form's broken children, from **this pass's** `ast->source`. No rule names a width.

```
grep -c 'col' wat-scripts/fmt/rules/*.wat  →  0 0 0 0 0 0 0 0
```

## One-pair — reported, no carve-out

```
(:wat::fmt::BlankBefore
  :id id)
```

Exploded. Builder may want a one-pair carve-out later; this stone did not add one.

## Level 2 — still open

`grep.wat:284-289`'s inner `Location/line` / `Location/col` arguments across sibling calls are not aligned. Named, not smuggled.

## Claim is its own rule

A first cut asserted Claim in the same `:then` as the key Breaks. Compound values still took R11 Breaks (the `not Claim` race). Splitting Claim+AlignPairs into their own rules, same shape as `defn-claim`, made R11 stay off. Pad is withheld when the next sibling has a Break, so a key line cannot grow trailing space.

## Existing fixtures

All `IDEMPOTENT=true`. `generic-fn` ret-spec still both tokens one line. `type-ctor` still glues+explodes. `io.wat` **COMMENTS=28**.

## Walls

Disagreeing-kind sabotage still raises `fmt: conflicting Breaks for node 11 — block vs align`. Deleted after. `ClaimedUnder` 0.

## Row 12 — the real gate

616 `@example` / `@example-norun` comment lines in `.rs`/`.wat` (excluding `target/`). 2 were prose, not forms. **614 example forms** run through `format-source` (rules collected once).

| | |
|---|---|
| run | **614** |
| changed (byte-unequal after stripping the write-newline artifact) | **342** |
| lines still over 120 | **1** |
| worst remaining | **132 cols** |

Worst remaining shape, verbatim:

```
(:wat::edn::ForeignRecord/get (:wat::core::match (:wat::edn::read-foreign "#some.unknown/Rec {:kind #some.unknown.Kind/Click [42]}")
                                ((:wat::edn::ReadForeignOutcome::Value fr)
                                  fr)
                                ((:wat::edn::ReadForeignOutcome::Malformed _)
                                  (:wat::kernel::assertion-failed! "bad fixture"
                                    :wat::core::None :wat::core::None))) :kind)
```

A positional compound riding the head line (STOP-3), not a kwarg-alignment miss. Not level 2.

## Commands

| command | result |
|---|---|
| `cargo build --release` | clean |
| `kwargs-none` / `kwargs-pos` / `kwargs-one` | ruled shapes, **IDEMPOTENT=true** |
| every previous fixture | ruled + idempotent |
| `run.wat` on `wat/io.wat` | **COMMENTS=28** |
| `grep -c 'col' rules/*` | **0** |
| kind-conflict sabotage | **raises**, then deleted |
| 614 doc examples | 342 changed, **1** line >120, worst 132 |
| `every_wat_scripts_file_loads` | **1 passed** |

No Rust. New: `AlignPairs`, `rules/kwargs.wat`, three fixtures, `load-file!` in every driver.

### Load-gate note

Bare `(:probe::R …)` fixtures failed the loader (`UnresolvedReference :probe::R`; empty `defservice` `:impls` → `defenum must have at least one variant`). Fixtures were rewritten as loadable `defn`s wrapping real constructors / a typed call. Gate then passed. The first red is in the log; it was the fixtures, not the engine.
