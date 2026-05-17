# BRIEF — Arc 203 Slice 3f: error propagation (Result-bearing wrappers, honest typed errors)

**Phase:** Sixth stepping stone. Replaces panic-on-error semantics (current `Result/expect` + `Option/expect` patterns) with honest `Result<T, :counter::ServiceError>` wrappers. Makes the canonical pattern complete for user-facing documentation.

**Predecessor:** Slice 3e shipped at `cd6f261` — server-id validation live; AccessDenied as a wire variant; wrappers still panic on transport errors.

**Successors:** Slice 3g — apply pattern to wat-lru CacheService; 3h — HologramCacheService; 3i — stdio services; 3j — closure.

## Goal

Update BOTH 3c+3d artifacts in-place to return Result-bearing wrappers with typed errors (no String escape hatches). Counter demos become honest about every error class; callers pattern-match Ok/Err and decide.

## Honest ServiceError shape

```scheme
(:wat::core::enum :counter::ServiceError
  (AccessDenied)                                              ;; server rejected server-id (wire-level)
  (PeerDied    (cause :wat::kernel::ThreadDiedError))         ;; thread tier — peer thread dropped/panicked
  (ServerDied  (cause :wat::kernel::ProcessDiedError))        ;; process tier — subprocess died (carries typed Panic chain)
  (Disconnected))                                             ;; clean recv-returned-None
```

`:wat::kernel::ThreadDiedError` (arc 060) and `:wat::kernel::ProcessDiedError` (src/types.rs:632) are substrate-provided typed errors. ProcessDiedError's `Panic` variant carries the structured chain. **NO String for chain** — String is escape-hatch and dishonest; data is structured per the substrate.

## Wrapper signature change

Every wrapper that calls `send` or `recv` returns Result. Example:

```scheme
;; Before (current — panics on Err):
(:wat::core::defn :counter::get
  [client! <- :counter::Client]
  -> :wat::core::i64
  (:wat::core::let
    [tx (:counter::Client/user-tx client!)
     rx (:counter::Client/user-rx client!)
     _  (Result/expect (send tx (Wire/User ... Get)) "send died")
     resp (Option/expect (Result/expect (recv rx) "recv died") "disconnect")]
    (match resp -> :i64 ((Value v) v) ...)))

;; After (3f — propagates as Result<i64, ServiceError>):
(:wat::core::defn :counter::get
  [client! <- :counter::Client]
  -> :wat::core::Result<wat::core::i64, counter::ServiceError>
  (:wat::core::let
    [sid (:counter::Client/server-id client!)
     cid (:counter::Client/client-id client!)
     tx  (:counter::Client/user-tx client!)
     rx  (:counter::Client/user-rx client!)]
    (match (send tx (Wire/User sid cid (UserReq/Get)))
      ((Ok _)        ; send succeeded → recv
        (match (recv rx)
          ((Err died)         (Err (ServiceError/PeerDied died)))
          ((Ok None)          (Err (ServiceError/Disconnected)))
          ((Ok (Some resp))   (match resp
            ((Value v)        (Ok v))
            ((Ok v)           (Ok v))
            ((AccessDenied)   (Err (ServiceError/AccessDenied)))))))
      ((Err died)    (Err (ServiceError/PeerDied died))))))
```

Verbose-is-honest. Wat doesn't have `?` operator; each layer matches + propagates explicitly.

## Both files updated

`wat-tests/counter-service-capability-N3.wat` (thread tier):
- `ServiceError` uses `PeerDied(ThreadDiedError)` — thread tier only sees thread death
- `ServerDied` variant N/A here (no subprocess)

`wat-tests/counter-service-process-N3.wat` (process tier):
- `ServiceError` uses BOTH PeerDied (for crossbeam parts if any) AND ServerDied (for subprocess death via ProcessDiedError)
- Subprocess death detection: when `Process/readln` returns Err, distinguish via Process/join-result Panic chain into ServerDied
- Subprocess's own wrappers (inside `:user::main`) also return Results — but their callers are the dispatch loop which handles errors locally (logs / responds AccessDenied / continues)

## Test body — demonstrate Ok + Err paths

Both files: existing happy-path assertions continue (Result/Ok pattern-match → extract value → assert). PLUS new error-path tests:
- After Stop, try to call a wrapper → expect ServiceError/PeerDied or Disconnected
- Cross-server forge from 3e → expect ServiceError/AccessDenied (was previously just asserted as Wire AccessDenied)
- (Process tier only) intentionally crash subprocess; try wrapper → expect ServiceError/ServerDied with typed chain

## Scope (no substrate changes)

Pure consumer slice. Zero edits to src/. Updates BOTH wat-tests files in-place.

Substrate provides:
- `:wat::kernel::ThreadDiedError` (arc 060) + accessors
- `:wat::kernel::ProcessDiedError` + Panic variant (src/types.rs:632, src/runtime.rs:18545)
- `Process/join-result` returning Result<R, ProcessDiedError> (src/runtime.rs:17089)
- `Result<T, E>` pattern-matching (arc 110/111)

## STOP triggers

1. **Recursive `:counter::*` defns now return Result** — dispatch loop wrappers may need Result-aware composition; surface if pattern is awkward
2. **ProcessDiedError construction from runtime data** — needs access to subprocess's Process value to call `Process/join-result`; surface if access pattern is novel
3. **Workspace baseline regresses** beyond 3 pre-existing failures
4. **Result<T, E> nesting in struct fields** — capability struct fields stay as-is (channels, ids); only wrapper SIGNATURES change

## HARD constraints

- DO NOT commit. Orchestrator commits atomically.
- DO NOT use String for panic-chain or any structured error payload — use substrate typed errors
- DO NOT touch src/ — pure consumer
- DO NOT use `--no-verify`
- DO NOT mint new substrate primitives

## Decay disclosure (orchestrator)

The Result-propagation pattern is verbose (no `?` operator in wat). Each send/recv site becomes a 3-4-level match. This is the honest cost; per `feedback_verbose_is_honest`, verbose forms reveal what they do. The wrappers' inner shape grows; the wrappers' callers benefit from typed errors.

Process-tier subprocess crash detection requires the Admin capability to retain Process<I,O> for join-result access — already shipped per slice 3d's AdminProc holding (peer, proc) fields.

## SCORE methodology

5 rows YES/NO + evidence:

| Row | What | Evidence |
|-----|------|----------|
| A | Both files parse + tests compile | `cargo test --release -p wat --test test counter_service` builds clean |
| B | Happy paths still pass (Result/Ok extraction) | Both tests pass |
| C | At least one Err path demonstrated per file (call-after-Stop → PeerDied or Disconnected) | Test asserts the Err variant matches expectation |
| D | ServiceError uses typed errors (no String for chain) | Code review of ServiceError enum confirms cause fields are `:wat::kernel::*Error` types, not String |
| E | Workspace baseline preserved | `cargo test --release --workspace --no-fail-fast` shows ≤ 3 failures |

## Time-box

Predicted: 90-120 min sonnet. Hard stop: 150 min.

## Workspace baseline (post-3e commit `cd6f261`)

3 pre-existing failures. Post-3f target: tests still pass (count unchanged); 3 failures preserved.

## On completion

1. Write `docs/arc/2026/05/203-struct-restricted/SCORE-SLICE-3F.md` per § SCORE methodology
2. Return final summary: rows passed/failed, workspace delta, file paths touched, honest deltas (especially around Result-propagation verbosity, ProcessDiedError detection at process tier), suggested BRIEF corrections for slice 3g (cache refactor)

You are launching now. T-minus 0.
