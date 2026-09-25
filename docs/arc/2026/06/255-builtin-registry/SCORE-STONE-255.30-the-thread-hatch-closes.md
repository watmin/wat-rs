# SCORE — STONE 255.30: the thread escape hatch closes

Struck against draw `995880ac7` (parent `eb5284ff3`). `ThreadSelfPeer` is gone.
A self-peer is `(:wat::kernel::Peer :- [S R])`. There is no alias. The one-way
derive in `wat/spawn.wat` is deleted.

The respelling is the recorded codemod
`wat-scripts/fixes/thread-self-peer-to-peer.wat` (replay fixture beside it).
It deletes the derive form, then `rename-keyword-exact` on
`:wat::kernel::ThreadSelfPeer`. Comments are not keywords, so they keep the
old name. Dry-run was a `/tmp` copy; the live apply was 69 `.wat`/`.wat.bad`
files and did not touch `bootstrap/` or `.claude/worktrees`.

## Where purity is asked

Already asking, left in place: `self-peer`, `connect`, `accept`.

Added:

- `infer_thread_prog_type` — the child's self-peer slots, when that type
  reaches `spawn-thread` concrete.
- `infer_fn` — a fn parameter whose type is a concrete `(Peer :- [S R])`.
  `spawn-peer` typechecks the user's fn literal here. The clause body only
  sees its own type variables, so the struct is visible on the literal, and
  the error head is `:wat::core::fn`. A type variable is skipped.
- `after` — `O` is the message type, and nothing else had checked it.

`send` and `recv` do not repeat the wall. A check on every `send` payload
refused the stdlib itself: `wat/bracket.wat:735` sends
`(:wat::bracket::PoolMsg.Setup :- [:D _])`, and that open `:D` read impure.
The world would not load. That check came back off.

A `Process` spawn still leaves `I` and `O` as fresh variables. The first
`send` binds them and is not re-checked. That is the hole this stone did not
close.

The remedy text no longer names `ThreadSelfPeer`. It says a resource belongs
in `:ephemeral` state, never on a channel.

## Rows

Pre-stone words are the draw binary, before the checker change. `rc` is the
next statement.

| row | pre | post |
|---|---|---|
| struct on a thread peer | rc=0, empty | rc=3, `:wat::core::fn` purity wall on `:p30::S` |
| pure payload | rc=0, stdout `7` | rc=0, stdout `7` |
| thread-locus defservice | rc=0, stdout `"echo:hi"` | rc=0, stdout `"echo:hi"` |
| bracket kwargs pool | rc=0, stdout `["echo:a" "echo:b" "echo:c"]` | rc=0, same stdout |

Post, the struct row's stderr begins:

```
[#wat.kernel/LociDiedError.StartupError {:error #wat.check/CheckErrors {:message "1 type-check error" ... :head ":wat::core::fn" :reason "a comm carries only pure data — type :p30::S is not pure (§7 purity wall). A resource belongs in :ephemeral state, never on a channel. Redesign I/O as records, scalars, or pure enums (no Sender, Receiver, or handle fields)."}]}}]
```

The three hatch fixtures also held positive controls that must keep loading.
The impure thread arm was taken out of each and the drivers now expect the
struct row's wall. The record-on-process control, the pure `self-peer`, and
the process struct-via-`pprintln` path still run.

`probe_arc293w_peer_derives_threadselfpeer.wat.bad` would have started clean
once both heads were `Peer`. It now spawns a thread whose self-peer carries a
struct, and the driver expects the same wall.

The pre-wired spawn (crash reason, join, readiness) is untouched. The floor's
existing crash and join rows passed.

## Survivors of the name

Live code keywords: none. The one non-comment hit is the string table in
`wat-scripts/fixes/reclaim-ipc-prime-names.wat` (the old prime-name migration).
Comments keep the name, because the codemod does not edit comments. The
angle-bracket diagnostic denylist still lists `ThreadSelfPeer` so that
spelling cannot return in a diagnostic. The replay `before.pre` keeps the
preimage.

## Floors

1. `.floor/2026-09-25T05-29-41Z`, exit 100. Not re-run.
   `Summary [ 331.736s] 6116 tests run: 6114 passed (9 slow), 2 failed, 22 skipped`.
   - `no_bare_is_err.rs:259` — `assert!(result.is_err())` in
     `probe_arc293_W2c_compile_time_send.rs:75` and
     `probe_arc293_W2d_peer_purity.rs:68`. Fixed by naming the
     `MalformedForm`.
   - `no_inlined_edn.rs:805` — `probe_arc255_30_thread_hatch.rs:62`, a stdout
     literal that opened with `[`. Fixed by comparing the bracket as a char
     and the inside as a string.

2. `.floor/2026-09-25T05-38-03Z`, exit 0. This is the floor of the tree that commits.
   `Summary [ 333.162s] 6116 tests run: 6116 passed (9 slow), 22 skipped`.

## Gates

Clippy `cargo clippy --all-targets --workspace -- -D warnings` after `touch`.
One run failed `clippy::manual_next_back` on `chars().rev().next()`; that was
fixed to `next_back` before the green floor. The run on the committed tree
exited 0 (`Finished dev profile`, 6.39s, no warnings).

Census. Pre `.census/2026-09-25T05-09-55Z.txt`, files=2269, 215 non-zero.
Post `.census/2026-09-25T05-44-49Z.txt`, files=2274, 216 non-zero.
`census-diff: no STOP-8`, exit 0. No existing file changed rc. Five new
paths: the four probes and the codemod. Four are rc 0. The struct probe is
rc 1, and its first error is the wall above. That is the negative row, and it
is why the non-zero count is 216.

Delta `.delta/2026-09-25T05-45-51Z`, exit 0:

```
  ORIG-CLEAN  160/179
  CONV-CLEAN  158/179
  NEW         2
  RECOVERY    0
```

NEW files: `wat-scripts/probes/arc-170/probe-c1-clean-surface.wat`,
`wat/holon/Ngram.wat`.

Ledger 211 → 208. `the_heresy_ledger_matches_its_frozen_census` passed.
Cured: `infer_poll_prime` 4→3, `infer_thread_prog_type` 2→1,
`project_peer_io` 4→3. `is_pure_type` stayed 3.

The golden
`probe_arc293_W2a_struct_no_cross__struct_rejected_at_wire_SEND.edn` changed
in the remedy sentence. Its `:line` and `:col` were left as they were, and
the test passed, so they still match the error.

## STOP triggers

1. No class (a) site. The send-time check was not a resource on a channel; it
   was a generic `PoolMsg.Setup` whose type variable was still open.
2. No checker case broke outside the rows this stone names. The stdlib loads.
3. No existing census file changed rc. The new struct probe is rc 1 for the
   wall.
4. Crash reason, join, and readiness were not edited. Their rows passed.
