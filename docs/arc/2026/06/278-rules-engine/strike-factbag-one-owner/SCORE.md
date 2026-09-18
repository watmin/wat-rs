# SCORE — FactBag owns the fact base; retract is still remove-every-equal

Behaviour-neutral. One owner. Five native doors became two. Retract's fold is unchanged. Floor GREEN. No cure.

## Scorecard

| # | result |
|---|---|
| 1 ★ behaviour-neutral | **HOLD on the engine.** Final floor `5465 passed, 21 skipped` — HEAD's 5460 plus **5 new lint tests** (4 detector + 1 live). The DESIGN required the gate; the EXPECTATIONS used 5460 as a proxy for "no value moved". Negation three-way still `clara=25 native=25 oracle=25 ALL THREE MATCH`. |
| 2 ★ one owner in wat | **HOLD in code.** `Session/facts` and `FactBag/items` as *forms* live only in `wat/rete/factbag.wat` (`of` and `items`). `query.wat:127,199` still *name* them in comments; the gate strips `;;`. |
| 3 ★ two doors in Rust | **HOLD.** `grep -rn '"facts"' src/rete/` → `session.rs:1483` (`session_facts`) and `session.rs:1547` (`session_with_facts`). |
| 4 ★ gate mutation-proved | **HOLD.** Three live REDs quoted below. Restored. |
| 5 ★ wrap codemod | **HOLD.** `wrap-session-facts-in-factbag.wat`. Dry-run wrapped only Session ctor `:facts` and `Session/facts` reads; FireStratAcc untouched. Re-run on `insert.wat` after the second pass: **0 changes**. |
| 6 retract untouched | **HOLD.** `remove-every-equal` is the same foldl `not (= f fact)` then conj. `retract` is now that door. Docstring's "symmetric with insert" left as-is. |
| 7 grid | **HOLD on a live axis.** `check-grid-three-way.sh negation` AGREED. The parked retract-multiplicity axis is still MISMATCH; that is strike 2's redraw, not this migration. |
| 8 clippy | **HOLD.** `cargo clippy --all-targets --release -- -D warnings` rc=0. |

★ load-bearing. **Row 1 is the whole claim: a migration that changes a value is a failed migration.**

## Row 4 — three live REDs

Driven `no_raw_factbag_access_outside_the_owner`, one banned form at a real site, restore after each.

**Rule 1 — `FactBag/items` in `insert.wat:9`:**

```
raw FactBag access outside wat/rete/factbag.wat:
  wat/rete/oracle/insert.wat:9: raw `FactBag/items` (only wat/rete/factbag.wat may unwrap)
```

**Rule 2 — `Session/facts` in `insert.wat:9`:**

```
raw FactBag access outside wat/rete/factbag.wat:
  wat/rete/oracle/insert.wat:9: raw `Session/facts` (only wat/rete/factbag.wat may read the field)
```

**Rule 3 — `"facts"` in `insert.rs:10`:**

```
raw `"facts"` outside src/rete/kernel/session.rs's two doors:
  src/rete/kernel/insert.rs:10
```

## Load order (honest delta)

`FactBag` the **record** is declared in `wat/rete.wat` immediately before `Session`, so the field can name the type. `factbag.wat` loads after and holds every **door** including `of` (the one `Session/facts` call). DESIGN wanted both in `factbag.wat`; a file cannot both precede Session (for the type) and follow it (for `of`). The owner of multiplicity is the door file.

## Native contract

`session_facts` unwraps FactBag → inner PVec. `session_with_facts` wraps PVec → FactBag. `to_transient` reads through `session_facts`; `to_persistent` writes through `wrap_factbag`. `insert.rs`'s third door and `rules.rs`'s raw `"facts"` writes are gone. Fire engine `wm.facts` is still a PVec.

## First floor (captured, not re-run)

`.floor/2026-09-06T08-36-56Z/`: `5463 passed, 2 failed`. Arms:

1. `every_walking_gate_declares_non_vacuity` — the new gate lacked a `NON-VACUITY:` comment (the asserts were there; the magic word was not).
2. `every_rete_name_in_wat_scripts_code_resolves` — wrap file used `starts-with? ":wat::rete::factbag::"`; the extractor read that prefix as a rete name. Replaced with exact door heads.

## Final floor

`.floor/2026-09-06T08-48-04Z/`: `Summary [ 460.058s] 5465 tests run: 5465 passed (2 slow), 21 skipped`.

## Still open

Strike 2: `remove-one` + the redrawn retract-multiplicity axis (duplicate only the retracted key) + TMS fuzzer model from Clara. `remove-every-equal` stays until then.
