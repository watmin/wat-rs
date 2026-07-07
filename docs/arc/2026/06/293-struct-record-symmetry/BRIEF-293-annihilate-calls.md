# BRIEF — 293: annihilate `:calls` from `defservice` (COMPONENDO DELEO, R33)

> **Executor: one sonnet SHADOWDANCER.** A **deletion** in the `defservice` macro (`wat/service.wat` only). Path B
> (`823b20ac`) made a dialed peer's dispatch intrinsic, so `:calls` (which shipped a callee's client contract) is dead —
> **zero consumers** (grep `:calls` across the tree found none). Work ONLY in `/home/watmin/work/holon/wat-rs/` (`pwd`
> first; `.claude/worktrees/` illegal). `cargo build`; `./target/release/cargo-wat <f>`; `cargo nextest run --release`
> (NEVER `cargo test`). **Commit NOTHING.** The reward is subtraction: remove the `:calls` machinery + its
> `::client-forms` payload; KEEP the per-op client methods (`~@methods`, emitted directly) and `service-forms`.

## STEP 0 — the ONE live consumer: annihilate the test-of-the-annihilated-feature (`wat-tests/service-telemetry-bridge.wat`)

The macro deletion's only blocker is this fixture. Builder ruling: *annihilation is total — a test measuring an
annihilated feature is annihilated; but a test of BEHAVIOR that merely uses `:calls` incidentally just loses the
`:calls` line (keep its coverage).* Apply exactly:
- **Remove** the `:calls     [:wat-tests::recorder]` clause from the `:wat-tests::worker` defservice (~line 28). The
  worker's `:Work` op calls `recorder/record` (a per-op client fn emitted top-level), which resolves in the SAME
  universe — so the **thread + hibernate** deftests stay green without `:calls`.
- **KEEP** `telemetry-bridge-on-thread` (~44-58) and `telemetry-bridge-survives-hibernate` (~84-102) — they test the
  bridge/hibernate BEHAVIOR (worker→recorder forwarding), not `:calls`. Coverage preserved.
