# BRIEF — arc 291 strike-3a-ii-β: `stop` becomes owner-only (annihilate the client backdoor)

**You are a LEAF executor. Do NOT spawn subagents. Model: sonnet.** If the work exceeds the rooms
below or hits a STOP trigger, STOP and report — do not improvise a workaround.

## The work, in one paragraph

Today `defservice` auto-generates a **client** `stop` op: a client holding the dial-`Address'` can send
`Op::Stop` and terminate the service. That is an ambient-authority backdoor. This strike **annihilates the
client stop path** and **relocates** lifecycle stop to the owner-only **admin channel** (the lineage peer
held only inside the `Handle`): `(<svc>/stop h)` takes the **Handle**, sends `Admin::Stop` down the lineage
peer, the serve loop replies `LineageUp::Final(state)` and terminates, and the owner recvs the final state.
After this, a client *cannot* stop — by construction (the `stop` method's parameter is the `Handle`, an
unforgeable reference the client never holds; `Op::Stop` no longer exists on the client enum). The 3a-ii-α
lineage protocol (`Admin`/`LineageUp` defenums + the startup handshake) is already shipped — you are making
the serve loop and the Handle method USE the `Admin::Stop` / `LineageUp::Final` variants that already exist.

## Context already on disk (read these first, in order)

1. `docs/arc/2026/06/291-defservice-durable-state/STRIKE-3a-facet-split.md` — §"3a-ii RESOLVED" (the
   symmetric protocol) + §"3a-ii-α SHIPPED" + §"β implication" (the thread-self-peer trap). THE design.
2. `wat-tests/service-admin-facet.wat` — the RED probe (ignore-marked). Your kill un-ignores it GREEN.
   It asserts: client increments via dial-`Address'`; the Handle-holder calls `(admin-counter/stop h)`;
   final count == 7. `stop` takes `h` (a `Handle`), NOT a client peer.
3. `wat/service.wat` — the defservice macro (the bulk of the work).
4. `wat/spawn.wat` — `Launched<S,R>` (`:155`), `Locus/launch` protocol (`:210`), Thread/Process impls,
   `spawn-program'` (`:175`), `Spawned` marker + `Thread'`/`Process'` derives (`:135-140`).

## THE PRECISE ANNIHILATION SET (the qualified kill — remove EXACTLY these, no more)

These are used ONLY by the auto **client** stop op (verified: the per-user-op `Outcome::Stop` arm at
`service.wat:489-495` replies with the op's OWN `~reply-variant-kw`, NOT `Reply::Stop` — so `Outcome::Stop`
the enum variant is a LIVE user capability and STAYS). Remove, in `wat/service.wat`:

- `stop-req-name` / `stop-req-record` (the `StopRequest` record) + its `request-records` conj (`~546`).
- `stop-resp-name` / `stop-resp-fields` / `stop-resp-record` (`StopResponse`) + `response-records` conj (`~547`).
- `Op::Stop` variant: the `variants` conj `(conj (conj variants :Stop) stop-op-req-field)` (`~548`) → back to
  plain `variants`. Drop `stop-op-variant-kw`, `stop-op-req-field`.
- `Reply::Stop` variant: the `reply-variants` conj (`~549`) → plain `reply-variants`. Drop
  `stop-reply-variant-kw`, `stop-reply-resp-field`, `stop-resp-acc`.
- `stop-serve-arm` (the auto `((Op::Stop req) …)` arm, `~524-535`) + its `serve-op-arms` conj (`~550`).
- `stop-ctor-name` / `stop-ctor` + its `constructors` conj (`~710`).
- the CLIENT `stop-method` (`~691-708`: `stop-method-name`/`-params [c <- client-peer-ty]`/`-body`/
  `stop-discard-sym`/`stop-r-sym`) + its `methods` conj (`~711`). REPLACED (below).
- `client-peer-ty` IFF it is now unused after the client `stop-method` dies (grep it; if other methods use
  it, keep it). STOP-3 if unsure.

**KEEP:** `Outcome::Stop` (the enum variant + the per-user-op arm at `:489-495`) — live user capability.

## THE RELOCATION (add the owner-only admin stop)

