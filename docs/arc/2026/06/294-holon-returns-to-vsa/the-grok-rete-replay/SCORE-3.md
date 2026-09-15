# SCORE 3 — every spawned program starts: the nested-program gate

Branch: `replay/grok-rete`. **Committed, not pushed.** Main untouched.
Parent: `1d2c88b3d` BRIEF 3. Finding 8.

```
floor  scripts/floor.sh   .floor/2026-09-15T00-51-44Z
       Summary [ 223.359s] 5502 tests run: 5502 passed, 22 skipped   exit=0
clippy cargo clippy --release --all-targets -- -D warnings           CLIPPY_RC=0
```

5498 + 4 = 5502: the tree gate, E1 RED fixture, E5 missing-rune test, `deftest_wat_tests_process_child_is_fresh_universe`.

Two earlier floors were RED, captured, not re-run:

- `.floor/2026-09-15T00-13-39Z` — 3 lint walls + gate TIMEOUT at 30.012s (STOP-4).
- `.floor/2026-09-15T00-42-42Z` — `probe-child-inherits-defns.wat` unexpected `)` (rune splice closed `forms`); `replace("::", "_")` on a deftest path.

## EXPECTATIONS

| # | result |
|---|---|
| E1 | **RED as required.** Temp fixture of `f2e0ac26b^:…/probe-m1-ann-erase.wat`. Child at **line 34**. `assert_startup_error!(…, check MalformedForm { head == ":wat::core::match", reason == "arm #1: variant arm head \`:probe::CMsg::Setup\` is not namespaced; write \`<enum>.<Variant>\`" })`. File is `/tmp/nested-program-erase-old.wat`. |
| E2 | **PASS.** Isolated `nested_program_literals_start_on_the_child_path` green. Floor 63.313s loaded. |
| E3 | **142 = 141 census + 1 stone literal.** Isolated: `checked=123 assembled=7 templates=5 data=7 wall_ms=29107`. 123 = 121 `spawn-peer` + 1 `Locus/launch` + 1 `wat-tests/process/child-is-fresh-universe.wat`. A let-binding RHS of an assembled concat is walked twice; `collapse_hits` keeps Assembled over Data (`wat/spawn.wat:570`). |
| E4 | **derived, no verb-name list.** One root `(":wat::kernel::spawn-program", 1)`. A stdlib param is carrying iff it reaches a carrying position unchanged, as a `concat` operand, or through a `let` binding. Derived today: `spawn-peer` idx 1, `spawn-hermetic-program` idx 0, `Locus/launch` idx 4. Thread tier takes a fn, never forms. |
| E5 | **RED.** `nested_program_gate_refuses_a_rune_whose_test_does_not_exist` splices `test(no_such_nested_program_gate_test)` onto the old-erase child; the name is not in `live_tests()`. |
| E6 | **children START** (gate rows for the seven are empty). Probe **runs** below. |
| E7 | **GREEN.** Fixtures `replay/wrap-nested-forms-{recv-call,recv-in-recvoutcome,sendoutcome-stopped}/`. Shards 10/11/12 + `every_recorded_migration_is_fixtured_or_runed` PASS. Nested `(forms …)` child. Second run 0 changes on the fixtures. |
| E8 | **migrations only** on the seven. `git diff` is wrap-in-place (equal ins/del). No parent rewrite: tools walk the tree but emit edits **only inside** `(:wat::core::forms …)`. |
| E9 | **D3 classified and fixed.** Finding 8 table: leftover of “child is a fresh universe” (same class as `probe-child-inherits-defns.wat`). Unlike that probe, arc 112’s *purpose* is Process phantom-type unification (`arc112_scheme_probe.rs`), not the failure. The unresolved `:my::worker` hid the claim. Defect in the probe. Fix: ship `:my::worker` into the child. `arc112_probe_spawn_program_parametric_return` PASS. |
| E10 | **green.** Clippy 0. Isolated gate 29.1s / 123 startups. Loaded 63.3s (2.2×). nextest override: warn 90s / kill 180s, **above** `binary_id(wat::lint)`. |

## D1 — the gate

`tests/lint/nested_program_starts.rs`. Child path is `InMemoryLoader` + `startup_from_forms_with_inherit(forms, None, loader, parent Config)` — not `--check`. `start_child` returns `Result<(), StartupError>` (no String flatten). Expected failures are pinned by a co-located `rune:lint(nested-program, expected) — test(<rust name>)`; a rune whose test is missing is refused.

