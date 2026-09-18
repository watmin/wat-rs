# DESIGN — Tier B: the check skips what it already proved

**Drawn 2026-09-18**, builder-ruled: *"B"*, after *"B is our target after defclause is made correct"* —
and `defclause` is now correct (`38b43a6ac`). **NOT STRUCK.**

## The prize, measured after Tier A

```
accounted boot now                     191.30 ms
  check:body-infer(ALL fns)            109.64 ms   57.3 %
  check:retired-syntax(ALL fns)         11.26
  check:restricted-call(ALL fns)         3.43
  check:def-position(ALL fns)            2.16
  ──────────────────────────────────────────────
  the four sweeps                      ~126.5 ms  ≈ 66 % of what REMAINS
                                        boot ≈ 0.22 s → ≈ 0.09 s
```

Every `stdlib-*` phase already reads **0 hits** — Tier A works, and what is left is dominated by the
checker re-proving, on every boot, facts about code that has not changed.

## ⛔⛔ PHASE 1 IS A PROOF, AND IT GATES EVERYTHING

The claim Tier B rests on:

> **For every stdlib function `F`, the verdict of `check(F)` is determined solely by state fixed at bake
> time — no user-supplied state can reach it.**

⚠ **The spike's evidence for this is EMPIRICAL and its bound was never lifted**: it enumerated the names
probed during *one* program's stdlib sweep and found zero user-declarable ones. A different user program
could drive stdlib inference down a path probing something else. **A census cannot close this. A proof
can**, and the builder's namespace ruling is what makes one possible:

> every name a stdlib body can resolve is either `:wat::` — **reserved and unforgeable** — or a local
> binding.

**Prove or refute that, by reading the seven doors the spike instrumented** (`src/spike_probe.rs` records
them) and the reservation guard (`resolve/registration.rs:120`, which reaches `Reserved` only under
`Privilege::User`). ⛔ **If any door can consult user-supplied state for a `:wat::` name, that is the
finding and Tier B stops there.**

### Known routes that must each be closed, and why each is suspicious

| route | status going in |
|---|---|
| user `defn` / `def` | `defined_values` mutated at `check.rs:817` before body-infer at `:844` — the spike found this route **closed in this corpus**; prove it closed **by construction** |
| user `defclause` | walled at check time (`38b43a6ac`) — ⛔ **but eval-time/REPL `defclause` does NOT pass through `check_program`**, which that stone disclosed. Does the reservation still refuse it? |
| user macros | Tier A's witness proved user macros register **before** stdlib expansion; expansion is now cached, but say why that cannot change a **verdict** |
| user types / `extend-type` | measured: `extend_regs` probed **zero** times during the stdlib sweep — confirm structurally |
| acronyms / `defclause` stubs / companions | the `defclause` stone found companions minted (or *declined*) around the same registries |
| `installed_dep_sources()` | the boot-cache key consults it at **runtime**; can a dep contribute a name? |

## Phase 2 — the elision, only if phase 1 proves the closure

Restrict the four `ALL fns` sweeps to functions the snapshot did **not** supply. The first boot derives
**and checks**; later boots skip what the snapshot already proved.

⭐ **The control that matters is not that boot got faster — it is that errors are still found.** The
sweeps must still run over **user** functions, in full. A Tier B that also skips a user error is strictly
worse than a slow boot, and it would be invisible.

## Trap-doors

1. ⛔ **Prove that the sweeps are PURE.** The spike measured `check_program(forms, sym, types) ->
   Result<(), CheckErrors>` with immutable borrows and **zero** `RefCell`/`Mutex`/`RwLock`/`Cell`/`unsafe`
   in `src/check.rs`. Re-confirm — if a sweep has a side effect the elision drops it silently.
2. ⛔ **A stale, absent or corrupt snapshot must derive AND check** — everything, as today. Drive all
   three, as the Tier A stone did.
3. ⛔ **Eval-time / REPL `defclause` bypasses `check_program`** (disclosed by `38b43a6ac`). Establish
   whether that can reach a stdlib name; the reservation should refuse it, but **drive it**.
4. **Do not touch** `check.rs:6068`'s dispatch precedence, `env.rs:456`'s unconditional insert, or
   `runtime.rs:2112`'s `Existing::Equivalent` — all named open items from the previous stone.
5. **A green floor proves little on its own** — the corpus may contain no program that would expose a
   dropped error. Argue from the path and drive the negative control.

## Out of scope

The `Existing::Equivalent` codegen repair · the REPL check gap as a *fix* (establish reach only) · the
queue promotion (unblocked by Tier A; its own re-weigh).
