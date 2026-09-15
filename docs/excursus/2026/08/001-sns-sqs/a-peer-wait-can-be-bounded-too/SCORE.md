# SCORE — a peer wait can be bounded too

**SCORED.** Executor: grok, 2026-09-15, branch `sns-sqs`, HEAD `edf359ef1` (DRAWN). Did not commit.

⛔ The headline is that **a hang became a named death.** Recovery is the deferred supervision ruling.

```
     Summary [ 561.516s] 5251 tests run: 5251 passed (9 slow), 22 skipped
```

`.floor/2026-09-15T20-41-34Z/` — `scripts/floor.sh` exit **0**, **no `ARM.txt`**.
Count **5251**, unchanged. Clippy **0**. NORUN **0**.
Floor delta is a band, not a number (EXPECTATIONS). Every service still starts: `child-main` is generated into all of them.

---

## ⭑ THE HEADLINE — `recv-by-deadline` accepts a Peer; child-main's TimedOut arm can fire

One name, every kind of thing you can wait on. The Peer path is `call-by-deadline`'s race, in-tier:

```
kind ← peer.is_socket_tier()     ;; peer-wire?, NEVER env/peer-kind
tmr  ← after(kind, ms, sentinel)
select [peer tmr]
  idx 0 → RecvOutcome from the peer
  idx 1 → RecvOutcome::TimedOut   ;; payload unread
```

`child-main` is one line: `recv` → `recv-by-deadline` + `:wat::service::CHILD-MAIN-STARTUP-DEADLINE-MS`. **No arm changes.**

---

## Acceptance gate (EXPECTATIONS 1–3)

`timeout -k 2 8` on `probe-a-bare-recv-cannot-time-out.wat`:

```
control=TimedOut
negative=Message
subject-starting
```

then SIGKILL (exit 137). No leftover `release/wat`. No `"expected owner handle"`.

- CONTROL: `recv-by-deadline` on a silent Peer prints **TimedOut** — the variant is producible on a Peer.
- NEGATIVE: `recv-by-deadline` on `after 1ms :tick` prints **Message** — a live peer still answers.
- SUBJECT: bare `recv` still blocks (`subject-starting`, then killed). The differential holds.

SIGTERM did not stop the blocked SUBJECT (SIGKILL required) — same fact as the 125 s hang this session measured. A *bounded* wait is the thing that becomes killable; the bare recv is still unkillable by ordinary means. That is the evidence, not a leftover bug in the new path.

---

## STOP-1 — no arm change

`git diff wat/service.wat` child-main: **one line**. The six-arm match is untouched.

Trap-door 2, measured from the enum (`types.rs:1872–1934`), not the two `@ret` docs:

| variant | bare `recv` | `recv-by-deadline` |
|---|---|---|
| Message | yes | yes |
| Closed | yes | yes |
| Stopped | yes (docs said Shutdown) | yes |
| TimedOut | never constructed | yes — now on Peer too |
| Lost | yes | yes |
| Malformed | in the enum; child-main already matches it | mapped from `ServiceEvent::Malformed` |

Both primitives return `RecvOutcome`. The `@ret` docs disagreed with each other and with the enum. The enum is the authority. Swap needed no arms.

---

## STOP-2 — tier from `peer-wire?`

`Peer::is_socket_tier()` is `peer-wire?`. Thread timer if false, process/timerfd if true. Not `Env/peer-kind`. `select` refuses mixed tiers; getting this wrong is a runtime refusal. Copied from `call-by-deadline`'s own comment.

---

## STOP-5 — inert

Reserved keyword `:wat::kernel::__recv_by_deadline_timer__`. Discriminated by **idx**, never by value: a peer that sent this keyword would still be `Message` at idx 0. Same `__` family as the crash/sever sentinels. Not a fourth parameter (arity stays 2; existing Thread/Process sites unchanged).

---

## STOP-3 / STOP-4

- **No `:deadline-ms` clause revived.** The constant is `:wat::service::CHILD-MAIN-STARTUP-DEADLINE-MS`. D4's refusal of that clause name is untouched (the comment that mentions it is "NOT a :deadline-ms clause").
- **Only `child-main` converted.** `owner-recv-loop` already used `recv-by-deadline`. The other 258 bare recvs stand.

---

## The startup constant: **30000 ms**

Named `:wat::service::CHILD-MAIN-STARTUP-DEADLINE-MS`. Why that number:

- The owner sends the ship **immediately after spawn**. A healthy spawn-and-send is milliseconds.
- A wait of tens of seconds is already pathological (DESIGN).
- This session: a blocked bare recv ignored SIGTERM for **125 s**; `timeout 30` never returned. 30000 ms sits inside that window — the hang becomes a named death *before* the operator reaches for `kill -9`.
- ~3 orders of magnitude above a healthy send, not so tight that a loaded-CI `:init` false-fires.
- Named, so it is not the `inbox-cap 64` mistake.

What happens on timeout is **unchanged**: the existing arm raises *"the peer is alive and silent"* — now true and reachable.

---

## WHAT LANDED

- `src/runtime.rs` — Peer arm of `recv-by-deadline`; `after_timer_peer` shared with `after`; `eval_peer_select_values` shared with `select`; idx-mapped `RecvOutcome`.
- `src/intrinsic/kernel/message.rs` — `@arg`/`@ret` docs (six variants; Peer admitted).
- `src/check.rs` — comment only. `infer_recv_by_deadline` already used `project_peer_io` (Peer was a check-time admit and a runtime refusal).
- `wat/service.wat` — one line in `child-main` + the named constant. Stdlib rebuilt.
- `wat-scripts/scratch-pad/probe-a-bare-recv-cannot-time-out.wat` — CONTROL TimedOut, NEGATIVE Message, SUBJECT still blocks.

Nothing else. 258 other bare recvs, send/readln/accept/poll, and recoverable timeout: out of scope.
