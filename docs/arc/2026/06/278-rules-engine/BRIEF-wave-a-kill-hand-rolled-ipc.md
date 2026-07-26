# BRIEF — Wave A: annihilate the hand-rolled IPC affordances (`make-channel`, `peer-pair'`)

> **The reframe that makes this stone right.** Wave A was recorded (arc 278, 24m) as *"migrate 21
> pure raw-channel files to `peer-pair'`/`send'`/`recv'`."* That plan is **wrong**, and the builder
> corrected it: locus is reachable only through **`defservice` and brackets**. A bare pair of
> connected ends exists so a caller can hand-roll the very thing those two constructs provide.
>
> **So `peer-pair'` is not the migration target — it is a second thing to kill.**
> Builder-ruled 2026-07-26: *"i don't think we need this thing."*

## Grounded state (measured this session — re-verify, do not trust blind)

**No production or stdlib file calls `make-channel`.** Three files match on grep and **all three are
comment-only**: `wat/kernel/channel.wat` (declares the `Sender`/`Receiver`/`Channel` typealiases but
never calls), `wat/query/mem.wat` (prose explaining a *rejected* MVar-via-channel-pair workaround),
`tests/comms/probe_arc293_W2d_positive.wat`. **Confirm this with a non-comment grep before acting** —
`grep -v '^\s*;;'` — because the comment trap already fooled one pass today.

**Twelve files hold every real call site**, in three classes:

| class | files | disposition |
|---|---|---|
| **subject-is-dead** | `wat-tests/service-template.wat` (3 calls), `counter-service-{thread-N1,thread-N3,capability-N3}.wat` (2 each) | hand-rolled actors that `defservice` replaced — annihilate with the feature |
| **incidental users** | `tests/function/wat_arc170_closure_extraction_t{8,9}.wat`, `tests/program/wat_arc170_program_contracts_t7_non_portable.wat`, `tests/types/typealias_fn_type_spawn.wat`, `tests/channel/probe_arc254_channel_payload_portable_i64.wat` | **CLASSIFY FIRST** — see below |
| **the seal set** | `tests/process/pdeathsig_kills_orphan_child.wat`, `pdeathsig_diagnostic.wat`, `lifeline_orphan_clean_via_substrate.wat` | **BLOCKED** — see below |

**`peer-pair'`** (`src/check.rs:5001`, `(:S :R) -> (Tuple Peer'<S,R> Peer'<R,S>)`, "mints both ends
without spawning") has **zero production callers**. Its only wat callers are three arc-293 purity
probes that use it as the *subject* of a check — `probe_arc293_W2c_compile_time_send.wat`,
`probe_arc293_W2d_peer_purity.wat`, `probe_arc293_W2d_positive.wat`.

## ★ The incidental five — classify before you migrate

These use a channel as a convenient **typed or capturable value** while testing something else
entirely: closure extraction, a `typealias` over a fn type, payload portability. **Do not reflexively
convert them to another pair primitive.** For each, answer in your report:

> *What does this test actually require — a channel, a peer, or merely a value of some type?*

If a test needs only "a value with a parametric type that can be captured," give it that, and the
whole pair question evaporates for that file. If one genuinely needs two connected ends and neither
a `defservice` nor a bracket fits, **that is a finding** — say so, because it would be the first real
case for a bare-pair primitive and would reopen `peer-pair'`'s fate.

## The purity probes must retarget, not die

Three arc-293 probes assert the §7 purity wall fires on a **wire-peer producer** given an impure type
argument. That assertion is real coverage and must survive. But it needs *a* peer producer, not
`peer-pair'` specifically — `socket-pair'`, a `defservice` surface, or `connect'` can carry it.

**Retarget them and keep the assertion identical.** If the wall cannot be provoked through any
surviving producer, **STOP** — that would mean deleting `peer-pair'` removes an enforcement site with
no replacement, which is a different decision than deleting an unused affordance.

## The seal set is BLOCKED — do not touch it

`pdeathsig` ×2 and `lifeline_orphan` prove orphan-kill behaviour by poking Rust-level `child_pid()` +
`mem::forget`. Migrating them to the opaque `Process'` needs a **local-fork `pid` accessor**, and one
does not exist — verified: the spawn family has `Process/input`, `/join-result`, `/stdin`,
`ProcessOpts`, and no pid anywhere.

They are **unique coverage** (arc 278, 24m: *"hand-carry, never fleet-delete"*). Leave them calling
`make-channel`, and **name the pid accessor as their prerequisite in your report.** `make-channel`
cannot be fully deleted until they move; getting the other nine off it is still the win.

## ⚠ `socket-pair'` — SCOPE QUESTION, do not decide

`socket-pair'` (`src/check.rs:5011`) is the process-tier twin — two bare `SocketPeer'` ends, same
hand-rolled-IPC shape, two probe-only callers. It may share `peer-pair'`'s fate or may not. **The
builder has not ruled it.** Survey it — who calls it, what would break, whether the purity probes
depend on it as a retarget host — and **report; do not delete it.**

## STOP triggers

1. **If an incidental test genuinely needs two connected ends** and neither `defservice` nor a bracket
   fits — STOP and report. That reopens `peer-pair'`.
2. **If the purity assertion cannot be retargeted** to a surviving producer — STOP.
3. **If deleting `peer-pair'` breaks anything outside the three probes** — STOP and report; my scope
   said zero production callers, and I have been wrong about absence three times today.
4. **Do not touch the seal set** and do not build the pid accessor here.

## Method

- `target/release/wat --check <f.wat>`; `cargo build --release` freely.
- **The load-order gate must print `[]`** after any `wat/` change (this stone likely removes
  `wat/kernel/channel.wat`, which is a stdlib file — expect to run it):
  ```clojure
  (:wat::core::defn :user::main [] -> :wat::core::nil
    (:wat::kernel::println (:wat::deporder::verify-stdlib)))
  ```
- A narrow filtered `cargo test --release --test <target> -- <filter>` is encouraged. No full
  `cargo nextest run` — the orchestrator measures centrally.
- **State your expected floor.** Deletions drop it; migrations hold it. Say which you expect and why.
- Foreground only. Do not commit.

## Your report

The non-comment grep confirming the stdlib is clean; the incidental five classified with *what each
actually needed*; where the purity assertion landed; the `socket-pair'` survey; the seal set's
prerequisite named; your expected floor; any STOP.
