# EXPECTATIONS — excursus 001 stone WO-OPT

**Written BEFORE the strike, 2026-08-30.** Blast radius derived from the BRIEF's own
"Blast radius" section.

⚠ Floor already carries ONE known failure — `probe_arc278_span_macros`, the journal
key-collision arm. **Expected: exactly that one.**

## The scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | `(write-json v)` type-checks | `wat --check` a 1-arg call | `CHECK=0` — it is `1` today |
| 2 | ★ 1-arg ≡ 2-arg-with-default | compare the two output strings | **byte-identical** — the property, not just "it runs" |
| 3 | same for `write-json-natural` | both forms | identical |
| 4 | 3 args still rejected | `wat --check` | type error — the guard's upper bound holds |
| 5 | 0 args still rejected | `wat --check` | type error |
| 6 | the guard is in the CHECKER | read the `infer_` fn | arity enforced in `check.rs`, not the runtime handler |
| 7 | the exception is documented | the intrinsic file's header | says the JSON verbs are the optional-arity rows, like `reader.rs:80` does for `read-frame` |
| 8 | ⛔ `write` / `write-pretty` unchanged | `git diff -- src/check.rs` around `:19100` | still `Exact(1)`, `params: vec![t_var()]` |
| 9 | no registry reshape | `git diff -- src/intrinsic/mod.rs` | **empty** — no `Range`/`AtLeast` |
| 10 | `wat/edn.wat` unchanged | `git diff -- wat/edn.wat` | **empty** |
| 11 | `crates/wat-edn/` unchanged | `git diff -- crates/` | **empty** |
| 12 | floor | `./scripts/floor.sh; echo "FLOOR=$?"` | exactly one failure, the known arm |
| 13 | prior stones | `probe_ex001_*`, the 6 inst arms, write-opts arms | all PASS |

## Runtime prediction

**30–60 minutes.** The exemplar is exact and three files deep. Most of it is the verify tail
(~1m20s build, ~5m floor).

## Trap-doors

1. **Row 2 is the real test.** "1-arg works" passes with *any* default. Only comparing the two
   outputs proves the omitted argument means `(:wat::edn::opts)` and not something else.
2. **Two verbs, possibly one renderer.** WRITE-OPTS found that `write-json-natural` does *not*
   share `write-json`'s Inst arm. Do not assume one `infer_` fn covers both without checking.
3. **Going Variadic loses the declared arity in the registry.** That is why the guard moves to
   the checker and why the header note (row 7) matters — the next census over intrinsic arities
   will see `Variadic` here and needs to find the reason next to it.
4. **The 8 live call sites are optional to revert.** Leaving them explicit is not wrong. If you
   revert them, they are proof of row 1; if you do not, say so.

## Not in this stone

- `:wat::edn::write`, `write-pretty` — `Exact(1)`, sort-key path.
- The `WriteOpts` struct and constructors — correct as shipped.
- A `Range` arity in the intrinsic registry — arc 255's, not this excursus's.