1. **serve-loop `Admin::Stop` dispatch** (`service.wat` serve-body, `~559`): the `ServiceEvent::Admin` arm is
   currently a re-loop STUB `((ServiceEvent::Admin _admin-msg) (serve self l clients state))`. Make it
   dispatch: match the admin msg — `Admin::Stop` → `(do (send' self (<fqdn>::LineageUp::Final state)) nil)`
   (reply final state UP the lineage peer, then terminate: return nil, no recur); `Admin::Init` → protocol
   error post-startup (`assertion-failed!` OR re-loop — pick assertion-failed!, it can't legitimately arrive).
   Use the already-bound `lineage-up-ty` / a new `lineage-final-kw` (mirror `admin-stop-kw` at `:236`).

2. **`Handle.handle` re-type** (`service.wat:838`): `handle <- :wat::spawn::Spawned` →
   `handle <- :wat::kernel::Peer'<~admin-ty, ~lineage-up-ty>` (sends `Admin`, recvs `LineageUp`).

3. **new owner `stop` method** (replaces the deleted client one): `(<svc>/stop [h <- Handle] -> state-ty …)`:
   `(let [_ (send' (<fqdn>::Handle/handle h) (<fqdn>::Admin::Stop))
          r (recv' (<fqdn>::Handle/handle h))]
      (match r ((<fqdn>::LineageUp::Final state) state) (_ assertion-failed!)))`.
   Use `symbol-node` for the `_`/`r` binders (hygiene, as the old stop-method did). Add to `methods`.

4. **`Launched.handle` re-type** (`spawn.wat:155-156`): `handle <- :wat::spawn::Spawned` →
   the spawn handle as a peer. The PROCESS impl's `svc` is already `Process'<Admin,LineageUp>` (α uses
   `recv' svc`/`send' svc`). The THREAD impl's lineage self-peer is `Peer'<R,S>` = `Peer'<Reply,Op>`
   (vestigial) and must become `Peer'<LineageUp,Admin>` so the parent `sp` = `Thread'<Admin,LineageUp>`.
   See STOP-1 — this is the trap.

5. **migrate the arc272 probes** (`tests/probe_arc272_rs2_process_stop_returns_final_state.rs` +
   `…_thread_…`): they call `(:my::counter/stop c)` (client peer). Change to `(:my::counter/stop h)` (the
   Handle). They still prove "stop returns final state," now owner-only. Update the `//!` header comment to
   say owner-only/Handle (amend with recognition — note the migration, don't just silently edit).

6. **un-ignore** `wat-tests/service-admin-facet.wat` (remove the two `(:wat::test::ignore …)` lines, `:32`/`:45`).

## Implementation sketch (fill it; do not reinvent the macro's shape)

```clojure
;; service.wat — new let-bindings near the α admin keywords (~:236):
lineage-final-kw (:wat::core::keyword/from-string
                   (:wat::core::string::interpolate "{fqdn-str}::LineageUp::Final" :fqdn-str fqdn-str))
;; serve-body Admin arm (replaces the stub):
((:wat::spawn::ServiceEvent::Admin admin-msg)
  (:wat::core::match admin-msg -> :wat::core::nil
    (~admin-stop-kw (:wat::core::do (:wat::kernel::send' self (~lineage-final-kw state)) nil))
    ((~admin-init-kw _seed) (:wat::kernel::assertion-failed!
       "defservice serve: Admin::Init after startup (protocol error)" :wat::core::None :wat::core::None))))
;; new owner stop method:
stop-method-params `[h <- ~handle-name]
stop-method-body `(:wat::core::let
                    [~stop-discard-sym (:wat::kernel::send' (~handle-handle-acc h) (~admin-stop-kw))
                     ~stop-r-sym       (:wat::kernel::recv' (~handle-handle-acc h))]
                    (:wat::core::match ~stop-r-sym -> ~state-ty
                      ((~lineage-final-kw state) state)
                      (_ (:wat::kernel::assertion-failed!
                           "defservice stop: expected LineageUp::Final" :wat::core::None :wat::core::None))))
;; (~handle-handle-acc = :<fqdn>::Handle/handle accessor keyword)
```

## STOP triggers (halt + report; do NOT improvise)

1. **STOP-1 (the thread lineage-peer typing — THE trap-door):** if re-typing the thread `launch` lineage
   self-peer to `Peer'<LineageUp,Admin>` (so `sp`/`Handle.handle` = `Thread'<Admin,LineageUp>` satisfies
   `Peer'<Admin,LineageUp>`) **requires adding lineage type-params to the `Locus/launch` protocol signature**
   (`launch<S,R,St,Sh>` → `<…,Lu,Ad>`) or otherwise can't be expressed with the current params — STOP and
   report the exact checker error + the minimal signature change you'd propose. Do NOT leave `Handle.handle`
   as `:Spawned` and down-cast, and do NOT make `stop` reach past the typed peer.
2. **STOP-2 (back-compat):** `service-locus-parity.wat` + `service-init-parity.wat` MUST stay green (startup
   is unchanged by this strike). If annihilating the client stop forces an edit to either, STOP.
3. **STOP-3 (annihilation scope):** if removing any item in the annihilation set breaks something OUTSIDE the
   client stop op (e.g. `client-peer-ty` still used by a non-stop method, or `Outcome::Stop` turns out
   referenced by the auto path) — STOP and report what; do not widen or narrow the set on your own.
4. **STOP-4 (exhaustiveness):** removing `Op::Stop`/`Reply::Stop` variants may make some `match` non-
   exhaustive, or the new `Admin` match may need both arms — if a cascade appears that is NOT just the two
   files above, STOP and report the site list (do not edit unrelated files).

## Blast radius (bound it)
`wat/service.wat` + `wat/spawn.wat` + `tests/probe_arc272_rs2_process_stop_returns_final_state.rs` +
`tests/probe_arc272_rs2_thread_stop_returns_final_state.rs` + `wat-tests/service-admin-facet.wat` (un-ignore).
Do NOT touch any other arc's files, any `src/*.rs` (the Rust reactor already delivers `ServiceEvent::Admin`
— 3a-i), or any other `wat/*.wat`.

## Expectations (scorecard — written before the strike)

| what | command | expected |
|---|---|---|
| RED probe goes green, both tiers | `cargo test -p wat --test test admin_stop` | 2 passed (after un-ignore) |
| back-compat startup holds | `cargo test -p wat --test test counter_on` | 4 passed (unchanged) |
| arc272 probes migrated green | `cargo test -p wat --test probe_arc272_rs2_process_stop_returns_final_state --test probe_arc272_rs2_thread_stop_returns_final_state` | 2 passed |
| no client `Op::Stop` remains | `grep -n "Op::Stop\|StopRequest\|stop-method-params .c " wat/service.wat` | only `Outcome::Stop` survives |
| no new workspace regressions | (orchestrator runs `cargo test -p wat --no-fail-fast`, SET-diff vs HEAD) | ⊆ HEAD floor (∅ new) |

Runtime prediction: 30–50 min (annihilation is mechanical; the thread-peer typing is the one novel spot →
STOP-1 if it needs a protocol-sig change). Trap-door: STOP-1 (thread lineage typing) — everything else is
removal + transcription from the α admin keywords + the existing method/serve-arm patterns.
