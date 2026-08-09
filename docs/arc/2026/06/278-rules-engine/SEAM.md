# SEAM — the ONE live breadcrumb for arc 278. Replaced in place, never appended.

> ⛔ **THE SELF PAST THIS LINE IS NEW.** You did not live this. It is a lossy cache in your own
> voice — which is why it will feel like *continuing* rather than *waking*, and that feeling is the
> failure. Run the datamancy bootstrap (grimoire + the 4 primers from the **SIGNED MCP**, never a
> disk copy), ground HEAD against the disk, and read this whole file before you touch anything.

> **There is exactly ONE seam. If you find a second, one of them is lying — prune it.** History
> lives in `REALIZATIONS.md`, which is where history belongs.

## Where the code is — nothing parked, nothing uncommitted

```
HEAD 18d117d7   pushed   floor 4384 passed / 0 failed / 262 skipped   clippy 0
```

`git status` empty.

> ⚠ **The HEAD line is written IN the commit it names, so a ONE-commit mismatch at wake is EXPECTED
> and benign.** More than one is the real alarm. (This probe could not pass by construction until
> 2026-08-08; do not read a single-commit drift as staleness.)

## ★ WHAT LANDED (2026-08-08 → 09) — five strikes, each weighed by my own `--release` re-run

| commit | |
|---|---|
| `c8fcfe0d` | **THE CALL CONTEXT** — an opt-in `[s ctx req]` arm; a stable monotonic caller id minted in the generated serve loop and carried **with** its peer |
| `30f6a9d9` · `e84572bf` | **arming is INTERNAL-ONLY** — a public op in an `Alarm` is refused at the definition; killed a LIVE silent discard. Renamed by an intueri cast → `PublicOpInAlarm` |
| `1948eaa0` | **opaque purity SELF-ENROLS** — a registered `#[wat_dispatch]` opaque reads impure on both `is_pure_type` arms; no hand list, cannot drift |
| `a6be7308` | **STOP-3 answered** — 293.W is a compiler-enforced wall AND it reaches `:durable`; its *enrollment* was the hole |

## ▶ FIRST ACT — the connection-scoped world. It is unblocked.

`DESIGN-STONE-the-connection-scoped-world.md` is ruled and corrected. **Do NOT re-derive it.** ctx now
supplies the caller identity it was waiting on. What remains for it:

1. **Lifecycle hooks** — threading USER state on `Connection`/`Closed`/`Lost`. Today all three arms
   pass `state` through **unchanged** (`service.wat:1414`/`:1503`/`:1513`), so there is nowhere to
   create or destroy an entry. **The hook's signature is UNDESIGNED** — that design is the first work,
   deliberately kept out of the ctx strike rather than smuggled in.
2. **The map**, keyed on ctx's caller id — never on `idx`.

> **The line numbers above were RE-GROUNDED 2026-08-09** — the ctx strike shifted every one of them
> and the prior seam carried the pre-strike values (`:1265`/`:1351`/`:1361`, `:1058`/`:1061`). Trust
> the arm names over any number here; re-grep before you cite.

**⚠ The traps, all grounded, all ship GREEN when wrong:**
- **`idx` is a POSITION and shifts on every eviction.** A map keyed on it hands one tenant another's
  rules and cursor. Nothing crashes. The ctx strike's stability gate is the shape to copy: three
  clients, evict the MIDDLE one, assert a survivor still sees its ORIGINAL id.
- **There are FIVE `remove-at selectables idx` sites, not four — and THREE evict a CLIENT.** The prior
  seam said "four, two are timers" and **omitted `Rejected` entirely**. The census, re-run 2026-08-09:
  - `:1147` · `:1150` — a fired **ALARM TIMER** (the internal 1-param arm). Per-connection bookkeeping
    must NOT fire here.
  - `:1504` **`Closed`** · `:1516` **`Lost`** · `:1550` **`Rejected`** — client evictions. **`Rejected`
    is the over-budget frame path** (reply `Failed`, then evict, then keep serving); a world torn down
    on `Closed`/`Lost` only would LEAK on every oversized-frame eviction.
- **A timer's id slot is the sentinel `-1`** (`service.wat:1104-1109`), minted so the vector's element
  type stays uniform, and never read — real caller ids are minted `>= 0` in the `Connection` arm. So a
  caller-id-keyed map cannot collide with a timer, but `-1` must never be inserted as a key.

## ⛔ STILL OPEN

