# DESIGN — `conferre` L2-3: the two stratifiers disagree, and nothing in the tree compares them

## Why

`stratify.rs:205` says, in its own header: *"Mirrors `stratify-sweep`
(`wat/rete/oracle/stratify.wat`)."* **It does not.** The native sweep computes a term the
oracle's sweep has no counterpart for:

```rust
// src/rete/kernel/stratify.rs:221-227  — exists / acc :from of a type THIS SET derives: +1
for b in &view.exists_and_from_types {
    let derived = rule_parts.iter().any(|other| { … });
    let v = *type_strata.get(b).unwrap_or(&0) + i64::from(derived);   // ← +1 when derived
```

The oracle folds `:exists` inner and accumulate `:from` into **`rule-consumes`** instead
(`stratify.wat:157-158`), and `req-pos` is explicitly **NOT +1** (`:233-234`): *"a positive consumer
may sit in the SAME stratum as its input."*

## ⛔ RETRACTED — THE FIRST MEASUREMENT READ A STDLIB FIVE MONTHS OLD

The numbers below were taken through the `wat` MCP and are **INVALID**. The MCP runs
`/home/john/.cargo/bin/wat --mcp`, and that binary carries the `wat/` stdlib **`include_str!`'d at
build time**. The installed binary was dated **2026-08-23 14:10**; two direct checks, not
inferences from an mtime:

- `grep -c 'THIS FILE USED TO CLAIM' ~/.cargo/bin/wat` → **0**. The header block added to
  `stratify.wat` on 2026-08-31 (`16f504e14`) is not in the binary.
- `git diff 2aa14fa1c HEAD -- wat/rete/oracle/stratify.wat` → **335 insertions, 0 deletions**.
  At the last commit before that build, **the file did not exist**; the oracle stratify the MCP
  answered from is a pre-split ancestor under the same name.

So `:wat::rete::stratify` resolved, ran, and returned a plausible, correctly-anchored answer **from
the wrong implementation**. ⭐ The anchor did its job and could not have caught this: it proves the
verb DISCRIMINATES, never that it is the verb at HEAD. **A live-looking process is a build
artifact.** Refresh with `cargo install --path . --force` from `wat-rs/`, then kill the running
`wat --mcp` processes so they respawn on the new binary.

