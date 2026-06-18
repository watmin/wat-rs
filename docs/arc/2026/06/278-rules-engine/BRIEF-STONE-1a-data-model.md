# BRIEF — Stone 1a: the rete data model (`wat/rete.wat` born)

Single-hop sonnet in `/home/watmin/work/holon/wat-rs`. **No sub-agents. No `git`.** This is a WAT stone
(pure wat on the stone-0 persistent collections + the stdlib registration in Rust). Build, run the named
tests, report verbatim. Another agent weighs independently.

## The work
Mint **`wat/rete.wat`** holding the rete engine's DATA MODEL: the data-flow records, the `Rule` record, the
MVP node records + the `Node` defenum sum, the `Session` record, and a `render-dag` fn. EDN-round-trippable
(it's all data on `PersistentMap`/`PersistentVector` + records). NO compile, NO fire — just the data model
standing as data. Register the new file in the stdlib load order. Un-ignore the probe.

## Read first (in order)
1. `docs/arc/2026/06/278-rules-engine/DESIGN-STONE-1a-data-model.md` — the EXACT vocabulary (records, fields,
   the `Node` defenum, `Session` + its 7 fields, `render-dag`) + the ONE contract decision (`Element.fact`
   is a `PersistentMap` for v1). Implement it verbatim. The names are intueri-blessed — do not rename.
2. `wat/lint.wat` — `(:wat::Record::def :wat::lint::FixEdit [field <- :Type …])` is the record shape to
   mirror (and record field accessors `:ns::Record/field` are auto-generated — the probe uses
   `:wat::rete::Session/network`).
3. `wat/service.wat` — `(:wat::core::defenum :wat::service::Outcome<S,R> …)` is the EXACT defenum syntax to
   mirror for the `Node` sum. **Read its variant-arm form on disk; do not guess.**
4. `wat/source.wat` (arc 283, a recently-minted stdlib file) + `src/stdlib.rs` — how a new `wat/*.wat` is
   registered in the stdlib load order. Mirror it: register `wat/rete.wat` AFTER `wat/Record.wat` (it uses
   `:wat::Record::def`) and after anything it depends on. (PersistentMap/Vector are Rust intrinsics —
   always available — so Record.wat is the real ordering constraint.)
5. `tests/probe_arc278_1a_data_model.rs` — remove its `#[ignore]`. It hand-builds a 2-node Session and
   asserts `Session/network` length == 2 and `render-dag` returns a non-empty String. It is your contract.

## render-dag
`(:wat::core::defn :wat::rete::render-dag [session <- :wat::rete::Session] -> :wat::core::String …)` — walk
`(:wat::rete::Session/network session)` (id → Node), emit one readable line per node (its id · kind ·
children ids); a simple edge-list/text graph. Use the existing wat string ops (`string::concat` /
`string::interpolate` — interpolate is expand-time-legal but here it's a runtime defn so either works) and
the PersistentMap iteration helpers (keys/values/foldl — grep `wat/service.wat` / `wat/deporder.wat` for the
HashMap fold idiom; the same works on PersistentMap). Match a node variant via `:wat::core::match` over the
`Node` defenum.

## STOP triggers
1. If `defenum` cannot range over record-typed variants the way DESIGN-1a assumes (the `Node` sum) — STOP,
   report what defenum actually supports (read service.wat's Outcome first).
2. If registering `wat/rete.wat` breaks the stdlib load order (deporder violations) — STOP, report the
   dependency it tripped on.
3. If `render-dag` needs a PersistentMap iteration primitive that doesn't exist yet — STOP, name it (do not
   hand-roll around it).

## Verify (paste verbatim)
```
cargo test --release -p wat --test probe_arc278_1a_data_model -- --include-ignored   # 1/1 GREEN (un-ignore it)
cargo test --release --test test_stdlib_load_order | grep result                     # 1/0 (deporder green — rete.wat ordered correctly)
cargo test --release -p wat --lib 2>&1 | grep "test result"                          # 931 / 36 (UNCHANGED — no Rust changes)
cargo test --release --test test 2>&1 | grep "test result"                           # deftest: was 264/1; +1 if you add a wat-tests deftest
cargo test --release -p wat --test nursery -- --test-threads=1 | grep "test result"  # ~893 / 4 (UNCHANGED)
cargo build --release 2>&1 | tail -2                                                  # clean
```
Report: the full `wat/rete.wat` source, the stdlib.rs registration diff, the probe un-ignore, all command
outputs verbatim, any STOP hit. Do not claim a green you did not see. No git.

## Blast radius
NEW `wat/rete.wat` · `src/stdlib.rs` (one registration line in the load order) · un-ignore the probe ·
optionally a `wat-tests/` deftest. NO other Rust. NO behavior change to any existing file. No git.
