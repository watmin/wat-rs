# BRIEF — STONE H: every wire encoder is strict

Read `DESIGN-stone-H-every-wire-encoder-is-strict.md` first — its census table is your map.

## The work, in order

1. **Measure before changing:** (a) process-tier `after` with a handle `msg` — what the receiver
   gets; (b) thread-tier `try-send` of an unencodable-but-legal in-locus value — does it raise;
   (c) `HandlePool` and a `Value::Vector` sent over a process wire — what arrives.
2. **`after`, process tier** (`src/kernel/resource.rs:1128`): use the strict encoder; refuse at
   `list_span` with the same message shape stone G uses.
3. **`Value::to_wire`** (`src/comms/mod.rs:163`, callers `src/comms/process.rs:336,462`): establish
   whether wat can reach it with a user value. Unreachable → say how you proved it and leave a
   comment that states the invariant. Reachable → strict.
4. **`try-send`** (`src/kernel/message.rs`, the arm around `:403-430`): encode only on the socket
   arm; the thread tier passes the `Value` through as `send` does.
5. **`println`/`eprintln`**: confirm a child's stdout is not its peer channel. One line of evidence.
6. **The invariant where the next hand meets it** — ideally a gate (e.g. no call to the lenient
   writer from a module that ships wire frames); if the material does not allow one, say so.

## STOP triggers

1. A wire path you find that the census lacks — add it and report the delta.
2. (c) shows a value arriving as something other than itself — REPORT; do not change the strict set.
3. Changing `try-send` alters any thread-tier behaviour other than the spurious refusal.
4. Any gate reddens that you did not add — capture whole, name the arm. ⛔ Do not re-run first.

## Prove it

Extend stone G's probe (`tests/comms/probe_ex003_stone_g_wire_send_refuses.rs`) or add a sibling:
`after` with a handle refuses at the sender; `after` with a pure value arrives whole; thread-tier
`try-send` of an in-locus value is `Sent`; process-tier `try-send` of a handle still refuses.
**Mutation:** `after` back to lenient → its case reds.

## Mechanics — ⛔ read

- Floor: `nohup scripts/floor.sh > <file> 2>&1 &`, then repeated foreground
  `until grep -qE '^ *Summary' <that file>; do sleep 30; done` blocks. Do not end your turn while it
  runs. Never `pgrep -f 'cargo …'`. Read `Summary` lines, never piped exit codes.
- `cargo fmt` reformats the whole workspace — `rustfmt <file>` or neither.
- `git add` explicit paths BEFORE `git ls-files`-based gates; never `git add -A`.