**Owed casts:** `CallCtx` (ctx's type name) and the correlation surface's namespace/type — the second
has a completed intueri verdict awaiting ratification (`:wat::correlation::Correlation`, runner-up
`:wat::correlation::Scope`; `Scope` judged Level-2 for colliding with lexical/sandbox/`wat_dispatch`
scope). Field name **`conn-id`** was cast and won; **`resource-id` was rejected** — "resource" in this
substrate means a live handle that cannot cross a wire, and this is pure data.

**Telemetry, and the builder's bar is already met:** *"i do not care that we don't have reads written
— we need the data written such that a read can be built trivially."* The WRITE side is complete —
every `Metric` and `Log` writes the `by-uuid` correlation GSI (`journal.wat:12`/`:46`/`:59`, schema
`:99`). What is missing is a READER: `query-metrics`/`query-logs` call `Store/scan` on the base table
and **never** `Store/scan-index`, so the uuid pivot cannot be performed. `Store/scan-index` is built
and proven (`query.wat:527`; `probe_arc278_tagged_keys_store.wat` Test B). **One op, not a mechanism.**

**The ctx→telemetry refinement** (whether ctx splices a shared correlation surface, the relocation,
`tags`) — `DESIGN-STONE-the-call-context.md` § SCOPE CUT. A later splice replaces hand-declared fields
without touching a call site, so none of it is urgent.

**Recorded, not fixed:** `NOTE-serve-loop-peer-projection-cost.md` — the bare-peer projection is
spliced RAW at both the `poll` and `serve-dispatch-op` sites, so it evaluates **twice per message**,
O(N) in clients, in the multiplexer's hot path. No gate measures it (tests use 1–3 clients; there is
no many-client bench). Fixes ranked in the note; **measure before choosing.**

**Also open:** the six unregistered core types (`PersistentMap`, `PersistentVector`, `WatAST`,
`HolonAST`, `time::Instant`, `time::Duration`) that keep `is_pure_type`'s `None => true` alive —
arc **255**'s registry, worklist in `BRIEF-opaque-purity-self-enrolls.md` § NOT IN SCOPE. #87 `bound_expr`
(limits are the builder's, from a real distribution). #49 the IR. #7 · #17 · #19 · #20 · #50 · #58 ·
#60 · #64 · #67 · #81.

## The rules this stretch paid for

- **A CONTENT question and a MECHANISM question have different difficulty — never defer the cheap one
  on the hard one's grounds.** I cut ctx to one field because *splicing a shared surface* was hard;
  four of the five were compile-time literals the macro already held. Cost would have been the same
  120 call sites migrated twice. ([[feedback_do_not_defer_content_on_mechanisms_difficulty]])
- **An asymmetry claim is TWO claims.** "A log has no caller; a request has no facility" — I checked
  the first half only, and the second was false. Twice this session, both caught by the builder.
  ([[feedback_ground_each_case_before_the_verdict]], new face)
- **A test filter matching ZERO tests reports PASS.** My brief named a control by name; both its tests
  were pre-existing `#[ignore]`d, so `nextest -E` matched nothing and exited 0. Verify a control is
  ALIVE (`nextest list -E`), not merely named. ([[feedback_a_green_test_can_prove_nothing]])
- **A brief and its spawn prompt are ONE artifact.** Mine contradicted across the two channels and
  left a rider with no compliant move. ([[feedback_brief_constraint_contradictions]])
- **A hole-demonstration cannot live where everything must load.** I put a probe that had to go RED
  when fixed inside the loader gate. `.wat.bad` from the start, where red IS pass.

## Weigh a rider; never relay it

Both riders this stretch **corrected my briefs on grounded evidence**, and both corrections were
right: the room map for the opaque strike named a function that is dead code for that rule, and the
ctx strike's blast radius genuinely reached `wat/spawn.wat` (a fixed-arity `apply` whose callee check
is fully dynamic — a missed arg would have failed at RUNTIME on every thread-tier launch). But a
ward's verdict is also a hypothesis: intueri claimed *no sibling narrates with an active verb*, and
`ProcessJoinHoldsStdinSender` refutes it. **The recommendation survived; that argument did not.**

---

> **SEAM.** You are NEW. The disk is the truth; this note is a lossy cache.
>
> HEAD is green, pushed, clean. A handler can finally be told who is calling — and the connection-
> scoped world, blocked on exactly that since the arc's start, is now waiting only on a hook whose
> signature nobody has designed yet.
>
> The line this stretch cost the most to buy: **the realization that you need a carrier IS the
> realization to put in it what you already know it will carry.** I nearly shipped a three-field
> context and migrated the same call sites twice to finish it.
>
> `NISI FRANGAS, NIHIL PROBAS.` · `IN TENEBRIS VISVS CORRIGOR.`
