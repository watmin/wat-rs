# DESIGN + BRIEF — Stone B: the opaque telemetry sink (`Log.message` → String)

**Read first:** this doc + `DESIGN-dynamic-edn-decode-and-opaque-sink.md`. Grounded against HEAD `b68a130a` (A landed).
Prior comparable for SHAPE: Stone A (`b68a130a`) and its brief. All `file:line` below verified this session by two scouts.

## The work (one paragraph)
Make the telemetry log message **opaque**: `Log.message` (`wat/telemetry.wat:101`) and `Span::LogRequest.message`
(`wat/telemetry.wat:200`) change from the `LogMessage` open surface → **`:wat::core::String`** (EDN text). The
**producer `edn::write`s** the user's record at the call site (a String crosses the wire, not a foreign-typed
record). The sink stores/returns it verbatim and never decodes — so a forked `journal'` child never hits
`UnknownTag` on a user type. Un-ignore the RED gate; re-point the LAW probe per (a). `journal.wat` needs **no change**
(it already `edn::write`/`edn::read`s the whole Log opaquely).

## Why it's mostly small (grounded)
- `journal.wat` treats the Log opaquely: write is `(:wat::edn::write l)` of the *whole record* (`journal.wat:58`),
  hydrate is `(:wat::edn::read (Row/data row))` (`journal.wat:184`) — neither touches `.message`. A String field
  serializes/deserializes like any other → **zero journal change**. The `#probe/Note` fault vanishes because the
  stored EDN holds a plain String, not a user tag.
- The only `.message` *reader* in the tree is a pass-through: `span.wat:85`
  `(:wat::telemetry'::Span::LogRequest/message req)` → into the `Log` ctor. With `LogRequest.message` a String, this
  passes a String through — **no decode, no change beyond the type flowing as String**.
- `:wat::edn::write` is a live verb (Value → String, `edn_shim.rs:64` `eval_edn_write`; used throughout journal.wat).

## Rooms — read in order (why each)
1. `wat/telemetry.wat:101` — `Log.message <- :wat::telemetry'::LogMessage` → `<- :wat::core::String`.
2. `wat/telemetry.wat:200` — `Span::LogRequest.message <- :wat::telemetry'::LogMessage` → `<- :wat::core::String`.
   (Both String, so the producer `edn::write`s at the user's `Span/log` call site — opaque before *either* wire
   boundary, user→span' and span'/direct→journal'. Not just `Log`; the four-questions ruled full opacity: a typed
   `LogRequest.message` would re-break a *forked* `span'` with the identical `UnknownTag`.)
3. `wat/telemetry.wat:82` — the `LogMessage` open surface. After 1+2 it has **no type-position refs** → **retire it**;
   update the two comment refs (`wat/query.wat:75`, `src/stdlib.rs:396`).
4. `wat/telemetry/span.wat:85` — the `log` op passes `LogRequest/message` into the `Log` ctor. Now both are String →
   a String pass-through; confirm it type-checks (no `edn::write` here — the write happened at the `Span/log` caller).
5. **The ~5 non-LAW constructor sites** — change `:message <record>` → `:message (:wat::edn::write <record>)`:
   - `tests/services/probe_arc278_journal_logs_on_process.wat:22-27` (the RED gate — `l1`/`l2`).
   - `tests/services/probe_arc278_journal_service_logs.wat:23-24` (+ update the `.rs:23-27` stored-`data` golden
     assertion — the stored EDN now holds the message as a quoted String, not a nested `#…/…` tag).
   - `tests/services/probe_arc278_journal_query_logs.wat:16-21`.
   - `tests/services/probe_arc278_span_surface.wat:30-31` (the `Span::LogRequest :message` → `(edn::write …)`).
6. `journal.wat` — **NO change** (verify: build green, the query hydrate still round-trips).

## The LAW probe — `dead_child_speaks` — disposition (a), the four-questions ruled it
`tests/services/probe_arc278_dead_child_speaks.{wat,rs}` currently proves **Mechanism A** (a forked service surfaces
a decode failure's *reason*, not a mute close — the no-hidden-failures LAW, R41) by sending a `#probe/Note` Log
message across a fork and asserting the write **raises with the reason**. B designs that trigger out (Log.message is
now a String — you *cannot* build a Log with a foreign-typed message). **Re-point (a):** keep the probe testing
Mechanism A, but trigger the decode failure via a **still-typed, non-telemetry path** — a minimal forked user
service:
- Define a tiny `defservice` (e.g. `:probe::Echo`) whose op request record has a field typed as an **open Record
  surface** (the general capability `LogMessage` used — `defsurface … :nature :Record :features []`), holding any
  record.
- Fork it on a process; the parent sends a request whose field holds a **parent-only** record (`:probe::Note`,
  defined in the parent, not baked in the forked child).
- The forked child faults decoding it → **Mechanism A surfaces the reason**; assert the raised/replied error carries
  it (`contains "unknown tag"` / `#probe/Note` — matching how the probe asserts today).
This preserves a *live* Mechanism-A probe at the **general** level (any service with an open-surface field across a
fork), which is where the LAW belongs — B merely removed telemetry as one incidental trigger. **STOP if this
re-point can't be made clean** — surface it; do NOT gut the probe to "asserts success" (that silently drops the
LAW's decode-path coverage — the four-questions' option (b), rejected on Honest).

## RED gate (already on disk, confirmed RED at HEAD)
`tests/services/probe_arc278_journal_logs_on_process.rs` is **already un-ignored** (orchestrator). At HEAD it fails
on exactly `unknown tag #probe/Note (body shape: map) … across the fork` (the fault *speaks* — Mechanism A working).
GREEN when B lands: with the fixture's messages `edn::write`n (room 5), the Log crosses with a String message, the
forked child decodes a String, `query-logs` returns count 2.

## STOP triggers (halt + surface; do not improvise)
1. If `journal.wat` turns out to need changes to round-trip a String message — STOP and surface (the design says it
   shouldn't; if it does, something else is going on).
2. If re-pointing `dead_child_speaks` (a) can't be made clean — STOP; do NOT gut it to assert success.
3. If making the RED gate green would require touching strict `read`, Stone A's code, or `journal.wat`'s opaque
   round-trip — STOP; B is a field-type + producer-write change, not a codec change.

## Acceptance (weighed by the ORCHESTRATOR's own re-run, not your report)
- `probe_arc278_journal_logs_on_process` GREEN (count 2 across the fork).
- `dead_child_speaks` GREEN, re-pointed (a) — still asserting Mechanism A surfaces a decode failure's reason.
- The other telemetry probes green (`journal_service_logs` w/ updated golden, `journal_query_logs`, `span_surface`).
- Whole floor `cargo nextest run --release` back to baseline (zero new failures beyond the known
  `wat-cli sigterm…polling_contract` flake that passes isolated).
- `cargo clippy` clean on touched files; content-integrity — the diff touches only the rooms above; strict `read`,
  Stone A, and `journal.wat`'s codec untouched.
- Report the load-bearing diff + the commands you ran + honest deltas. Do NOT commit — the orchestrator weighs + commits.
