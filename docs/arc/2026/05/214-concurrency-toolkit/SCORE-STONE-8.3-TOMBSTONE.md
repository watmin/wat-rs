# SCORE — Stone 8.3: the child-universe boot + the deadlock tombstone

**Mode A, orchestrator-direct** (a proof stone, not a build stone — first
contact with previously-hanging process tests is own-eyes work under the
envelope; the fixture migration was 5 surgical lines).

## The proof

The DESIGN's 8.3 claim — *"a forked child does NOT inherit service plumbing;
each universe boots its own service peers on its own fd 0/1/2; the fd-7
inheritance dance has no successor"* — was scouted by the Phase-B conferre
cast (the mechanism exists: spawn_process → invoke_user_main →
bootstrap_wat_vm_process → a fresh trio). This stone PROVES it live:

| Tombstone | Result |
|---|---|
| `probe_run_hermetic_clean_exit_no_deadlock` | **ok, 0.05s** — the test NAMED for the deadlock |
| `probe_run_hermetic_panic_body_no_deadlock` | ok — the panic-path twin |
| gamma rows a–e (single/multi-thread println, panic recovery, scope-drop cascade, readln round-trip) | **5/5 ok, 0.06s** — child threads through their own trio |
| `probe_spawn_process_stdin` | **ok, 0.02s** — a forked CHILD readln→(+1)→println through ITS OWN booted universe, across the fork boundary |

Tests that hung for a month finish in hundredths of a second.

## The work

- 5 value-position `:wat::core::nil` fixtures → bare `nil` (the arc-242
  class; the standalone let-body lines — every type-position use untouched).
- The five `#[ignore = "arc-170 fd-leak fixed..."]` markers DELETED — the
  arc-170 ignore-drawdown advances **−5** (after 8.2's −3; 56 remain
  tree-wide, the named classes: nil-fixture batch, 249-diag, 251, the
  parked-255 gate + leniency gate, walker pair).
- First runs under `setsid timeout` per the envelope policy (fork tests);
  all three binaries sit in the corpus runner's excluded-by-design set —
  the gated run-tier is their home; the envelope was the honest channel.

## Gates (orchestrator-run)

lib 943/0/1 · check --all-targets 0 errors · the three revived binaries
8/8 green enveloped · FULL CORPUS 649/0 (the non-leaky set, unchanged —
the revived binaries are run-tier residents).

## The class's standing

The deadlock that OPENED arc 214 — the ambient-stdio-ProcessPeer round-trip —
now has: its mechanism dead (no handle-passing, no fd-7 inheritance, no
bridges — Slices 8.1–8.2w), its architecture's successor warded (the
vigilatum-stamped src/services/), and its tests ALIVE AND GREEN (4.4's
peer_process_round_trip + this stone's five). **The class has no living
member and a standing tombstone.** What remains of Slice 8: 8.4
(`:wat::services::start`, the user-service sugar) — then Slice 6 takes the
nearly-tenantless typed_channel.
