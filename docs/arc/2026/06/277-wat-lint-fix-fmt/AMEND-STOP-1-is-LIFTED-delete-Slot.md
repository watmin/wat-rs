# AMEND — mid-strike. **STOP-1 IS LIFTED. Delete `Slot`.**

> **Builder:** *"delete Slot - a design with no consumer is unfalsifiable"*

The BRIEF's **STOP-1 said "do NOT delete `Slot`"** because the deletion was a second contract
decision and deletions clear a high bar here. **The builder has ruled it. STOP-1 no longer applies.**

## SEQUENCE — this matters, do not invert it

```
1  land the lexical `->` glue
2  PROVE row 2 — a generic fn renders `-> :wat::core::i64` BOTH TOKENS ON ONE LINE
3  THEN delete Slot
4  PROVE row 2 again, plus rows 3/4/5 (non-generic fn, defmacro, defn)
```

⛔ **Deleting first breaks every ret-spec in between.** Prove the replacement carries the load
before removing the load-bearer.

## WHAT GOES

`Slot` has ~20 mentions in `wat/fmt.wat` and ~15 across `wat-scripts/fmt/`. All of it:

```
the :wat::fmt::Slot record
its builder (the registry walk + read-string of Row/syntax + the arrow scan)
its map, its query, its use in R11's :when
any driver line that exists only to load or print it
```

## ⛔ WHAT MUST **NOT** BE LOST — the deletion removes the code, not the knowledge

- **`[[NOTE-the-registry-already-knows-the-slots]]`** — the four measurements. It stays, and it is
  now the *only* record that the grammars parse, that `Row/name` is the DOT form while the corpus is
  the COLON form, and that the refusal discipline works. **Do not touch it.**
- **The three scratch-pad probes** — `277-does-the-registry-know-slots.wat`,
  `277-can-wat-read-its-own-grammar.wat`, `277-locate-the-slot-in-a-grammar.wat`. They are
  loader-gated and durable; anyone can re-run the measurement in one command. **They stay.**

★ **That is why this deletion is safe and the "keep it for later" argument was weak.** The knowledge
is in a NOTE and three runnable probes; only the unused machinery goes. If a future case needs a
grammar-derived fact, it is a morning's work from probes that still run — not a rediscovery.

## THE REVISED ROW 6

```
BEFORE  6  Slot's consumer count REPORTED — expected 0. Do not delete it.
AFTER   6  ★ Slot is GONE:  grep -c Slot wat/fmt.wat wat-scripts/fmt/**  ->  0 everywhere
```

Every other row is unchanged, and **rows 3/4/5 matter more now than before** — they are the proof
that removing `Slot` cost nothing, since `fn`, `rete::fn` and `defmacro` were its only clients.

## AND IF THE DELETION BREAKS SOMETHING

**STOP and report it.** A ret-spec that regresses when `Slot` is removed means the lexical glue did
not actually cover a case `Slot` was covering — **that is a finding about the replacement**, and it
is worth far more than a green achieved by putting `Slot` back.
