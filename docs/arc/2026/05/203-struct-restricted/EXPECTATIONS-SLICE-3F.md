# EXPECTATIONS — Arc 203 Slice 3f: error propagation

**BRIEF:** `BRIEF-SLICE-3F.md` · **Drafted:** 2026-05-17 post-3e `cd6f261`.

## Prediction

**Runtime:** 90-120 min sonnet. Hard stop 150 min.

Both files updated in-place. Every wrapper signature changes from `-> :T` to `-> :Result<:T, :counter::ServiceError>`. Every send/recv site becomes a 3-4-level explicit match (no `?` in wat). Subprocess crash detection at process tier uses `Process/join-result` returning ProcessDiedError.

## Scorecard

| Row | Pred | Conf |
|---|---|---|
| A — parse + compile | YES | medium-high |
| B — happy paths pass | YES | high |
| C — Err path demo per file | YES | medium-high (call-after-Stop is straightforward; subprocess crash detection novel at process tier) |
| D — ServiceError uses typed errors (no String) | YES | high |
| E — workspace baseline | YES | high |

## Honest deltas predicted

1. **Verbose Result-propagation** — every send/recv site grows 3-4 levels. Expected per `feedback_verbose_is_honest`
2. **Subprocess crash → ServerDied** — at process tier, ProcessDiedError comes from `Process/join-result` (returns Result<R, ProcessDiedError>); slice 3f must thread access to the Process value through wrappers to call join-result on demand. Likely needs AdminProc.proc! (already present) accessed by wrappers
3. **Recursive defn return-type when dispatch returns Result** — server dispatch tail-calls itself; if dispatch returns Result, the recursive site needs Result-handling. May need refactor
4. **Test body match arms grow** — every assertion path now matches Ok/Err; existing assert-eq calls wrap an Ok extraction

## Workspace baseline (post-3e `cd6f261`)

3 pre-existing failures. Post-3f: counter-service tests pass with Result wrappers; baseline preserved.
