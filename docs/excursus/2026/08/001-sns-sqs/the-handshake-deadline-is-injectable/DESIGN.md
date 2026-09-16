# DESIGN — the handshake deadline is injectable

**Drawn 2026-09-15**, builder-directed: *"manifest runner then, and get the handshake deadline
injectable."* **Stone 1 of 2** — it exists so `the-probes-run-in-the-floor` can afford the silent-child
row. **NOT STRUCK.**

## Why this first

`probe-a-silent-child-cannot-hang-launch.wat` drives the `TimedOut` arm that `ab419aaa3` made reachable
at five handshake sites. It costs **30 s per tier**, because the deadline is a frozen literal:

```
(:wat::core::def :wat::spawn::STARTUP-HANDSHAKE-DEADLINE-MS 30000)      wat/spawn.wat:108
```

Named by 5 sites (`spawn.wat:552`/`:609`, `test.wat:334`/`:444`, `service.wat:3510`). The floor is
~542 s; two 30 s parks is ~11% for one row. So either the number becomes injectable or the row stays
out, and the arm stays unguarded — which is the whole disease the runner is being built to cure.

## The mechanism, and why the obvious routes are closed

| # | route | verdict |
|---|---|---|
| a | **read an env var in `wat/spawn.wat` at load** | ⛔ **CLOSED BY LOAD ORDER.** `spawn.wat` is manifest position **171** and no env-reading primitive exists before it — `grep -rn 'env' wat/program.wat` finds no var/getenv/read at all. Same wall that blocked a queue-backed `brackets/map`. |
| b | **a mutable/settable stdlib global** | ⛔ the stdlib is **frozen into the binary at build time**; a `set!` on a frozen `def` is not a knob, it is a hole. |
| c | **thread it through `launch` as a parameter** | ⛔ partial: `child-main` is *generated* and takes no such parameter, so the child end stays fixed while the parent end moves — half a fence, which is exactly what `ab419aaa3` was struck to remove. |
| d | ⭐ **a runtime-sourced value: read `WAT_STARTUP_HANDSHAKE_DEADLINE_MS` once in Rust, default 30000, expose it to wat under one name** | **the shape.** Always available regardless of load position; one place; the 5 sites change from naming a constant to calling a nullary. |

⭑ **Precedent exists for the env-knob form**: the runtime already reads `WAT_TEST_OUTPUT`,
`WAT_RUNTIME_BIN`, `UPDATE_EDN`, `RUST_BACKTRACE`. What has **no** precedent is a *stdlib constant*
sourced from Rust — `grep -rn 'STARTUP-HANDSHAKE-DEADLINE-MS' src/` is empty today. State that in the
SCORE: this is the first one, so the shape it sets will be copied.

⛔⛔ **THE TRAP-DOOR THAT MUST NOT BE TAKEN: do NOT read the env var inside `recv_by_deadline`.** It is
the cheapest diff and it is wrong — it would silently re-time **every** `recv-by-deadline` in the corpus,
including `owner-recv-loop`'s 10 000 ms and any future caller, from one variable named for the handshake.
The knob must attach to the **handshake's number**, not to the primitive that consumes it.

## Env-var discipline

- Read **once** (`OnceLock`), not per call — a deadline that changes mid-run is a different bug.
- **Invalid or absent → 30000**, silently. A test harness typos a value; the floor must not turn that into
  a red somewhere unrelated. ⚠ But an invalid value must not read as *deliberate*: log-or-report is the
  open question — name your choice in the SCORE.
- **Process-tier children inherit env**, so parent and child agree by construction. Verify it rather than
  assume: the child's deadline must move with the parent's.
- **Zero is not a wait** — precedent `zero-is-not-a-wait/`. Decide and state what `=0` means (refuse, or
  clamp to 1); do not leave it undefined.

## Scope

- **Only the handshake number.** `owner-recv-loop`'s 10 000 ms and `call-by-deadline`'s caller-supplied
  `ms` are untouched.
- The justification prose stays in `wat/spawn.wat` (it is the number's home even when the value arrives
  from Rust) with a pointer to the Rust site. **Two homes for one value is the drift `ab419aaa3` removed —
  do not reintroduce it**: exactly one place may hold `30000`.

## Trap-doors

1. The 5 call sites are in **three files**, one of them a generated body (`service.wat:3510`) — a
   quasiquote, so the call must be spliced as generated code, not a literal.
2. `wat/test.wat` sites are on the path of **every spawned test program**; a mistake here reddens broadly.
3. Do not change the **default**. 30000 stays; only its injectability is new.