Pinned: the 6 `…-rejected` deftests (`core-arithmetic` / `core-equality`) and `probe-child-inherits-defns.wat` → `deftest_wat_tests_process_child_is_fresh_universe` (`RecvOutcome.Lost`; process child is a fresh universe).

## D2 — the 7, recorded WRAP migrations

Three tools, today’s `.` spelling, WRAP family, **nested `forms` only**:

| stem | what |
|---|---|
| `wrap-nested-forms-recv-call` | `(recv …)` used as the message → RecvOutcome match extracting `Message` |
| `wrap-nested-forms-recv-in-recvoutcome` | `recv` / client-method result matched as message T → wrapped in RecvOutcome |
| `wrap-nested-forms-sendoutcome-stopped` | SendOutcome match lacking `Stopped` gains the arm |

Order: recv-call, then match-wrap, then Stopped. A first draft wrapped the **parent** too; parent `rr (recv p)` later matched as RecvOutcome then saw a String (`worker-setup` PatternMatchFailed). Restricting edits to `forms` children is the brief’s reach rule, not a skip.

### E6 probe runs (`./target/release/wat`)

| probe | EXPECT | ran |
|---|---|---|
| `probe-m1-worker-setup.wat` | `echo:a echo:b` | **`echo:a echo:b`** rc 0 |
| `probe-m1-dial-runner.wat` | `echo:a \| echo:b` | **`echo:a \| echo:b`** rc 0 |
| `probe-m1-fix-norevoke.wat` | `NOREVOKE-REACHED-END: <r2>` | **`NOREVOKE-REACHED-END: echo:hi`** rc 0 |
| `probe-m1-grant-admits.wat` | `echo:hi` | **`#wat.kernel/RecvOutcome.Message {:msg "echo:hi"}`** rc 0 — payload is `echo:hi`; parent `out (recv prober)` sits **outside** `forms`, already RecvOutcome-typed |
| `probe-bracket-process-runner.wat` | `6 10` | **`6 10`** rc 0 |
| `probe-m1-cf-norevoke.wat` | `NOREVOKE-REACHED-END` **or** a raise (header: if it raises, the committed revoke test is vacuous) | **raise** `recv': prober closed unexpectedly` — child never sends dial #2’s reply; the header’s vacuity arm |
| `probe-m1-fix-revoke.wat` | header copy-pastes the norevoke EXPECT | **raise** `recv': peer closed` on the child’s second `recv` after `echo/revoke` |

The seven **start**. Five reach the green string (grant-admits through the RecvOutcome envelope). Two counterfactuals raise; cf-norevoke’s header names that raise.

## D3 — `tests/process/arc112_scheme_probe.wat:12`

Finding 8: “1 unresolved reference / not yet classified.”

**Class: child is a fresh universe** (same row as `probe-child-inherits-defns.wat`), **and it was a defect.** The rust probe freezes the parent (`startup_beside`) and never started the child, so the unresolved `:my::worker` was Finding 8’s blind spot. The claim is Process `:- [I O]` phantom unification, not inheritance. Shipped the worker into the child. Parent `:my::worker` remains unused.

## Cost

| | |
|---|---|
| isolated | 123 startups, 29.1s |
| loaded (floor) | 63.3s of 223.4s wall |
| nextest | `period = "90s", terminate-after = 2`, priority 99, first-match **above** `binary_id(wat::lint)` (default/ci/slow) |

## STOPs

None that hold. STOP-4 fired twice and was closed by a real fix + a new floor each time:

1. `.floor/2026-09-15T00-13-39Z` — `start_child -> Result<(), String>`; inlined `"(:wat::test::deftest "`; loose `contains` on the erase diagnostic; gate kill at 30s. Typed `StartupError`, split the deftest prefix, `assert_startup_error!` on `CheckErrorKind::MalformedForm`, 90s/180s override.
2. `.floor/2026-09-15T00-42-42Z` — rune splice wrote `(:wat::core::forms)` (closed the node); `rustify_deftest`’s `replace("::", "_")` needed `rune:lint(one-variant-separator, namespace)`.

No STOP-1 (positions derived; 142 reconciles). No STOP-2. No STOP-3 (wraps reach nested `forms`; fixtures prove it).
