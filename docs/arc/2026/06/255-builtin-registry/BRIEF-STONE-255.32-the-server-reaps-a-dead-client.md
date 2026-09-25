# BRIEF — STONE 255.32: the server reaps a dead client — no reason, no eprintln

**Drawn 2026-09-25 against `main` @ `d9bca6907`.** Floor 6116/6116, clippy 0, census 215 non-zero, delta
**NEW 2 / RECOVERY 0**, ledger 208. **Executor: grok, via pulsare.** Strike, write the SCORE, then
`pulsare_yield kind=scored`.

## Read first

`FINDING-INTUERI-the-outcome-vocabulary.md`, especially its last section, **the RULING (the supervisor
pattern)**:

> *"the server does not receive a client's death reason as it cannot use that meaningfully.. the client
> just goes away and the server must reap their resources once they observe a dead handle. a dead
> service should optimistically notify all connected clients that its crashing without providing a
> reason, the reason may only be provided to the admin handle who owns the service (supervisor pattern)"*
> and *"eprintln must be our final statement just before we ungracefully terminate"* (R51).

## The defect

`wat/service.wat` ~:1970–1980. The generated serve loop's `ServiceEvent.Lost` arm has this comment:

> *"surface the reason on the honest loud sink (stderr) BEFORE evicting + continuing to serve"*

Its body is `(:wat::kernel::assertion-failed! …)`, which **raises**, so the eviction and the continue never
run. It is unreachable today only because `poll'` never builds `Lost`. If a server ever sees one, **one
client crash kills the service** (a DoS). The comment's promise is also impossible: in wat, `eprintln` **is**
the dying declaration (R51). The neighbouring `Rejected` arm (~:2005) already refuses `eprintln` for exactly
this reason.

## The work

1. **The arm reaps and continues.** Evict `idx` (the same eviction the `Closed` arm does,
   `(:wat::seq::remove-at selectables idx)`) and recurse into serve. **No reason is used, no `eprintln`, no
   raise.** Rewrite the comment to state the ruling: *a client's death belongs to its own owner; the server
   reaps the dead handle and serves on.* Keep the arm's shape as it is (binding `cause` as `_cause` is fine).
   **Do not rename or remove the variant here**: the vocabulary migration is its own stones.
2. **Leave `wat/bracket.wat`'s `Lost` arm alone.** There the pool **is** the owner of its runners, which is
   the supervisor vantage, where raising is correct. Say in the SCORE that you read it and why it stays.
3. **Measure, do not change: does the code already honour the ruling's other two rows?**
   - **(a) Server vantage:** census every path by which a *server* could receive a client's death **reason**:
     `poll'` (the finding says it has no crash channel), peers-only `select'`, and anything else. Report
     file:line for each and whether a reason can arrive.
   - **(b) Client vantage:** when a service crashes, do **all** connected clients get an optimistic,
     **reasonless** notice? The finding mentions a reason-free `PeerCrashed` sentinel broadcast by
     `serve-dispatch-op` (`src/kernel/serve.rs`, `src/kernel/peer.rs` ~:256). On **both** loci, measure:
     what a connected client's `recv'` returns when the service panics mid-serve. Verbatim, with the
     variant and whether any reason leaks. **Write a probe**: a two-client service that panics on one op,
     with both clients' outcomes printed.
   - **(c) Supervisor vantage:** the owner's handle (`Thread'`/`Process'`) gets the reason. Confirm it with
     the same probe (the owner's `recv'`/stop outcome, verbatim).
4. **Rows:**
   - a macro-expansion row showing the emitted `Lost` arm has no `assertion-failed!` and no `eprintln` and
     recurses into serve (expand at the form level; the arm is unreachable at runtime today). Show the
     pre-stone expansion has `assertion-failed!`;
   - the probe from 3(b)/(c) committed as a row that pins today's behaviour, whatever it is.

## STOP triggers

1. 3(a) finds a path where a server **does** receive a client's death reason → report it verbatim, and
   do not change it.
2. 3(b) finds a client **receiving the service's reason**, or **no notice at all** → report it verbatim,
   with the variant. That is a ruling violation for a later stone, not for this one.

## Expectations — fixed before the strike

| what | expected |
|---|---|
| the emitted `Lost` arm | evicts and recurses; no `assertion-failed!`, no `eprintln`; pre-stone raised |
| `bracket.wat` `Lost` | unchanged, with the reason stated |
| 3(a)/(b)/(c) | measured, verbatim, per locus |
| floor · clippy · census · delta · ledger | green · 0 · `no STOP-8` · NEW 2 / RECOVERY 0 · ≤ 208 |

## Doctrine (the house rules; `wat-rs/CLAUDE.md` is not injected, so they are here)

- The floor is **`scripts/floor.sh`** (`cargo nextest run --release`), captured to `.floor/`. Read the
  Summary line, never a piped exit code.
- ⛔ **THERE IS NO KNOWN FLAKE. A RED IS A RED.** Never re-run to green. Copy the failing block **verbatim**
  from the captured log, name the assertion that fired, fix it at its cause if it is this stone's, then
  re-run the whole floor and say so.
- Also run clippy (force a recheck by touching a file), `scripts/replay/census.sh --diff` against a
  pre-change census you take first, `scripts/replay/delta.sh`, and the keyword heresy ledger (it may
  shrink, never grow).
- Line-pinned `.edn` goldens that move because `wat/service.wat` changed: recapture them with
  `UPDATE_EDN=1`, and show the diff is `:line` only.
- ⭐ **Prove every row can say BOTH words** on a pre-stone binary built from this brief's commit. Capture
  `rc=$?` into a variable on the **next** statement.
- `defservice` cannot be runtime-macroexpanded. Expand at the form level; copy the shape of
  `wat-scripts/scratch-pad/255-17a-child-main-transport.wat`.
- **If this brief contradicts the code, the code wins — say so plainly** in the SCORE.
- Commit locally with `git add -- <paths>`; **do not push**. Write
  `SCORE-STONE-255.32-the-server-reaps-a-dead-client.md` beside this brief.
