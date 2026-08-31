# BRIEF — the gate must tell a declared red from a rotted one

Add a gate that walks every `.wat` under `docs/arc/` and requires each to load on the current
runtime OR declare, in a closed-vocabulary rune, why it does not. Read `DESIGN.md` beside this file
first — its ★ ONE CONTRACT DECISION is the whole point, and the cheap way to make this gate green
violates it.

## Read in order, and why

1. **`tests/lint/wat_scripts_fixes_load.rs`** — whole file. **This is the gate you are modelling
   on**, down to `startup_from_source` + `FsLoader` (its header explains why `InMemoryLoader`
   would make it lie about relative `load-file!`). Its own doctrine sentence — *"ALL wat must
   remain correct, always"* — is what `docs/arc/**` is currently exempt from.
2. **`tests/lint/mod.rs`** — three lines. `build.rs` generates the module list from sibling `.rs`;
   **you do not register anything.** Drop the file in.
3. **`tests/lint/no_ceiling_raise_in_rete.rs:92`** — the non-vacuity guard, with its reason written
   out. `complectens` found 10 of 15 walking gates lack one. Yours has one.
4. **`docs/arc/2026/06/278-rules-engine/probes/red-owner-signals-child.wat`, header** — the prose
   that already declares a red-by-design file. You are formalising exactly this into a rune.
5. **`docs/arc/2026/05/130-cache-services-pair-by-index/complected-2026-05-02/README.md`** — read
   it before you touch either `.wat` beside it. It is why the `historical` category exists.

## The dispositions — all 10 driven at HEAD `819c79b9a`, none guessed

| file | do this |
|---|---|
| `probes/surface-field-dispatch.wat` | **MIGRATE**: `:holder` → `:nature`. Verified: it then prints **142**, which its header promises. Do not mark it. |
| `probes/red-owner-signals-child.wat` | mark `red-by-design` — its header already says why, in prose |
| `harness-experiri/experiri-acc-head.wat` | mark `red-by-design` — the A3 repro; the refusal is the proof |
| `harness-experiri/experiri-then-match.wat` | mark `red-by-design` — the D5 repro; ditto |
| `130-…/complected-2026-05-02/substrate.wat` | mark `historical` — quote its README |
| `130-…/complected-2026-05-02/test.wat` | mark `historical` — ditto |
| the remaining 4 | nothing — they load |

## The order

1. **Write the gate first and run it BEFORE any migration or marking.** It must go RED, and it must
   name **three** rotted-or-undeclared files plus the two historical ones — five reds, from a walk
   of ten. Quote that output verbatim; it is the proof the gate sees real rot.
2. Then migrate the one, mark the five.
3. Re-run: GREEN.
4. Mutation-prove: strip the rune from one marked file → that file alone reddens. Restore. Then
   revert the `:nature` migration → `surface-field-dispatch` alone reddens. Restore.

## STOP triggers

1. **If you are tempted to mark `surface-field-dispatch.wat` instead of migrating it, STOP.** That
   is the ★ decision failing. It is rot, not design, and the migration is one keyword.
2. **If a `historical` file looks migratable, STOP and leave it.** Migrating it destroys the record
   it exists to be, on a builder instruction quoted in its README.
3. **If the walk finds a `.wat` not in the table above, STOP and surface it** — the table is HEAD
   `819c79b9a`'s state, and a new one is a finding.
4. **If you find yourself widening the rune vocabulary past the two categories, STOP.** A third
   category needs its own discriminating question, and inventing one silently is how
   `rune:purgare`'s vocabulary became undefined (`excusare`, 2026-08-30).

## What "declared" must mean

A rune whose reason is *"it fails"* is not a reason. `red-by-design` must name **what the failure
proves**; `historical` must name **what past state is preserved**. A reader must be able to check
the sentence against the file's own behaviour — that is the difference between this gate and a
suppression list.
