# BRIEF — STONE ①: the rete vocabulary becomes registry rows

## The work, in one paragraph

`RETE_OPS` (`src/rete/vocabulary.rs:247`) is a 74-row table of real verbs, each carrying its own
name, params, return type and purity metadata. **Zero of the 74 are registry rows.** Fold the table
into `registry()` as a fourth stream, deriving each `IntrinsicEntry` from the row it already has.

## Why it is a registration and not a deletion

The arc measured this and wrote the epitaph —
`NOTE-rete-ops-is-a-population-missing-from-the-registry-not-an-authority-over-it.md`:

> *"A rete row is a **DIFFERENT VERB** from its core twin, usually a **TOTALIZED** one.
> `:wat::rete::i64::/` is TOTAL — `(… 1 0 :undefined -1)` returns `-1` — where core `:wat::i64::/`
> is PARTIAL."*

These are not aliases and not typos. They are the vocabulary a `where` body is written in, and
`Boundary::MakeRule` deliberately resolves those bodies as code.

## Read in order

1. `docs/.../NOTE-rete-ops-is-a-population-missing-from-the-registry-not-an-authority-over-it.md`
   — the measurement, including why the 17 apparent meta "disagreements" are not disagreements.
2. `src/rete/vocabulary.rs:247` `RETE_OPS` — the table. A row carries `rete_name`, `core_name`,
   `class`, `params`, `ret`, `meta { pure, deterministic, total }`, `type_params`.
3. `src/intrinsic/mod.rs:586` `registry()` — it already folds TWO inventory streams and its own
   comments call one of them "the THIRD stream". You are adding a fourth source; unlike the others
   it is a `const` table, so iterate it directly — no `inventory` submission needed.
4. `src/intrinsic/mod.rs:529` `IntrinsicRegistry` / `IntrinsicEntry` — note `handler` is an
   `Option`. A rete op has **no per-op handler**: it dispatches by `class` through
   `dispatch_rete_op`. `handler: None` is correct and is the same shape special forms use.

## Implementation sketch

In `registry()`, after the existing streams:

```
for op in crate::rete::vocabulary::RETE_OPS {
    r.register(IntrinsicEntry { /* derived from op's own fields */ });
}
```

⛔ **DERIVE, never transcribe.** Every value comes from the row. If a field the entry needs has no
counterpart on `ReteOp`, **STOP and report which** — a hand-written literal beside a table that
already holds the truth is how the registry drifts, and it is the defect this arc exists to remove.

## Acceptance

```
cargo nextest run --release -E 'test(p1_annotation)'  10 · 'test(a1_one_rule)' 4 · 'test(a2_a_variant)' 15
cargo clippy --release --all-targets -- -D warnings   EXIT 0
```

Plus: state in the SCORE how many rows registered and whether any row could not produce an entry.

⚠ **You do NOT run the corpus census.** I run it — that is the measurement half of the tier rule,
and handing a rider a corpus sweep is a mistake I made once today already.

## STOP triggers — each is a REJECTION

**STOP-1.** If an `IntrinsicEntry` field has no counterpart on `ReteOp` — STOP and report which.
Do not invent a default.

**STOP-2.** If registering a rete row collides with an existing registry entry — STOP and report
the name. A rete verb and its core twin are DIFFERENT verbs with different FQDNs; a collision means
something else is wrong.

**STOP-3.** If any earlier stone's filter moves — STOP with the verbatim block.

**STOP-4.** If this requires touching `dispatch_rete_op` or the routing — STOP. This stone adds
rows; it changes no behaviour. A registration that alters dispatch is not a registration.

## Tier

You edit and report. Run the three targeted `-E` filters and clippy. **Do NOT run
`scripts/floor.sh`, an unfiltered `nextest`, or any corpus-wide `--check` sweep.**

⛔ **Do NOT background a long-running command and end your turn** — ending your turn ENDS you and
nothing will wake you. Run what you need in the foreground. Everything asked for here is seconds,
not minutes.

⛔ **Do NOT call `pulsare_yield`, `pulsare_knock`, or contact any peer.** Report to me only.
