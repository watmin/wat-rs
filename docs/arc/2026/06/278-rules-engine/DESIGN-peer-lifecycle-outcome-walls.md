# DESIGN — the peer-lifecycle OUTCOME WALLS: `connect'` / `accept'` / `poll'` / `close'`

> **Ruling (builder, 2026-07-23):** *"for any options… four-questions… we deliver an enum for code to handle
> exceptions with. raise is uncatchable on purpose… it's a thing that must never happen."* So the audit's four
> gaps all resolve one way: every **handleable** runtime failure becomes a **matchable enum variant** the caller
> faces; **raise** (`panic_any`/`EvalBreak`, uncatchable, structured-exit) is RESERVED for **invariant violations
> that must never happen** (a program bug). This completes the no-hidden-failures LAW across the whole peer
> lifecycle — recv'/send' (values) are done (R53/R57); connect/accept/poll/close are the rest of the surface.

## The classification — which raises STAY, which become enum variants
| current raise | class | disposition |
|---|---|---|
| arity/type mismatch (wrong args, non-Address' passed) | **must-never-happen** (the checker prevents it; the runtime check is defensive) | STAYS a raise |
| "peer already closed" (double-close / use-after-close) | **must-never-happen** (a program bug — you closed it twice) | STAYS a raise |
| ECONNREFUSED / peer-cred reject / accept io-error / worker-panic-on-join / killed-by-signal / process-wait-fail / poll Lost/Malformed | **handleable** (a real runtime condition code should react to — retry, fallback, log, abort) | → an ENUM VARIANT |

## The four enums (four-questions ruled)

### `poll'` → gate the EXISTING `:wat::spawn::ServiceEvent` must-use (NO new type)
`poll'` already returns `ServiceEvent` (Impure) — a matchable enum carrying `Message`/`Closed`/`Lost[cause]`/
`Malformed[cause]`/`Rejected[cause]`. It is VALUE-FACED (no raise on a peer failure). The only gap is the
**swallow-axis**: `ServiceEvent` is not must-use, so `(let [_ (poll' peers)] …)` drops a `Lost`/`Malformed` event
silently. **Fix = add `"wat::spawn::ServiceEvent"` to `MUST_USE_PARAMETRIC_HEADS`** (the arm built for
`RecvOutcome`) + a RED probe + a checker-scout sweep (likely ~0 sites — poll' lives in the serve loop, always
matched). Four-questions: Obvious/Simple/Honest/Good-UX all YES (the exact recv' pattern, already shipped). **This
is strike 1 — smallest, no new type.**

### `close'` → `CloseOutcome` (Pure) — the tier-asymmetric outcome unified
Grounded modes: thread → clean (was `Unit`) or worker-panic-on-join; process → an **exit code** `i64` (was the
`i64` return!), or killed-by-signal, or wait-fail. The current `nil`-vs-`i64` return is already an honest-shape
smell. Unify:
```clojure
(:wat::core::defenum :wat::kernel::CloseOutcome :wat::enum::Pure
  :Closed   []                              ;; clean close (thread; process exit 0 — see Q below)
  :Exited   [code <- :wat::core::i64]        ;; process exited with a non-zero status (the current i64 return)
  :Signaled [signal <- :wat::core::i64]      ;; process killed by a signal
  :Failed   [cause <- :wat::kernel::Failure]) ;; thread join panicked / process wait failed
```
Four-questions — the one open fork: **`Closed` vs `Exited[0]` for a clean process exit.** Honest leans: a clean
exit IS an exit-with-code-0, so `Exited[code]` could carry every process exit (0 included) and `Closed` be
thread-only — but that splits by *tier*, which the surface should hide (R32). Alternative: `Closed[exit <- (Option
i64)]` (None=thread, Some=process code) — one variant, loci-agnostic. **CLOSED (needs your ratify): drop `Exited`,
make `Closed[exit <- (:wat::core::Option :wat::core::i64)]`** — loci-agnostic, the code rides in the field, the
caller matches `Closed`/`Signaled`/`Failed`. Pure (no live resource — the peer is consumed; carries only i64 +
Failure). "peer already closed" STAYS a raise (double-close is a bug).

### `accept'` → `AcceptOutcome<R,S>` (Impure — `Accepted` carries a live `Peer'`)
```clojure
(:wat::core::defenum :wat::kernel::AcceptOutcome<R,S> :wat::enum::Impure
  :Accepted [peer <- :wat::kernel::Peer'<R,S>]  ;; a live peer — WHY it is Impure (mirrors RecvOutcome<O>)
  :Rejected [cause <- :wat::kernel::Failure]     ;; peer-cred/policy denied the connecting pid (NOT retryable)
  :Failed   [cause <- :wat::kernel::Failure])    ;; io/rendezvous failure — the listener errored (maybe retryable)
```
Four-questions: the named-variant-per-kind doctrine (R52 / io-budgets "a NAMED variant per failure kind so the
caller cannot guess") splits `Rejected` (security — a caller does NOT retry) from `Failed` (io — a caller MAY
retry) — distinct because handled distinctly (Honest). Impure because `Accepted` holds a live `Peer'` (the
RecvOutcome<O> reasoning). `AcceptOutcome<R,S>` mirrors accept's current `Peer'<R,S>` return.

### `connect'` → `ConnectOutcome<S,R>` (Impure)
```clojure
(:wat::core::defenum :wat::kernel::ConnectOutcome<S,R> :wat::enum::Impure
  :Connected [peer <- :wat::kernel::Peer'<S,R>]  ;; a live peer — Impure
  :Refused   [cause <- :wat::kernel::Failure]     ;; ECONNREFUSED — no listener at the address (retryable)
  :Rejected  [cause <- :wat::kernel::Failure])    ;; peer-cred: the server's kernel-vouched identity failed (NOT retryable)
```
Same four-questions as accept: `Refused` (transport — retry) vs `Rejected` (identity — do not retry), named per
kind. Impure (`Connected` holds a live `Peer'`). `ConnectOutcome<S,R>` mirrors connect's current `Peer'<S,R>`.

## The per-verb wall (each mirrors the recv'/send' OUTCOME WALL)
For connect'/accept'/close', each strike: (1) register the enum in `types.rs` (purity as above); (2) convert the
`eval_*` — map each handleable `RuntimeError` to its variant, `Ok(peer/code)` → the success variant, keep the
must-never-happen raises; (3) `infer_*` returns the new outcome type; (4) add it to the must-use set (parametric
head for the `<S,R>` ones, bare Path for `CloseOutcome`); (5) the corpus SWEEP — every `(connect'/accept'/close'
…)` site now faces the outcome (`match … Connected/Refused/Rejected …`); (6) a RED probe; (7) weigh green, bank.
Worklist per verb = the CHECKER (R52), not a grep (the recv'-sweep lesson). Atomic per verb (no green state where
the verb returns an outcome but sites drop it).

## Sequencing (smallest-first, each banked green before the next)
1. **`poll'`** — the must-use gate only (no new type). The recv' twin; quick.
2. **`close'`** — `CloseOutcome` (Pure); moderate (the tier-unify + a small sweep — close' sites are fewer).
3. **`accept'`** — `AcceptOutcome<R,S>` (Impure).
4. **`connect'`** — `ConnectOutcome<S,R>` (Impure); likely the largest sweep (connect' is common).

## Open fork for the builder (the ONE contract decision that isn't obvious)
- **`CloseOutcome`: `Exited[code]` as its own variant vs `Closed[exit <- (Option i64)]`** — I lean the latter
  (loci-agnostic, hides the thread/process seam, R32). Ratify or override.
- Everything else (the enum direction, the named-per-kind failure split, the purity) follows the ruling + the
  established SendOutcome/RecvOutcome precedent; I proceed on those.

## Boundary
- The must-never-happen raises (arity/type, double-close) STAY raises — per the ruling ("raise is a thing that
  must never happen"). This is NOT a regression; it is the ruling applied.
- The `Failure` cause on every failure variant is `message-only` where the verb cannot know more (honest — same as
  send'/recv' `Lost`), or carries the real io/errno reason where it has it (connect's ECONNREFUSED string).