- **DELETE** `telemetry-bridge-on-process` (~60-80, including its `(:wat::test::ignore …)` line) — its own header calls
  it *"THE GATE: `:calls` ships recorder's client-forms into that child"*: it MEASURES `:calls`'s cross-process ship,
  the annihilated feature. It is annihilated with it. (No coverage loss to worry about — the project is accreting tests,
  not yet enforcing coverage; a Path-B cross-process test accretes later, when that's built.)
- **Rewrite the header comment** (~lines 1-6): drop the `:calls` / "cross-process contract distribution" framing;
  describe it as the actor-network bridge (a worker dials a recorder service and forwards work).

## THEN — confirm zero consumers remain (after STEP 0)
```
grep -rn ":calls" --include="*.wat" . | grep -v "wat/service.wat" | grep -vi "no :calls\|;; "   # expect: EMPTY (only comment mentions, if any)
grep -rn "::client-forms" --include="*.wat" --include="*.rs" . | grep -v "wat/service.wat"       # expect: EMPTY
```
(Search the WHOLE tree — the prior grep wrongly scoped `tests/` and missed `wat-tests/`.) If either finds a live
CLAUSE use, STOP and report — do NOT delete a live thing.

## The deletions (all in `wat/service.wat`)
1. **`known-clauses`** (~line 101): remove the `(:wat::core::HashMap/assoc … "calls" true)` link — `:calls` is no longer
   a recognized clause.
2. **The recognized-clauses error string** (~line 132): drop ` :calls` from the "recognized clauses: …" message.
3. **`calls-svcs`** binding (~lines 199-205) + its comment: remove.
4. **`callee-cf-calls`** binding (~lines 1195-1211) + its comment block: remove.
5. **`service-forms-body`** (~lines 1275-1285): it currently branches on `(length callee-cf-calls) > 0` to foldr-prepend
   the callee client-forms. **Simplify to just `own-forms-call`** — `service-forms-def` (~1286) references
   `~service-forms-body`, so either rebind `service-forms-body` to `own-forms-call`, or inline `~own-forms-call` into
   `service-forms-def`. (`own-forms-call` is unchanged; it never referenced the `:calls` path.)
6. **`client-forms-kw`** binding (~lines 1192-1194) + **`client-forms-def`** binding (~lines 1289-1300): remove — the
   `::client-forms` bundle existed only to be shipped via `:calls`; with `:calls` gone it has no caller.
7. **The final `do`** (~line 1389): remove `~client-forms-def` from the emitted forms. (KEEP `~@methods` at ~1387 — the
   per-op client fns, emitted directly, always available; KEEP `~service-forms-def`.)
8. **The header doc comments** (~lines 55, 61): remove the `:calls` line from the clause list / the "who I call" note.

Nothing else. Do NOT touch `~@methods`, `~@constructors`, `service-forms-def`, `op-methods`, the surface machinery, or
anything outside `wat/service.wat`.

## Why it's safe (grounded)
- The per-op client methods (`<fqdn>/<op>`) are spliced DIRECTLY into the top-level `do` via `~@methods` (service.wat:1387)
  — they do NOT come from `client-forms-def`. So callers keep calling `svc/op` exactly as before.
- `client-forms-def` (`:<fqdn>::client-forms`) was ONLY invoked by `callee-cf-calls` (the `:calls` fold). Removing both
  together leaves no dangling reference.
- `:calls` shipped a callee's contract to a CHILD PROCESS universe; Path B replaced that need — a `:nature :Peer`
  surface's dispatch is intrinsic (runtime + the top-level surface type), so a dialed peer's methods resolve in any
  universe with no shipping. Zero services use `:calls`, so nothing regresses.

## STOP triggers (halt + report, do NOT hack)
1. **STOP-LIVE-CONSUMER:** if the FIRST grep finds any `:calls` or `::client-forms` consumer, STOP + report it — the
   deletion is not clean.
2. **STOP-REGRESSION:** the whole floor must stay green modulo the known lint. If any service/process test fails, STOP +
   report — a coupling was missed (e.g. `service-forms-body` left dangling).
3. **STOP-NOCP:** `wat/service.wat` ONLY; do NOT delete `~@methods`, `service-forms-def`, or anything the emission needs.

## The gate (EXPECTATIONS — the orchestrator re-runs these)
| what | command | expected |
|---|---|---|
| `:calls` is gone (rejected as a clause) | `cargo wat` on a `(defservice … :calls [:x] …)` probe you author | `MalformedDecl` naming the recognized clauses (no `:calls`) |
| a peer consumer still works (Path B) | `./target/release/cargo-wat scratchpad/s3-probe-calls-less-consumer.wat` | prints `consumer forwarded to kv (no :calls), ok = true` |
| the bridge coverage survives + services unaffected | `cargo nextest run --release -E 'test(telemetry_bridge) or test(smem_roundtrip) or test(sqlite_store_differential) or test(counter) or test(multiparam) or test(service)'` | passed — incl. `telemetry_bridge_on_thread` + `telemetry_bridge_survives_hibernate` (the process one is DELETED, so absent) |
| whole floor | `cargo nextest run --release` | verbatim Summary; `0 failed` modulo the known `no_inlined_wat_in_tests` reminder |

Runtime ~40-60 min (a baked-macro change forces a rebuild + the suite).

## Final report (structured): the two grep results (both empty) · the exact deletions (the ~8 sites) · the verbatim gate
results (the `:calls`-rejected probe + the peer round-trip + the targeted service tests + the whole-floor Summary) · STOP
triggers hit or "none" · any coupling you had to untangle (e.g. `service-forms-body`).

## Prior comparable: the substrate-as-teacher deletions (Break Stuff / the container-drift kills) — remove the dead form,
let the floor stay green. Path B (`823b20ac`) is the fix that made this deletable; R33 `COMPONENDO DELEO` is the doctrine.
