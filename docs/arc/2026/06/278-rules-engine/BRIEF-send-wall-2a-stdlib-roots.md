# BRIEF — send' OUTCOME WALL, Strike 2a: face the SendOutcome at the STDLIB ROOT sites

> **Tier:** sonnet shadowdancer. **Arc:** 278 send'-wall Phase 2 (see `DESIGN-send-outcome-wall.md` STATUS).
> **Base:** the Phase-1 foundation is in the working tree (`send'` returns `SendOutcome`, 66 RED). This
> strike faces the outcome at the ~25 **stdlib root** send' sites — clearing the deftest + service +
> bracket RED cluster. Do NOT touch test-fixture send' sites (Strike 2b) and do NOT add the checker force
> (Phase 3). Leave uncommitted (the wall lands atomic).

## Why (one paragraph)

`send'` now returns `:wat::kernel::SendOutcome::{Sent, Closed, Lost[cause]}` — a value. Sites that were
`(send' …)` in statement/last position now leak an unfaced outcome, breaking ~40 `deftest_wat_tests_*` (via
the harness) + the service/bracket tests. Face the outcome at each **stdlib** site with its honest,
context-appropriate arms — **never a `_`-swallow** (`(let [_ (send' …)] …)` is the masking this arc
annihilates; R55). The 66 RED reduce to these roots; fixing them clears the bulk.

## The sites + their honest facing (ground each; the pattern is per-context)

Grep the exact sites: `grep -nE ":wat::kernel::send'" wat/test.wat wat/spawn.wat wat/bracket.wat wat/service.wat`.

1. **`wat/service.wat` (~15) — the serve-loop REPLIES** (`(send' (nth selectables idx) (Reply::… resp))`).
   A reply to a **gone client is NOT a service error** — the client left; keep serving. Face:
   ```clojure
   (:wat::core::match (:wat::kernel::send' (:wat::core::nth selectables idx) (~reply resp))
     ((:wat::kernel::SendOutcome::Sent)     (~serve-name self l selectables new-state))
     ((:wat::kernel::SendOutcome::Closed)   (~serve-name self l selectables new-state))   ;; client gone → keep serving
     ((:wat::kernel::SendOutcome::Lost _c)  (~serve-name self l selectables new-state)))  ;; client gone → keep serving
   ```
   (The continuation is the same for all arms — a gone client just means keep serving. For the `Stop` arm
   whose continuation is `nil`, all three arms → `nil`.) These are in the serve-op-arm macro templates —
   keep the quasiquote/unquote valid.

2. **`wat/test.wat:800` — the harness child completion-signal** (`(do ~body (send' self 0))`). The child
   signals success; the PARENT faces the outcome via its `recv'` (`test.wat:801`). So the child's send' can
   proceed regardless — but keep the child body's return honest and DON'T `_`-swallow. Face it:
   ```clojure
   (:wat::core::do ~body
     (:wat::core::match (:wat::kernel::send' self 0)
       ((:wat::kernel::SendOutcome::Sent)    nil)
       ((:wat::kernel::SendOutcome::Closed)  nil)   ;; parent's recv' already faces a gone self-peer
       ((:wat::kernel::SendOutcome::Lost _c) nil)))
   ```
   (This clears the ~40 `deftest_wat_tests_*`.)

3. **`wat/spawn.wat` (~2) + `wat/bracket.wat` (~6 remaining)** — ground each: a send-then-recv' proceeds
   (the recv' faces the death); a pool-setup/work fire faces all three arms explicitly (Sent → continue;
   Closed/Lost → the honest action — usually continue, since a gone pool worker is handled by the
   collect-loop's recv'). Replace the Phase-1 `(let [_ (send' …)] nil)` bracket stub (`bracket.wat:~603`)
   with a real faced match too (it was flagged as a temporary discard).

**Facing rule (hold it):** every arm is explicit; the failure arms do the *honest* thing for that site
(keep serving / proceed to recv' / the collect-loop faces it) — NOT a silent `_`. If a site's honest
failure action is genuinely "nothing" (a fire whose gone-peer is truly irrelevant), an explicit
`((Lost _c) nil)` arm is fine (faced + chosen), but it must be the explicit arm, never `(let [_ …] …)`.

## STOP triggers

- **STOP-0:** you touch a **test-fixture** `.rs`/`.wat` send' site (arc-259/214/293 peer tests) — that's
  Strike 2b. Or you add the Phase-3 checker force. STOP. Scope is the 4 stdlib `.wat` files only.
- **STOP-1:** a serve-loop site's honest failure action ISN'T "keep serving" (some reply site where a gone
  client IS fatal) — STOP on it, report; don't guess.
- **STOP-2:** the facing breaks a macro quasiquote/template — STOP, report the exact error; don't
  restructure the macro.

## Verify (weigh by your own re-run)

1. `./target/release/wat --check` clean on every edited `.wat`.
2. Floor: `cargo nextest run --release 2>&1 | tee /tmp/claude-scout/sendwall_2a_floor.log` — READ the
   Summary. The count MUST DROP from 66 toward green: the ~40 `deftest_wat_tests_*` + the service/bracket
   tests should now PASS. Report the new count + the REMAINING RED list (that's Strike 2b's worklist — the
   peer test fixtures). It will NOT be fully green yet (the test-fixture sites are Strike 2b).

## Deliverable

The ~25 stdlib root sites faced. Report: (1) the sites faced + their arms; (2) the floor count drop
(66 → N) + the remaining RED list; (3) `git diff --stat`. Do NOT commit (the wall lands atomic).

## Blast radius

`wat/service.wat`, `wat/test.wat`, `wat/spawn.wat`, `wat/bracket.wat` only. NO test fixtures, NO checker
force, NO `src/`. Scratch logs → `/tmp/claude-scout/`.
