# FINDING — the classification (a send that SHOULD be bounded)

**Struck 2026-09-16 by a spawned Opus subagent. REPORT-ONLY — no `.wat` and no `.rs` was touched.**

The recv twin (`a-wait-that-should-be-bounded/FINDING-the-classification.md`) is the shape this copies,
including its §0. Every row below cites a `file:line` **opened and read in context** at HEAD
`220e0aa7d`, branch `sns-sqs`. Enclosing-form line numbers are given so a re-read lands in the same
place.

---

## 0. ⚠ The instrument, before the table

### 0.1 What was run

`wat-scripts/census-waiter-bounds.wat`, invoked exactly as its own header specifies (a sorted EDN
vector of paths on stdin):

```bash
find wat wat-scripts tests docs -name '*.wat' | sort \
  | python3 -c 'import json,sys; print(json.dumps([l.strip() for l in sys.stdin if l.strip()]))' \
  | ./scripts/capped.sh --limit 8g ./target/release/wat ./wat-scripts/census-waiter-bounds.wat
```

Today's run, verbatim from its own summary block:

```
waiter-sites=684  yes=22 no=644 UNKNOWN=18
  recv yes=0 no=256 UNKNOWN=0
  recv-by-deadline yes=9 no=0 UNKNOWN=0
  select yes=7 no=3 UNKNOWN=4
  poll yes=0 no=0 UNKNOWN=14
  accept yes=0 no=8 UNKNOWN=0
  send yes=0 no=216 UNKNOWN=0
  try-send yes=6 no=0 UNKNOWN=0
  readln yes=0 no=161 UNKNOWN=0
```

**Live = `home=wat` or `home=wat-scripts/service`** (the census's `home-of`: `wat-scripts/service` is
`wat-scripts/queue/`, `wat-scripts/topic/`, `wat-scripts/fanout/`). Filtering the `primitive=send`
rows on that gives **32**, spread `wat/service.wat` 17 · `wat/bracket.wat` 8 ·
`wat-scripts/fanout/circuit.wat` 2 · `wat-scripts/queue/sqs.wat` 2 · `wat/spawn.wat` 2 ·
`wat/test.wat` 1. **That total is re-derived here, not quoted** — see §3.

### 0.2 ⛔ What it sees — and the grep that would have hidden a BOUND site

Independent cross-check, `grep` in **call position** (an open paren immediately before the head),
excluding `try-send` by requiring a non-letter/non-dash before the head and refusing a trailing letter:

```bash
grep -rhoE '\(:wat::kernel::send([^a-zA-Z-]|$)' --include=*.wat wat wat-scripts/queue \
    wat-scripts/topic wat-scripts/fanout
```

→ **32.** The grep and the census agree **exactly** on the live set. But the `|$` alternative is
load-bearing and it is the recv twin's §0 defect reproduced on this stone:

```bash
# WITHOUT the |$ — the shape the recv twin's BRIEF was bitten by
$ grep -noE ':wat::kernel::send[^a-zA-Z-]' wat/bracket.wat | wc -l
7        # census says 8
```

The missing one is **`wat/bracket.wat:643`**, where the head ends the line and the peer expression
starts on the next:

```
643|  (:wat::core::match (:wat::kernel::send
644|     (:wat::core::nth peers peer-pos)
```

