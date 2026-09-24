# BRIEF — STONE 255.16: one type binds a parametric surface once

**Drawn 2026-09-24 against `main` @ `58d7e0ad3`.** Floor 6023/6023, clippy 0, census `no STOP-8`, delta
**3 / RECOVERY 0**, ledger **220**. **Executor tier: Opus.**

## Why — the transport invariant can be broken today

255.15 found, and the orchestrator reproduced on both the pre- and post-255.15 binaries, a **runtime type
lie** (`WEIGH-STONE-255.15-…`, §"The hole it found"):

```wat
(:wat::core::extend-type :probe::Both (:probe::Loc :- [:probe::Shared]) (transport [self] (:probe::Shared :a 1)))
(:wat::core::extend-type :probe::Both (:probe::Loc :- [:probe::Wire]))            ; bodiless — accepted
(:wat::core::defn :probe::sc [loc <- (:probe::Loc :- [:probe::Wire])] -> :probe::Wire (:probe::Loc/transport loc))
(:probe::sc (:probe::Both :q 1))
```

`--check` **rc=0**; run **rc=2** — *"expected receiver of class `:probe::Wire`, got class
`:probe::Shared`"*. Dispatch keys on the flat `<Type>/<method>`, so the one body serves both bindings.
⛔ **A `Shared` transport passes as `Wire` — a route for a process to hold a shared-memory address**, the
rule `a6da457e3` made a type error and the narrow-waist ruling (SEAM) depends on.

## The work — make it unrepresentable

**At registration, refuse a second binding of one parametric surface on one type** —
`(extend-type T (S :- [A]))` followed by `(extend-type T (S :- [B]))` where `A ≠ B`. Refuse regardless of
whether either carries a body: the hole's form is one with a body and one without, and wat's flat dispatch
means two bindings can never both be honest. Re-registering the **identical** binding (255.15 skips it via
`type_exprs_same`; door-replace re-registers) must stay legal.

**The diagnostic must name both bindings and the type**, e.g. *"`:probe::Both` already binds
`(:probe::Loc :- [:probe::Shared])`; a type binds a parametric surface once"*.

## Measured by the orchestrator before drawing

- ⭐ **No legitimate code binds one parametric surface on one type twice.** Per-file census over every
  tracked `.wat`/`.wat.bad`: the **only** instance is `tests/types/probe_arc255_15_infer_transport_ambiguous.wat.bad`
  — 255.15's own negative fixture. (A cross-FILE double binding through loads is not covered by that
  census; the floor is the gate for it.)
- ⚠ That fixture's test asserts inference's `TypeMismatch`. After this stone it is refused **earlier, at
  registration**, with the new diagnostic. **Update its assertion to the new error — do not delete the
  fixture;** it becomes this stone's own negative row.

## Rooms — located by 255.15's executor, verify them

- `src/types.rs` — `splice_type_decls`' `extend-type` handling, and 255.15's
  `register_parametric_extension(child, target, span)` (the one door that records a parametric binding;
  it already keeps the parsed target in `TypeEnv::parametric_extensions`). **The check belongs at that
  door** — it already sees every parametric binding a type makes.
- `src/check.rs` `assignable`'s arc-170 Gap-1 arm — 255.15's ambiguity refusal (`solutions.len() == 1`).

⛔ **Stale citations — do NOT trust them:** `src/check.rs:17374` and `:17473` say *"types.rs:2151 stores the
extend-type target keyword VERBATIM"*. `types.rs:2151` is `Option` prose. **Fix both comments** to name
the real site — they misled the orchestrator's last brief and will mislead the next reader.

## Soundness rows

1. ⭐ The hole's program (above) is **refused at registration**, naming both bindings.
2. ✅ One binding per type still works — every 255.15 positive fixture unchanged.
3. ✅ Re-registering the **identical** binding stays legal (door-replace; duplicate skip).
4. ✅ **Different types** binding the same surface differently stays legal (`Th` → `Shared`, `Pr` → `Wire`)
   — that is the whole point of the surface.
5. ⛔ Order-independence: bodiless-first then bodied is refused the same way as bodied-first.
6. ⚠ **Measure, do not assume:** after this stone, can 255.15's `solutions.len() > 1` (ambiguity) still be
   reached any other way? Report it. If it is now unreachable, say so — **do not remove it in this stone**;
   removing a guard is its own decision.

## STOP triggers

1. If any **legitimate** program (the floor, the census) is refused by the new rule — **STOP and report
   it**; that is a case the census missed and needs a ruling, not a workaround.
2. If refusing at registration needs the check to run somewhere that does not see every binding a type
   makes (e.g. bindings arriving from a loaded file after the check) — **STOP, report the shape.**

## Expectations — fixed before the strike

| what | command | expected |
|---|---|---|
| the hole is refused | `--check` the program above | rc=1, new diagnostic naming both bindings |
| order-independence | bodiless first, then bodied | rc=1, same diagnostic |
| identical re-registration | same binding twice | rc=0 |
| different types, different bindings | `Th`/`Pr` | rc=0 |
| 255.15 fixtures | `cargo nextest run --release -E 'test(/arc255_15/)'` | 9/9, the ambiguity row's assertion updated |
| floor · clippy · census · delta | the usual | green · 0 fresh · `no STOP-8` · NEW 3 / RECOVERY 0 |

Runtime prediction: 1–2 hours.

## Doctrine

- ⛔ **THERE IS NO KNOWN FLAKE. A RED IS A RED.** Capture whole, never re-run to green, name the arm.
  ⚠ Bookkeeping exception: `harvest_wrap_split` — **cite and report, never a pass, never re-run to
  clear. Name it either way.**
- ⭐ Prove every probe can say BOTH words; strip ANSI from live output; carry a control each run.
- ⛔ This stone **refuses more** — the safe direction — but a refusal can still catch a legitimate case.
  The census and the floor are that check; STOP-1 is what you do if they fire.
- **If this brief contradicts the code, the code wins — say so plainly.** Commit locally; **do not push.**

## Out of scope

The `derive`-chain hole (satisfies statically, no methods at run time — a different mechanism, its own
stone) · the transport registry (step 2) · the generic `start` (step 3) · the message alias.