`[[a-cast-report-is-dated-drive-it-before-rowing-it]]` · `[[drive-wat-directly-not-through-cargo]]`
(whose ⛔ half says exactly this: `wat/` is `include_str!`'d and needs a rebuild).

## MEASURED — orchestrator, 2026-09-07, oracle side, on a VERIFIED-HEAD binary

Re-taken after `cargo install --path . --force` (binary 2026-09-07 00:05; `strings -a` finds the
2026-08-31 `stratify.wat` header in it — 1 hit — against an anchor of 4 `stratify-sweep` hits, so
the check itself discriminates). Instrument committed as
`wat-scripts/scratch-pad/arc278-l2-3-stratify-numbers.wat` — a number whose instrument is not in
the tree cannot be rechecked (`[[a-metric-without-its-instrument-cannot-be-rechecked]]`).

Rule set: `A` inserted · `ok :- A ⇒ Ok` (so `Ok` is **derived**) · plus one of:

| shape, over the derived `Ok` | `(:wat::rete::stratify …)` |
|---|---|
| `(:wat::rete::not (:l23::Ok …))` — the **ANCHOR** | `{"l23::Ok2" 1}` |
| `(?n <- (:wat::rete::acc::count) :from (:l23::Ok))` | `{}` |

Both engines record only *raised* strata (`stratify.rs:238-241`, `if required > cur`), so `{}`
means **every produced type is stratum 0** — the oracle gives `Tally` stratum **0**. The anchor is
what makes that readable: negation is the term the two engines *agree* on (+1 both sides), so its
non-empty map proves the verb discriminates.

⚠ **The re-measurement returned the same numbers as the retracted one.** That means the sweep's
behaviour did not change in this respect — it does **not** retroactively make the first reading
sound, and it must not be cited as though the staleness were harmless.

**By reading** `stratify.rs:222-227`, the native gives that same rule `Tally` stratum **1**
(`derived` is true — `ok` produces `Ok` and does not bag it). ⚠ **That half is a READING, not a
drive** — it is the first thing this strike must falsify
(`[[a-reading-cannot-see-an-execution-defect]]`).

Minimal rule set: `A` inserted · `ok :- A ⇒ Ok` (so `Ok` is **derived**) · `tally :- Seed,
(acc::count) :from Ok ⇒ Tally`.

| shape, over the derived `Ok` | `(:wat::rete::stratify …)` returns |
|---|---|
| `(:wat::rete::not (:st::Ok …))` — the **anchor** | `{"st::Ok2" 1}` |
| `(?n <- (:wat::rete::acc::count) :from (:st::Ok))` | `{}` |

`stratify-sweep` only records *raised* strata, so `{}` means **every produced type is stratum 0** —
the oracle gives `Tally` stratum **0**. The negation anchor is what makes that readable: it proves
the verb can emit a non-zero row, so `{}` is a measurement and not an inert call
(`[[anchor-a-new-measurement-before-trusting-it]]`).

**By reading** `stratify.rs:222-227`, the native gives that same rule `Tally` stratum **1**
(`derived` is true — `ok` produces `Ok` and does not bag it). ⚠ **That half is a READING, not a
drive** — it is the first thing this strike must falsify, because a reading cannot see an execution
defect (`[[a-reading-cannot-see-an-execution-defect]]`).

## ⛔ What this strike is NOT allowed to assume — the premise that already died

The obvious framing — *"no fixture exercises exists/acc-`:from` over a self-derived type"* — is
**FALSE and was refuted during the crawl.** `tests/rete/probe_arc278_derived_exists_acc.wat` is
exactly that shape (`:exists (Ok)` **and** `acc::count :from (Ok)`, `Ok` derived), and
`probe_arc278_derived_exists_acc.rs:40` asserts
`assert_eq!(nat, spec, "oracle must match native on derived exists/acc")` — **green on the floor
today**.

So the behavioural differential over the divergent term EXISTS and PASSES. Two readings survive
that, and this strike exists to decide between them:

1. **The strata differ but the facts do not** — the oracle reaches the same answer by a different
   route (`fire-support-fixpoint`, the supersession half added 2026-08-31: *"Stratification is still
   the ordering layer; it was never the supersession layer."*). Then L2-3 is **not** a correctness
   defect, and what is defective is the **false lockstep claim** in both headers.
2. **The strata differ and some fixture CAN make it observable** — and `derived_exists_acc` is
   simply under-powered, in the same way F2's grid fixtures staged no duplicate. The oracle's own
   header names the shape that would do it: an accumulate whose source **grows** mid-fixpoint
   (`(acc::count :from Out)` over an `Out` that goes 0→1→2).

## The one contract decision

**The instrument compares STRATUM NUMBERS, not derived facts.** Every existing differential
(`probe_arc278_7strat_native_differential`, the grid's `strat-neg` axis, `derived_exists_acc`)
compares query row counts. Counting facts is precisely what cannot see this
(`[[a-count-cannot-see-a-value-defect]]`): both engines can agree on every fact while assigning
different strata, and that is reading 1 above.

## Files

- `src/rete/kernel/stratify.rs` — the native sweep; **read only** in this strike.
- `wat/rete/oracle/stratify.wat` — the oracle sweep; **read only**.
- New: a test that drives both stratifiers on one rule set and compares the maps.

## Out of scope = REJECTED

- **Any cure.** No `+1` is added or removed on either side by this strike. The row's own report
  forbids a cure before a measurement, and after the measurement the cure may be a doc fix.
- **Clara.** Not needed yet: this is an internal native-vs-oracle question. Clara becomes the
  referee only if reading 2 holds and the two engines differ on FACTS.
- Touching `derived_exists_acc` — it is green and it is evidence.