⛔ **That site is on the BOUND list (row 6 of §2, `collect-loop`'s work dispatch).** A trailing-character
grep would have dropped a defect-list member from a 32-site table and nothing would have said so.

### 0.3 Where the census and the grep DISAGREE — and why the census is right

Over the census's own universe (`wat wat-scripts tests docs`) the corrected grep finds **221** text
occurrences against the census's **216** call sites. All five of the delta are `:wat::kernel::send`
appearing **inside a comment**, which the form-tree walk cannot see and should not:

| file | line | what it is |
|---|---|---|
| `tests/cli/wat_cli__check_bad.wat` | 9 | `;; \`(:wat::kernel::send no-such-thing 42)\`, whose first diagnostic was` |
| `wat-scripts/fixes/face-underscore-bound-send-prime.wat` | 5, 12, 14 | the codemod's header drawing the shapes it rewrites |
| `wat-scripts/scratch-pad/probe-handler-issues-request-and-returns.wat` | 9 | `;; OUTBOUND — YES. \`(:wat::kernel::send st op)\` inside a handler is …` |

None is live. The census is the stricter instrument and **216 is the number to quote**.

### 0.4 ⛔ What the instrument CANNOT see

1. **Four directories of `.wat` are outside its universe.** The header's own `find` line names
   `wat wat-scripts tests docs`. On disk there are also `wat-tests/` (85 files), `examples/` (6),
   `benches/` (1), `crates/` (3), carrying **20 further send call sites** (grep). So "216 corpus-wide"
   is **216 over four of eight directories**; the true `.wat` figure is **241** by grep, **≈236**
   discounting the 5 comment hits. ⭑ This does **not** move the live set: `home-of` maps anything
   outside `wat/ wat-scripts/ tests/ docs/` to `other`, so every one of the 20 is non-live by
   construction. It matters for the *ratio* the mechanism ruling will quote, not for this table.
   (This is the recv twin §6.2 lesson arriving on the send side: *a stdlib symbol's callers are not in
   the stdlib's own directory.*)
2. **`src/` is invisible.** Rust-side sends are not `.wat` call heads and were not counted.
3. **An indirect send would be invisible** — a `send` reached through `apply` with a keyword head would
   be attributed to `:wat::core::apply`. Checked: `grep -rn 'apply *:wat::kernel::send' --include=*.wat .`
   returns **nothing**, so no live site is lost this way.
4. **`bounded=` is meaningless for `send`.** `census::bound-of` has no send arm — every send falls
   through to `(Tuple "no" "none")`. `send yes=0 no=216` is therefore **not a measurement that 216
   sends are unbounded**; it is the instrument reporting that *no bounded-send form exists to detect*.
   That is consistent with `a-send-cannot-say-it-is-blocked`, which proved the form is missing, but the
   `no=216` column must not be cited as evidence on its own.
5. **The DESIGN's line numbers are stale by 21 lines.** It says to start at `wat/spawn.wat:527` and
   `:597`; at HEAD both are **comment lines**. The sends are at **548** and **618** (the shift is exactly
   +21 at both, from piece A `e03fc7f46` landing its match arms). Every row below uses today's numbers.

---

## 1. The three controls — predicted BEFORE reading, one DISAGREED

Predictions were written down from the census row text alone, before any of the three files was opened.

| # | control | predicted | read as | verdict |
|---|---|---|---|---|
| 1 | `wat/spawn.wat:548` (`ThreadOpts` `launch`) | **BOUND** | **UNKNOWN** | ❌ **DISAGREE** |
| 2 | `wat/bracket.wat:44` (`:wat::bracket::runner-loop`) | **PARK** | **REPLY** | ❌ **DISAGREE** |
| 3 | `wat/service.wat:2588` (serve loop, `ServiceEvent::Malformed`) | **REPLY** | **REPLY** | ✅ agree |

⭐ **Two of three disagreed, and both disagreements are the finding.**

**Control 1 — DISAGREE, and it inverts the DESIGN's own framing.** I predicted BOUND because the
DESIGN says `spawn.wat:527`/`:597` are *"the SEND halves of the handshake `ab419aaa3` bounded on the
recv side — `launch` sends the ship, then waits."* Reading the two sites: **that is true of `:618`
and false of `:548`.** At 548 it is the **CHILD** sending its readiness announcement **UP** (inside the
`fn` passed to `spawn-program`, `spawn.wat:541`), and the parent's bounded `recv-by-deadline` at
**573** is the *other end of that same message*, not a second wire. The two are not "send half + recv
half"; they are sender and receiver of one frame. See §5 for the verdict this forces.

**Control 2 — DISAGREE.** I predicted PARK because the recv twin classed the `recv` **five lines above**
(`bracket.wat:39`) as PARK. The recv is the worker idling; the **send is the worker's answer**, and
`collect-loop`'s exit condition is `(:wat::core::= collected m)` (`bracket.wat:629`) — a dropped result
means `collected` never reaches `m` and the *coordinator* hangs instead. Dropping is worse than
blocking ⇒ REPLY, not PARK. ⭑ **The class of a `recv` does not transfer to the `send` in the same
loop**; a worker loop is PARK on the way in and REPLY on the way out.

**Control 3 — agree.** `wat/service.wat:2588`, the `ServiceEvent::Malformed` arm, replies
`Reply::Failed[cause]` to the one client that sent garbage and keeps serving everyone else. The comment
at 2579–2584 says it outright: *"the caller is never left blind."*

---

## 2. Every live site — all 32

`enc` = enclosing form, with its own line number.

### `wat/service.wat` — 17

| # | file:line | enclosing form | what the send hands over | class | reason |
|---|---|---|---|---|---|
| 1 | `wat/service.wat:2245` | `defservice` (macro body), `serve-op-arms` `Outcome::Stop` arm (2240) | the final `Some resp` reply to the requesting client, immediately before the loop terminates | **REPLY** | The client is mid-call. Dropping it leaves that caller with no answer from a service that is about to be gone; a bound changes what the client receives, not when. |
| 2 | `wat/service.wat:2283` | `defservice`, `Outcome::Faulted` arm (2282) | `Status::Faulted[cause]` **up the lineage to the owner**, then recurse into serve with pre-op state | **BOUND** | Unsolicited status, not an answer to a request. A non-draining owner wedges the serve loop and every client with it, while the one path that *does* read `Faulted` (`/grant`, 3480) already tolerates its absence via `owner-recv-loop`'s `GaveUp`. Dropping is strictly cheaper than blocking ⇒ it wants a bound. |
| 3 | `wat/service.wat:2402` | `defservice`, `shape-guarded` (2391), `Validation::Invalid` arm (2396) | `Reply::…Response::RequestMalformed[path exp got]` to that client | **REPLY** | A 400. Handler never ran, state unchanged; every arm keeps serving. Dropping blinds the caller. |
| 4 | `wat/service.wat:2413` | `defservice`, `guarded-arm` (2408) | `…::RequestTooLarge[n cap]` to that client | **REPLY** | Same refusal protocol, size guard. |
| 5 | `wat/service.wat:2434` | `defservice`, `guarded-final` (2431) | `…::RequestTooManyEntries[k cap]` to that client | **REPLY** | Same refusal protocol, entry-count guard. |
| 6 | `wat/service.wat:2491` | `defservice`, `serve-body` (2478), `Admin::Stop` arm (2490) | `Status::Stopped[(stop-project state)]` to the owner, then terminate | **REPLY** | The owner is blocked in `owner-recv-loop` for exactly this frame. Dropping it makes `/stop` report `GaveUp` for a stop that actually happened — a protocol lie, not a delay. |
| 7 | `wat/service.wat:2497` | `defservice`, `serve-body`, `Admin::Hibernate` arm (2496) | `Status::Hibernated[(hibernate-project state)]` to the owner, then terminate | **REPLY** | Twin of row 6; `/hibernate` shares the shape (3328–3334). |
| 8 | `wat/service.wat:2517` | `defservice`, `Admin::AllowPeer` arm (2506) | `Status::PeersAllowed` ack after `allow` already ran (2511) | **REPLY** | ⭑ The grant is **already applied** when this fires. Dropping it reports `GaveUp` for a grant that landed, and the comment at 2502–2505 says the ack exists so *"grant-before-dial ordering holds"*. |
| 9 | `wat/service.wat:2537` | `defservice`, `Admin::DenyPeer` arm (2526) | `Status::PeersDenied` ack after `deny` already ran (2531) | **REPLY** | Mirror of row 8, same already-applied hazard. |
| 10 | `wat/service.wat:2588` | `defservice`, `serve-body`, `ServiceEvent::Malformed` arm (2585) | `Reply::Failed[cause]` to the client that sent an undecodable frame; client is KEPT | **REPLY** | Control 3. *"the caller is never left blind"* (2581). |
| 11 | `wat/service.wat:3198` | `stop-method-body` (3197) → generated `<fqdn>/stop` | `Admin::Stop` **down** the lineage from the owner | **BOUND** | ⛔ The kill path. `owner-recv-loop` is wall-clock bounded at 10000 ms — but `t0` is taken at **3203, AFTER the send**, and `owner-recv-loop`'s own header opens *"send already happened"* (4330). A service not draining its Admin channel makes `/stop` block before its budget has even started. A stop that cannot stop. |
| 12 | `wat/service.wat:3336` | `hibernate-method-body` (3335) → generated `<fqdn>/hibernate` | `Admin::Hibernate` down the lineage | **BOUND** | Exact twin of row 11; `t0` at 3341, after the send. |
| 13 | `wat/service.wat:3469` | `grant-method-body` (3466), process arm → generated `<fqdn>/grant` | `Admin::AllowPeer[pids]` down the lineage | **BOUND** | Same shape; `t0` at 3474, after the send. `GateOutcome` already has a `GaveUp` variant (3546) — the *recv* side can report giving up and the *send* side cannot. |
| 14 | `wat/service.wat:3572` | `revoke-method-body` (3569), process arm → generated `<fqdn>/revoke` | `Admin::DenyPeer[pids]` down the lineage | **BOUND** | Mirror of row 13; `t0` at 3577. |
| 15 | `wat/service.wat:3784` | `child-main-form`'s generated `:user::main` (let opened 3752) | `Status::Started[(Bound/address b)]` **up** to the owner, then straight into `serve` | **UNKNOWN** | Process-tier twin of `spawn.wat:548`. See §6. |
| 16 | `wat/service.wat:4294` | `defn :wat::service::send-keep-serving?` (4288) | the payload of **every deferred/directed serve-loop reply** — one site, five call sites (2086, 2120, 2209, 2232, 2271) | **REPLY** | ⭑ Helper-consumed, followed per the trap. Its own comment (4290–4291) states the protocol that makes it REPLY *and* shows the bound is affordable: *"A drop is not Stopped: do not send, still return true. The caller waits; T1's deadline turns that into timeout → discard → redial → retry."* The caller-side deadline already covers a dropped reply. |
| 17 | `wat/service.wat:4599` | `defn :wat::service::call-by-deadline` (4591) | the client's `op` — the one round-trip the whole deadline API is built around | **BOUND** | ⛔⛔ **The sharpest site in the corpus, and it is a contract violation by name.** `race-reply` (the timer) is invoked **inside the `Sent`/`Closed`/`Lost` arms** (4602–4604) — the deadline does not exist until the send returns. `call-by-deadline` can block without limit. `CallOutcome` has `DeadlineFired` (4307) and it cannot fire here. |

### `wat/bracket.wat` — 8

| # | file:line | enclosing form | what the send hands over | class | reason |
|---|---|---|---|---|---|
| 18 | `wat/bracket.wat:44` | `defn :wat::bracket::runner-loop` (32), `RecvOutcome::Message` arm (40) | `(work-fn item)` — the worker's **result** back to the pool parent | **REPLY** | Control 2. `collect-loop` exits only on `collected = m` (629); a dropped result hangs the coordinator. Dropping worse than blocking. |
| 19 | `wat/bracket.wat:86` | `defn :wat::bracket::process-runner` (71), `PoolMsg::Work` arm (81) | `(Tuple idx (work-fn item))` — indexed result | **REPLY** | Process-tier twin of row 18, same collect-loop contract. |
| 20 | `wat/bracket.wat:144` | `defn :wat::bracket::process-dial-runner` (114), `PoolMsg::Work` arm (138) | `(Tuple idx (work-fn held-peer item))` — indexed result from the dialing worker | **REPLY** | Same, dialing cousin. |
| 21 | `wat/bracket.wat:214` | `defn :wat::bracket::thread-kwargs-runner` (190), `PoolMsg::Work` arm (208) | `(Tuple idx ($impl item ctx))` — indexed result, kwargs tier | **REPLY** | Same, kwargs tier. |
| 22 | `wat/bracket.wat:497` | `runner-def` quasiquote (479) inside `defclause :wat::bracket::process-work-forms` → generated `defn :user::bracket::dial-runner` (480) | `(Tuple idx (:user::bracket::work-fn item k))` — indexed result from the generated n-dial runner | **REPLY** | One source site, expands per pool; the generated body is rows 18–21's loop. ⚠ Only visible because the census walks **into the quasiquote**. |
| 23 | `wat/bracket.wat:643` | `defn :wat::bracket::collect-loop` (620), `ServiceEvent::Message` arm (635) | `PoolMsg::Work[(Tuple cursor items[cursor])]` — the **coordinator dispatching** the next item to the runner that just answered | **BOUND** | ⛔ The site the naive grep loses (§0.2). The coordinator has N peers; blocking on one stalls the whole map/reduce while the others sit idle, and `map-worker`'s caller is owed a result vector with no arm to say why it never came. ⚠ A bound here must **report**, not skip: `dispatched = -1` already advances `cursor` (652) without the item landing, and `collected` then never reaches `m`. |
| 24 | `wat/bracket.wat:787` | `defn :wat::bracket::map-worker` (≈737), `peers` `mapv` closure (756), setup fold (781) | `PoolMsg::Setup[c]` — the per-worker dial/assembly carrier, before any Work | **BOUND** | Pool construction, before `collect-loop` exists to observe anything. A blocked Setup hangs `map-worker` with the pool half-built and nothing watching. |
| 25 | `wat/bracket.wat:797` | `defn :wat::bracket::map-worker`, same `mapv` closure (756) | `PoolMsg::Work[(Tuple i items[i])]` — the **initial per-worker primer** | **BOUND** | Same window: the primers are sent inside `mapv` *before* `collect-loop` is entered at 804. Blocking here hangs before a single `select` has run. |

⛔ **Defect found in passing at `wat/bracket.wat:789–790` — reported, NOT fixed (report-only).** The
`PoolMsg::Setup` match has a **duplicated arm**:

```
787|  (:wat::core::match (:wat::kernel::send p (:wat::bracket::PoolMsg::Setup c))
788|    (:wat::kernel::SendOutcome::Sent   nil)
789|    (:wat::kernel::SendOutcome::Stopped nil)  ;; arc 278 #73 — same: collect-loop's select' arm faces the stop
790|    (:wat::kernel::SendOutcome::Stopped nil)  ;; arc 278 #73 — same: collect-loop's select' arm faces the stop
791|(:wat::core::SendOutcome::Closed nil)   ;; surfaces via collect-loop's select' arm
```

`git blame`: **`86b30d5dce`, 2026-08-04**, committed, on the floor, green ever since. Line 790 is dead
by construction and the mangled indentation at 791 is the same edit. **The checker admits a duplicate
`SendOutcome` arm.** That is directly load-bearing for the mechanism this stone informs: the
`an-outcome-arm-cannot-be-a-wildcard` argument for (b) is *"28 exhaustive sites mean the COMPILER finds
them"* — a checker that does not reject a duplicated arm may also accept a codemod that duplicates one
instead of adding `TimedOut`. Separate stone; named here so it is not lost. *(Exact text at 791 is
`(:wat::kernel::SendOutcome::Closed nil)` — re-indented above only to fit the block.)*

### `wat/spawn.wat` — 2

| # | file:line | enclosing form | what the send hands over | class | reason |
|---|---|---|---|---|---|
| 26 | `wat/spawn.wat:548` | `extend-type :wat::spawn::ThreadOpts :wat::spawn::Locus` (529), `launch` (530), inside the child closure passed to `spawn-program` (541) | `(lu-mk-kw (Bound/address b))` — the **child's** `Status::Started` announcement **UP**, after `:init`, before entering `serve` | **UNKNOWN** | §5 + §6. |
| 27 | `wat/spawn.wat:618` | `extend-type :wat::spawn::ProcessOpts :wat::spawn::Locus` (601), `launch` (609) | `ship` (`Admin::Init`) — the **parent's** startup ship **DOWN** to the freshly spawned process | **BOUND** | ⭑ **This one is half a fence, unambiguously.** The parent sends at 618 and only then waits at 630 under `startup-handshake-deadline-ms`. A child that has not yet reached its own `recv-by-deadline` (`service.wat:3763`) — slow exec, blocked loader, wedged `:init` — is not draining, and the parent blocks *before* its deadline is armed. `ab419aaa3` fenced 630 and left 618 open. |

### `wat/test.wat` — 1

| # | file:line | enclosing form | what the send hands over | class | reason |
|---|---|---|---|---|---|
| 28 | `wat/test.wat:402` | `defmacro :wat::test::run-thread` (385), inside the quasiquoted child `fn` (397), after `~body` | `0` — the child test's **pass-marker** up to the holder | **UNKNOWN** | Thread-tier twin of row 26; the holder's wait is `recv-by-deadline` at `wat/test.wat:335`. §6. |

### `wat-scripts/fanout/circuit.wat` — 2

| # | file:line | enclosing form | what the send hands over | class | reason |
|---|---|---|---|---|---|
| 29 | `wat-scripts/fanout/circuit.wat:3819` | `defn :user::deadline-redial-is-fresh` (3807), `first` binding (3814) | `Hold::Op::Wait[WaitRequest]` on the first-dialled peer to a **deliberately silent** service | **BOUND** | The fn's whole claim is *"the deadline is honest"* (3806) — and the 200 ms timer is built at **3825, inside the `Sent` arm**, after the send. A blocked send makes the proof hang and print nothing; the caller (`:user::main`, 3865) is owed a result string. Same structural defect as row 17, in a demo that exists to demonstrate deadlines. |
| 30 | `wat-scripts/fanout/circuit.wat:3852` | same `defn` (3807), `retry` binding (3846), `ConnectOutcome::Connected` arm (3851) | the retry `Hold::Op::Wait` on the **freshly redialled** peer | **BOUND** | Same silent service, and this send has **no timer at all** — the code *"require[s] that send to land (Sent)"* (3849). Nothing bounds it. |

### `wat-scripts/queue/sqs.wat` — 2

| # | file:line | enclosing form | what the send hands over | class | reason |
|---|---|---|---|---|---|
| 31 | `wat-scripts/queue/sqs.wat:2071` | `defn :user::park-receive!` (2065), `let` binding 1 (2070) | `Queue::Op::Receive[…:wait wait]` — the long-poll **lodgement** | **BOUND** | ⭑ **The trap site, followed.** The outcome is not discarded: it is the **argument to `:user::send-ok!`** (2054), which matches all four arms. The *service* parks; the *send* does not — it returns once the frame is accepted. The caller is then owed the envelopes (`recv-envelopes!`, 2085). A blocked lodgement hangs a caller that has a wait protocol and cannot use it. |
| 32 | `wat-scripts/queue/sqs.wat:2076` | `defn :user::park-receive!` (2065), `let` binding 2 (2075) | `Queue::Op::Stats[StatsRequest]` — the **barrier** whose reply confirms the park is lodged | **BOUND** | Also consumed by `send-ok!`. The barrier `recv` at 2077 is bare and the assertion at 2081 names it *"expected Stats reply as barrier"*; a blocked send never reaches the barrier at all. |

⭑ **`:user::send-ok!` already anticipated this stone in prose.** `sqs.wat:2050–2052`: *"a variant ADDED
to `SendOutcome` later (the `TimedOut` of `a-send-cannot-say-it-is-blocked`) would have been absorbed
here too, at the only send-facing helper in this file (both of `park-receive!`'s sends go through it)."*
Piece A's fix is exactly what makes rows 31–32 classifiable.

---

## 3. Totals — re-derived

| class | count |
|---|---|
| **REPLY** | **15** |
| **BOUND** | **14** |
| **UNKNOWN** | **3** |
| **PARK** | **0** |
| **live total** | **32** |

15 + 14 + 3 + 0 = **32.** ✅ Matches the DESIGN's 32 — re-derived two ways (census `home=` filter, and
the corrected call-position grep of §0.2, which also returns 32).

Per file: `service.wat` 17 = 10 REPLY + 6 BOUND + 1 UNKNOWN · `bracket.wat` 8 = 5 REPLY + 3 BOUND ·
`circuit.wat` 2 = 2 BOUND · `sqs.wat` 2 = 2 BOUND · `spawn.wat` 2 = 1 BOUND + 1 UNKNOWN ·
`test.wat` 1 = 1 UNKNOWN.

**Non-live: 184** (216 census-universe − 32), as the DESIGN says — plus a further **20** in
`wat-tests/ examples/ benches/ crates/` that the census's universe never looked at (§0.4.1), all
non-live by `home-of` construction. Tests and probes; counted, not classified.

### ⭐ PARK is ZERO, and that is a structural fact, not an accident

The recv half had **7 PARK of 23**. The send half has **0 of 32**. A `recv` can be an idle worker with
nothing to do; a `send` is always an act of handing something to a specific someone, so "blocking IS
the loop" never applies. ⛔ **Consequence for the ruling: there is no class of send site for which
blocking is correct by design.** Every one of the 32 either wants a bound (14) or is a reply whose bound
is a protocol decision (15), or turns on a lifetime question (3). Nothing on this table is a site where
a bound would be *wrong*; the only question is what the bound should *do*.

---

## 4. ⛔ The BOUND list — 14 sites, in four species

| species | sites | the hang |
|---|---|---|
| **S1 — the deadline that starts too late** (4) | `service.wat:4599` · `circuit.wat:3819` · `circuit.wat:3852` · `bracket.wat:643` | A timer/deadline exists and is armed **after** the send returns. `call-by-deadline` is the flagship: `CallOutcome::DeadlineFired` cannot fire while the send blocks. |
| **S2 — the owner-side admin verb** (4) | `service.wat:3198` `/stop` · `:3336` `/hibernate` · `:3469` `/grant` · `:3572` `/revoke` | `t0` is taken after the send (3203, 3341, 3474, 3577); `owner-recv-loop`'s header says *"send already happened"* (4330). The 10000 ms budget never starts. **This includes the kill path.** |
| **S3 — the pool built before anyone is watching** (2) | `bracket.wat:787` · `:797` | Setup and primer sends run inside `mapv` (756) before `collect-loop` is entered (804). No `select` is running to observe anything. |
| **S4 — the request whose outcome a helper already faces** (4) | `sqs.wat:2071` · `:2076` (via `send-ok!`) · `service.wat:2283` (`Status::Faulted` to the owner) | Piece A gave these an outcome to read; there is no variant that says "blocked". `service.wat:2283` is the one where a blocked send wedges a **serving** loop rather than a dying one. |

⭑ **S1 and S2 are the same defect wearing two coats: nine of the fourteen are a bounded wait whose
send is not bounded — the half-fence, generalised.** `both-ends-bound-the-same-handshake` was struck
against exactly this drift, on the recv side. The send side has nine of them.

⭑ **And the corpus already contains the fix, applied once.** `wat/service.wat:2605–2611`, the
`ServiceEvent::Rejected` arm, is the corpus's **only live `try-send`**, and its comment (2599–2603) is
this entire finding stated three weeks early:

> *"The reply is a NON-BLOCKING `try-send'` (the deadlock guard): a client blocked mid-send on an
> extreme oversized frame is not reading its reply side, so a blocking `send'` could wedge the serve
> loop — `try-send'` skips a non-draining client and we still evict (it learns via EPIPE on its own
> send)."*

That is a REPLY site that was successfully bounded, and it shows the shape: **bound + evict**, so the
wire is not left desynced by the dropped frame. It is the exemplar, cited not described.

---

## 5. ⭑ `wat/spawn.wat:548` and `:618` — the verdict asked for, and it splits

The DESIGN asks one question about two sites and **they do not have the same answer.** Both its line
numbers are stale (+21; §0.4.5) and, more importantly, its description — *"`launch` sends the ship,
then waits"* — fits only one of them.

**`:618` (ProcessOpts) — YES, leaving it unbounded is half a fence.** The parent sends `ship` at 618,
*then* waits at 630 under `startup-handshake-deadline-ms`. A child that is alive but not yet draining
blocks the parent **before** its own deadline is armed. `ab419aaa3` fenced the wait and left the ship
open; this is the literal half-fence and it belongs on the BOUND list.

**`:548` (ThreadOpts) — NO, and the DESIGN's framing is wrong about it.** 548 is not the parent's ship:
it is the **child** announcing readiness upward, inside the closure at 541, and the parent's bounded
`recv-by-deadline` at **573 is the receiver of that very frame**. There is no second wire. The
comments say the design out loud (545–547, 552–557): *"the child's own `send'` here just needs to
proceed regardless… the parent's crash-aware `recv' sp` below faces every terminal outcome of this
handshake — including its own Stopped. Deciding here would decide it twice."*

⭑ **So the DESIGN's "one pair whose classification is close to predetermined" is really two different
things, and only one of them is a fence with a missing half.** The thread arm's send and the process
arm's ship are not twins; `:618`'s twin is `service.wat:3198`/`:3336`/`:3469`/`:3572` (send-then-
bounded-wait), and `:548`'s twins are `service.wat:3784` and `test.wat:402` (child announces, parent
waits under a deadline).

---

## 6. The UNKNOWN — 3 sites, one question, and the measurement that would settle it

**`wat/spawn.wat:548` · `wat/service.wat:3784` · `wat/test.wat:402`** — all three are a freshly spawned
child sending a readiness/completion marker **up** to a parent that is, at that instant, sitting in a
`recv-by-deadline`. The three pairings, each read: `spawn.wat:548` → parent at **`spawn.wat:573`**;
`service.wat:3784` → parent at **`spawn.wat:630`**; `test.wat:402` → holder at **`wat/test.wat:335`**.

What defeats the classifier is a **runtime-lifetime** question I did not measure:

- The only window in which these sends can block is **after the parent's deadline fires**, because
  until then the parent is draining. When it fires, the parent runs `assertion-failed!` and unwinds.
- If the parent's `Thread`/`Process` handle is dropped promptly on that unwind, the child's send
  observes `Closed` and every one of its four arms is reachable — **the sends are transitively bounded
  by the parent's deadline, and `ab419aaa3` did not leave them open; it gave them a bound they did not
  have.**
- If the handle can outlive the unwind, the child blocks forever holding a thread or a process —
  **BOUND, and a leak with no reporter.**

I am not going to assert either. The RAII claim is stated in `bracket.wat:14–16` (*"recv' raises …
when the parent's Thread is dropped"*) for a **different** direction of the same wire, and inferring
from it is exactly the shape ruled against by `[[feedback_cite_an_exemplar_do_not_describe_one]]`. Per
the DESIGN's §UNKNOWN test — *"the code does not say, and reading harder will not settle it"* — these
are **UNKNOWN**, not folded into BOUND to tidy the table.

⭑ **The measurement that settles all three at once, and it is cheap** (a scratch-pad probe, ~one hour):
give a child an `:init` that blocks past `WAT_STARTUP_HANDSHAKE_DEADLINE_MS`, let the parent's 573/630
deadline fire, and then ask whether the child's subsequent `send` returns `Closed` or never returns —
redirected to a file and read WHILE IT RUNS (trap-door 2 of `a-send-cannot-say-it-is-blocked`). Either
answer converts three UNKNOWNs into a class. Fixture 2 of
`wat-scripts/scratch-pad/probe-a-full-inbox-blocks-send-forever.wat` is the shape.

---

## 7. What surprised me, stated separately from the classification

1. **PARK = 0** (§3). I went in expecting the recv half's 7-PARK shape to have a send analogue. It has
   none, and the reason is structural.
2. **The kill path is on the BOUND list** (`/stop`, row 11). A stop that can block before its own
   10000 ms budget starts is a worse fact than a slow `call-by-deadline`, and neither the DESIGN nor
   `a-send-cannot-say-it-is-blocked` names it.
3. **`send-keep-serving?` (row 16) shows a bound is already affordable at the REPLY sites it serves.**
   Its `drop?` parameter deliberately drops the reply and returns `true`, on the stated grounds that
   *"T1's deadline turns that into timeout → discard → redial → retry"* (4290–4291). The protocol is
   already built to survive a dropped reply at five of the fifteen REPLY sites.
4. **A duplicated `SendOutcome::Stopped` arm has been green on the floor since 2026-08-04**
   (`bracket.wat:789–790`, §2). The checker admits it.

---

## 8. What I could NOT settle

1. **The three UNKNOWNs** (§6) — needs the lifetime probe, not more reading.
2. **Whether `service.wat:2283` (`Status::Faulted`) is BOUND or REPLY.** I classed it BOUND because
   dropping is cheaper than wedging a *serving* loop, and because `/grant` (3480) and `/revoke` (3583)
   already tolerate its absence through `GaveUp`. But those two arms mean `Faulted` **is** read as a
   reply on one path, so a bound that drops it changes what `/grant` observes. One site, and it does
   not move the 14/15 split enough to matter for the ruling.
3. **What the census's `no=216` licenses.** §0.4.4: it is the absence of a detectable bound form, not a
   measurement that 216 sites are unbounded. Anyone quoting it in the ruling should quote it that way.

---

## 9. Row 11 — what the ruling now needs. ⚠ A RECOMMENDATION, NOT A DECISION.

The counts say the expensive mechanism is worth less than it looked and a cheaper one is worth more.
**14 sites want a bound outright** (§4), and 9 of those 14 are one defect — a bounded wait whose send
is not bounded — so the honest fix reaches far more than 3 sites and the "is it worth 215 arms?"
framing is answered *yes on need*. But **15 REPLY sites** say the variant alone does not finish the
job: at a REPLY site a bound **drops a frame**, and the corpus's own one successful example
(`service.wat:2605–2611`, `try-send` + evict) shows that dropping is only safe when something
**resyncs the wire afterwards** — which is exactly the desync hazard `the-gate-methods-face-an-outcome`
ruled on. So my recommendation is a **three-way split, in this order**: (i) take mechanism **(b)**
— `send-by-deadline` + `SendOutcome::TimedOut` — but **scope its first landing to the 14 BOUND sites**,
where a `TimedOut` needs no new protocol and the S1/S2 species just move their existing deadline to
start before the send instead of after it (`/stop`, `/hibernate`, `/grant`, `/revoke`, `call-by-deadline`
and the two `circuit` proofs are each a `t0`-placement change plus one arm); (ii) leave the 15 REPLY
sites **explicitly out of scope for that landing** and raise them as their own stone — *what a serve
loop does when a client will not read its answer* — whose exemplar already exists and whose answer is
"bound **and** evict", not "bound"; (iii) settle the 3 UNKNOWNs with the one-hour probe in §6 **before**
the codemod, because if they are transitively bounded they need no arm and if they are not, they are
three more S1 sites. ⭑ The fan-out cost is also smaller than *THE RULING OWED* feared and the 5-line
delta in §0.3 does not change it: 203 sites already match exhaustively and the compiler finds them;
the 12 blind are named; and the **only** thing that still argues against (b) landing now is the
`bracket.wat:789` duplicate-arm defect (§2) — **if the checker will accept a duplicated `SendOutcome`
arm, "the compiler finds them" is the Simple answer's load-bearing claim and it is currently
unproven.** I recommend that defect be closed, or the claim re-measured, before the codemod runs.

---

## 10. Verification — report-only, confirmed rather than assumed (EXPECTATIONS rows 7, 10)

**`git status --short`** at the end of the strike, in full:

```
?? docs/excursus/2026/08/001-sns-sqs/a-send-that-should-be-bounded/FINDING-the-classification.md
```

One untracked file — this document. **No `.wat` change, no `.rs` change.** The
`bracket.wat:789` duplicate arm (§2) was found and **left in place**, per the DESIGN's
*"If a site is obviously wrong, report it; do not fix it."*

**Floor**, run through `scripts/floor.sh` (release, captured whole at `.floor/latest/`), Summary line
read from the log — never a piped exit code:

```
Summary [ 555.487s] 5274 tests run: 5274 passed (9 slow), 22 skipped
```

`grep -cE '^ *FAIL|^ *TIMEOUT|^ *SIGSEGV' .floor/latest/raw.log` → **0**. Script exit **0**. **GREEN.**

**Clippy**: `cargo clippy --workspace --all-targets --release` → rc **0**, zero `error`/`warning`
lines. ⚠ Stated honestly: it reported `Finished in 0.09s`, i.e. it served **cached** results because
nothing in `src/` changed. That is the correct confirmation for a report-only stone — the cache being
valid *is* the evidence that no Rust source moved — but it is not a fresh lint pass and should not be
read as one.

---

## ⭑ REGRADED BY THE ORCHESTRATOR ON HIS OWN RUNS, 2026-09-17

**Report-only confirmed**: `git status --short` shows **one untracked file — this document**. No `.wat`,
no `.rs`. ⭑ **No fresh floor was run, deliberately and stated**: `src/` and `wat/` are byte-identical
to `3c3cf7e6f`, whose floor I ran myself (`Summary [ 552.272s] 5274 passed`, no `ARM.txt`), and the
executor's own run on this exact tree agrees (`555.487s`, 5274 passed). A report-only stone that
changes no code cannot move a floor; running one anyway would be theatre, and claiming it as evidence
of *this* stone would be worse.

| claim | the orchestrator's own check |
|---|---|
| ⛔ **the DESIGN's line numbers are stale by +21** | ✅ **CONFIRMED — and it is my error.** `wat/spawn.wat:527` is the comment *"Returns Launched{…}"* and `:597` is *"recv' the child-minted Address'…"*; the sends are at **548** and **618**. I quoted an earlier measurement into a DESIGN handed to an executor instead of re-deriving it — `[[feedback_state_what_the_instrument_can_see_before_quoting_it]]`, *"a wrong count in a DESIGN spends someone else's strike."* It cost nothing only because the executor re-derived. |
| ⭑ **the §0 grep defect reproduced itself on this stone** | ✅ `grep -coE ':wat::kernel::send[^a-zA-Z-]' wat/bracket.wat` → **7**; with `([^a-zA-Z-]\|$)` → **8**. The missed site is `bracket.wat:643`, head at end of line, peer on the next — **and it is on the BOUND list.** |
| ⛔ **`bracket.wat:789–790` is a committed duplicated arm** | ✅ two identical `(:wat::kernel::SendOutcome::Stopped nil)` arms, `git blame` → **`86b30d5dce`, 2026-08-04**, *"a stop is not a death — RecvOutcome/SendOutcome gain Stopped (#73)"*. Green on every floor since. **The checker accepts it.** |
| ⭑ **`/stop` takes `t0` AFTER the send** | ✅ send at `service.wat:3198`, `~stop-t0-sym` bound at `:3203`. The kill path blocks *before* its 10 000 ms budget is armed. |

### ⭐ THE FINDING THAT CHANGES THE RULING

`a-send-cannot-say-it-is-blocked`'s four-questions table gave mechanism **(b)** its *Simple* answer on
the grounds that **the compiler finds the sites** — 28 of 32 live matches are exhaustive, so a new
`SendOutcome::TimedOut` breaks them loudly. **A checker that accepts a duplicated arm weakens exactly
that.** If `(Stopped nil)` twice type-checks, a codemod that duplicates an arm instead of adding
`TimedOut` also type-checks, and the instrument the ruling leans on has a hole in it.

⛔ **So the executor's sequencing advice is accepted: close `bracket.wat:789` — or re-measure the
claim — BEFORE the mechanism lands.** It is currently the only thing undermining (b)'s Simple answer,
and it was found by a report-only stone that was forbidden to fix it.

### The other three findings, accepted as reported

1. **Nine of the fourteen BOUND sites are ONE defect**: a bounded wait whose *send* is not bounded.
   `/stop`, `/hibernate`, `/grant`, `/revoke` all take `t0` after the send, and `owner-recv-loop`'s own
   header opens *"send already happened"* (`service.wat:4330`). **The half-fence, generalised** — and
   `ab419aaa3` (mine) is the stone that drew the first half of it.
2. **`call-by-deadline` violates its own contract by name** (`service.wat:4599`): `race-reply` is
   invoked *inside* the `Sent`/`Closed`/`Lost` arms, so `CallOutcome::DeadlineFired` **cannot fire
   while the send blocks**. A deadline that does not cover the whole round-trip.
3. ⭑ **The corpus already contains the fix, applied once** — `service.wat:2605–2611`, the only live
   `try-send`, whose comment is this finding written three weeks early. The shape is **bound + evict**,
   and the eviction is what stops a dropped frame from desyncing the wire.

### PARK is ZERO, and that is a structural result

The recv twin had **7 PARK of 23**. This has **0 of 32**. *A recv can be an idle worker; a send is
always handing something to someone.* So there is no class of send site where blocking is correct by
design — which is a stronger statement than the classification was asked for, and it simplifies the
ruling: the question is never *"should this block?"* but only *"what does it do when it cannot
deliver?"*

### Controls: two of three DISAGREED, which is why the table is trustworthy

`spawn.wat:548` predicted BOUND → read **UNKNOWN**; `bracket.wat:44` predicted PARK → read **REPLY**;
`service.wat:2588` predicted REPLY → **REPLY** ✅. ★ Control 2's lesson is the reusable one: **the class
of a `recv` does not transfer to the `send` five lines below it.** A worker loop is PARK on the way in
and REPLY on the way out — `collect-loop` exits only on `collected = m`, so a dropped result hangs the
**coordinator**, not the worker.

### And it split my own `spawn.wat` verdict rather than accepting the framing

The DESIGN asserted both spawn sites were "half a fence". Read: `:618` (ProcessOpts) **yes** — the
parent ships, then waits at `:630` under the deadline, so a child not yet draining blocks the parent
*before* its deadline is armed. `:548` (ThreadOpts) **no** — it is the **child announcing readiness
upward**, and the parent's bounded recv at `:573` is the receiver *of that very frame*. Sender and
receiver of one message, not two wires. Refusing a framing handed down in the DESIGN is the behaviour
this campaign wants.
