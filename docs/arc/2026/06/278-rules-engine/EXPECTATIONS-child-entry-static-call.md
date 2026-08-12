# EXPECTATIONS — `<fqdn>::child-entry` (written BEFORE the strike, so the result cannot move the goalposts)

Brief: `BRIEF-child-entry-static-call.md`. Design: `DESIGN-STONE-the-child-entry-kills-the-manifest.md`.
Baseline at draw time: **HEAD `310f8050`, floor 4391/4391 passed, 0 failed, 262 skipped, clippy 0.**

## Scorecard

| # | what | the command that checks it | expected |
|---|---|---|---|
| 1 | the two `apply` sites are gone | `grep -c "core::apply" wat/service.wat` | **drops by exactly 2**; the survivors are `Locus/launch`-side, not child-main |
| 2 | `child-entry` is emitted per service | `./target/release/wat --check <corpus service>` then reflect its fn set | `<fqdn>::child-entry` present |
| 3 | the shipped main is a ONE-LINER | read the emitted `child-main-form` | body is a single call to `~child-entry-name` |
| 4 | ★ **the manifest is DERIVED, not enumerated** | `fn-forms` over `child-entry` vs today's `service-forms` names | walk ⊇ manifest. **Any manifest name missing = STOP-3** |
| 5 | `service-forms-def` is deleted | `grep -c "service-forms" wat/service.wat` | **0** |
| 6 | stdlib load order intact | `:wat::deporder::verify-stdlib` | prints `[]` |
| 7 | the process tier still round-trips | `cargo test --release --test services -- probe_arc272_6b_defservice_on_process` | green |
| 8 | ★ **the thread tier is UNTOUCHED** | `cargo test --release --test services -- probe_arc209_c2_defservice_dispatch` | green, **and no edit to `wat/spawn.wat`** |
| 9 | the union gate still boots a child | `./target/release/wat wat-scripts/scratch-pad/probe-arc278-union-closure-boots-a-process-child.wat` | `VERDICT MEANINGFUL` |
| 10 | the peer wall holds | `cargo test --release --test services -- probe_arc293w` | both green, incl. the `.wat.bad` refusal |
| 11 | floor | `scripts/floor.sh`, read the Summary line | **≥ 4391 passed, 0 failed** |
| 12 | clippy | `cargo clippy --release --all-targets` | 0 |

Rows **4** and **8** are load-bearing. Row 4 is the entire point — if the walk does not cover the
manifest, the strike has not replaced anything and the manifest cannot be deleted. Row 8 is the
guard: this is a process-side change, and a moved thread-tier test means the blast radius escaped.

## Runtime prediction

**35–60 minutes.** Two `apply`→static rewrites are mechanical; the cost is the `child-entry`
extraction (moving a quasiquoted body into an emitted defn with correct hygiene) and the cascade
from every `defservice` recompiling. Time-box at **2×** = 2 hours.

## Trap-doors, named in advance

- **The locus parameter's type (STOP-1)** is the single most likely place this stops. The reference
  probe deliberately does not answer it — it takes the listener directly — so this is genuinely
  unproven and is why STOP-1 exists rather than a guess.
- **Hygiene (STOP-2).** Today's binders are `symbol-node`+unquote to dodge
  `ProgramBodyIntroducesName` in a `:user::main` body. In a `<fqdn>::` defn that pressure may
  vanish — or a *different* gate may fire. Either outcome is information; neither is a licence to
  improvise.
- **A wide first-build cascade is EXPECTED**, not a crisis — every `defservice` recompiles. Read
  the errors as the worklist.
- **The `:derived`-style silent-omission risk inverts.** Today a forgotten manifest entry fails at
  child startup, far from the cause. After, an unreached callee fails at the walk — but only if
  row 4 is actually diffed. **Do not accept "the tests pass" in place of row 4**; the existing
  tests did not catch the manifest's omissions either (that is what opened this thread).

## How this gets scored

The orchestrator re-runs every row itself — the rider's report is a hypothesis until a current
`file:line` or a re-run confirms it. Floor is weighed centrally, once, after the tree is quiescent.
A red is a red: capture the failing test's whole stdout+stderr verbatim, name the exact arm, do not
re-run before reporting.
