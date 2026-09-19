# DESIGN — census B: `compiled:calls` counts pairs, not calls

Census audit section B (`../vigilia-2026-09-05/recon/census-name-audit.md:41-49`), **re-grounded at
HEAD 2026-09-06** — the defect is live and larger than the audit rowed it.

## The finding

Two sites bump `compiled:calls`:

- `src/rete/compiled_cond.rs:959` — inside `exec_compiled_with_key_ids`. **A real execution.**
- `src/rete/kernel/fire/delta.rs:78` — inside `let matched = if skip_span { … Some((0u32,0u16)) }`.
  **The executor is never invoked; this arm exists precisely to skip it.**

So the key counts **candidate (fact, alpha) pairs considered**, of which one arm executes and one
elides. Four sites in the tree say otherwise:

| site | says |
|---|---|
| `compiled_cond.rs:955` | *"the compiled path's **call counter**"* |
| `accum_cost.rs:44` | *"`compiled:calls` is a **CALL count**: one bump per compiled-condition **execution**"* |
| `accum_cost.rs:93` (assertion message) | *"a compiled condition is now **EXECUTED** on this axis"* |
| `accum_alpha_cost.rs:1365` | *"counts compiled-condition **EXECUTIONS only**"* |

And **one site tells the truth, 20 lines above the last of them** — `accum_alpha_cost.rs:1345`:
*"bumps once per (fact, candidate alpha) pair on BOTH sides … equal across the two arms by
construction and **can never tell them apart**."* The same comment block asserts both readings.

## ★ It is not only prose — a gate misdiagnoses

`accum_cost.rs:91-97` asserts `calls == 0` with the message *"a compiled condition is now EXECUTED
on this axis."* A `skip_span` bump would red that assertion while naming the wrong mechanism: the
elided arm reports itself as an execution. The number is right today and the **label** is wrong, so
nothing re-derives it — `[[a-right-number-vouches-for-a-wrong-label]]`.

## ⛔ THE ONE CONTRACT DECISION — and why C10 does not forbid it

`accum_cost.rs:85-90` carries an explicit prohibition:

> *"⛔ AND DO NOT SPLIT THE TWO DELTA ARMS (C10). Forcing `skip_span = false` at `fire/delta.rs` …
> Discriminating the arms is still a hot-path engine edit for an instrument's benefit."*

**That forbids changing which arm RUNS. This strike changes neither arm.** It changes the
`&'static str` each existing `census_count` call already passes:

- `compiled_cond.rs:959` → `census_count("compiled:exec")`
- `delta.rs:78` → `census_count("compiled:span-elided")`

Same two call sites, same call count, same branch structure, zero hot-path cost. C10's concern is
untouched; the discrimination it declined to buy with an engine edit is available for free in the
instrument, because **the two mechanisms were already at two different sites** — they were merely
sharing a name.

The old quantity is `exec + span-elided` for any consumer that wants pairs.

## Why splitting is the cure and renaming is not

Renaming to `compiled:match-attempts` would make the name true and leave the gate blind — it would
still be one number over two mechanisms, and `accum_alpha_cost.rs:1345`'s *"can never tell them
apart"* would remain the honest description. **One key carrying two mechanisms is why no name could
be right.** Splitting makes each name necessarily true and each gate discriminating.

## What the gates become — strictly stronger

`accum_alpha_cost.rs`'s C4/C14 probe drives both arms. Today: `assert_eq!(calls_built, calls_empty)`
— proves both bumps are alive, cannot say which arm took which path. After:

- `elided_built == exec_empty` — the same pairs, reached by the two different paths (still proves
  both sites alive: delete either and one side drops)
- `exec_built == 0` — the built arm executes nothing
- `elided_empty == 0` — the empty arm elides nothing

`accum_cost.rs`'s `calls == 0` becomes two assertions whose messages name the mechanism each
actually observes, and its exact-equality NAMES list holds both new keys absent on that axis.

## Protection already in place

`tests/lint/census_name_read_by_a_cost_test_is_emitted.rs` makes a read-but-never-emitted census
name a **build failure**, so a missed reader in this rename cannot silently become `unwrap_or(0)`.

## Out of scope = REJECTED

- Any engine behaviour change, including forcing `skip_span` (C10 stands).
- Census sections C–M. One section per strike.
- Re-pinning the deleted 80,200 under any name (`accum_cost.rs:99-102`).

## STOP triggers

1. Any measured value changes other than the census key names → STOP. This is a naming strike.
2. `exec_built == 0` or `elided_empty == 0` does not hold → STOP and report the counts. It would
   mean the arms are not what `:1345` says they are, and the DESIGN is wrong.
3. The split needs a new bump site anywhere → STOP. Two sites exist; this strike adds none.
