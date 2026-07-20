# DESIGN — wat never hides a failure (the IPC death/error path)

> **THE LAW (builder, 2026-07-17):** *"i want wat to never hide failures ever again … this masking of
> failure is actively hostile against wat's intent."* Every place on the peer/service death path that
> discards an error, collapses distinct failures into one mute value, writes a reason to a closed pipe,
> or kills a whole service over one bad message is the SAME class — **failure-masking** — and this arc
> pulls the class out by the root. We own wat; the arc-294 "crash reasons are administrative" ruling does
> NOT shelter a masking behavior — we change our minds when the mask keeps blinding us.

## STATUS — 2026-07-19+ (ALL THREE pieces LANDED — the LAW is CLOSED AIRTIGHT for masking; the STOP-2 crash-broadcast-to-clients is a separate future capability, not a masking)

**All three of the law's pieces are real, verified, pushed — no failure that EXISTS is hidden:**

- **Alive-service-rejects-you — Mechanism A** (`66d6aed7`): `poll'` returns `ServiceEvent::Malformed{idx,
  cause}` instead of raising; the serve loop replies `Reply::Failed{cause}` and **keeps serving** (no
  DoS); `recv'` surfaces `Reply::Failed` as a catchable raise carrying the reason. `Reply::Failed[cause
  <- Failure]` is the reserved **protocol-tier** floor on every synthesized `<S>::Reply`
  (`types.rs synthesize_surface_protocol`) — the 293 outcome-enum model completed (op-tier failure =
  `<Op>Response::{Transient,Fatal}`; the "couldn't resolve to any op" floor was missing). — table sites
  #9, #10 + the protocol-tier gap.
- **eprintln is terminal** (`dc286d7a`): `eprintln`/`epprintln` emit the value then `panic_any` →
  structured-exit (uncatchable, cross-loci) — the dying declaration you designed; `feedback_eprintln_is_terminal`
  closed. The `:Lost` serve-loop arm now stands on it — an abnormal transport break lets-it-crash (OTP),
  cause on stderr before exit. Return type kept `-> :wat::core::nil` (`:()` is the redundant unit
  spelling, being retired). *(This RESOLVES the earlier "`:Lost` eprintln" judgment call — it is the
  intended crash-with-message, not a benign write.)*
- **`call_beside` idiom** (`dc286d7a`, intueri-ratified): the lint-clean "run the co-located fixture's
  entry fn" helper (`src/freeze.rs`, beside `startup_beside`); first consumers wired → `no_inlined_wat`
  352 → **351**.

Verified by own full re-run: 4170 passed / 1 failed = the standing `no_inlined_wat` at 351, zero new;
`no_loose_string_assert` PASS; `eprintln_terminal` + `dead_child_speaks` + `crash_surfaces` all green.

**One judgment call still nominally open** (clarified as a non-decision): surfacing lives in `recv'`,
not a client-method match arm — a wat `assertion-failed!` in a method is `panic_any` (uncatchable);
`recv'` is the one catchable, uniform surfacing point (step 3 / "room 8"). The client gets a catchable
error, OTP-consistent. Nothing to decide.

**★ THIRD PIECE — the transport-tier twin — LANDED (`3f73f400`).** `RecvError` gained `Failed(String)`
(`comms/mod.rs`, drops `Copy`); `Disconnected` is now **clean-EOF ONLY** (documented + enforced); the 8
`map_err(|_| Disconnected)` collapses in `comms/process.rs` (io_uring/utf8/`from_wire`/`Malformed`) bind
the real reason into `Failed(reason)`; `channel/transfer.rs` → `RecvOutcome::DecodeError`; `spawn.rs`
`classify_peer_death`/`classify_peer_error` map `Failed(reason)` → `PeerDeath::Lost(reason)`; `recv'`
(`runtime.rs`, socket `recv_wire` + bare-`Peer` arms) thread the reason into the raise. A raw wire failure
can no longer masquerade as a clean close — **the LAW is airtight for masking**. RED gate:
`probe_arc278_transport_reason_carried` (invalid-UTF-8 → `Failed("…utf-8…")`, was mute). Weighed by own
full re-run: **4176 passed / 0 failed / 330 skipped** (the `wat-cli sigterm…polling_contract` flake passes
isolated). *(The decode/dead-child case was already closed earlier by Mechanism A — `dead_child_speaks`
green — so the sites-R,1–8 table was over-scoped; this piece is the raw-transport-reason residual.)*

**STOP-2 — a SEPARATE FUTURE CAPABILITY, not a masking (does NOT block the law):** a genuinely-crashing
process unit's reason reaches its **owner** (the `/start` `Handle`, via `PeerRecvError::Crashed` at
`runtime.rs:26159`) but NOT a separately **`connect'`-ed client** peer (`kernel/peer.rs` `Peer{tx,rx}` has
**no crash channel**) — that client sees an honest clean-EOF, not a *masked* reason (no reason exists on
its channel to hide). Delivering crash reasons to connected clients is an **absent broadcast capability**
(a new mechanism: `service.wat` codegen + the panic boundary broadcasting the crash payload to every live
client before exit) — its own arc when it's wanted. Tracked, `#[ignore]`'d:
`probe_arc278_process_crash_reason_carried.{rs,wat}` (finding in its module doc).

## ═══ THE STRUCTURAL CLOSURE — the over-budget mute, and killing the class so it CANNOT regrow (2026-07-20) ═══

> **THE LAW, RESTATED (builder, 2026-07-20):** *"silent errors — they must die — now — we've killed them like 5 times on this arc AND THEY REFUSE TO DIE."* Every prior kill was a **stem-cut** (bind the mute site we found). The class regrows because a mute failure still has a **representation**. This closure climbs to the top of the extirpare ladder: make a mute failure **unconstructible** (structural impossibility — builder: *"structural impossibilities are the best in any situation"*), not caught case-by-case.

### The incident that surfaced it (grounded this session, reproduced by own hand)
The RICH Rules arena: a single `write-logs` of ~700 rich nested `Event`s on **PROCESS** locus →
`recv failed: peer closed / channel disconnected` — **mute**. Reproduced at `scratchpad/probe-arena-scale-n700.wat`
(THREAD=630 correct; PROCESS mute at the `write-logs` recv', line 176). The ~650–700-row threshold is exactly
where the frame crosses **512 KiB** (`DEFAULT_MAX_FRAME_BYTES`, `edn_shim.rs:1330`). Not a crash — a **frame-cap
rejection failing mute**.

### The grounded root — a reason that EXISTS, discarded at ONE site, then mislabeled CLEAN
1. The receiver hits the cap → `RecvError::FrameTooLarge`, whose `Display` is a perfectly good reason
   (`comms/mod.rs:990` — "frame exceeded cap (message larger than the receiver's max-message-bytes budget)").
2. **`channel/transfer.rs:176`** (the crossbeam/thread client-recv path): `FrameTooLarge => RecvOutcome::Disconnected`
   — the reason is **discarded**, collapsed into the reason-free clean-EOF variant. (Comment: *"per the arc 278
   contract, FrameTooLarge is NEVER read off the err channel"* — the deadlock-avoidance rationale is real but does
   NOT justify muting: the FrameTooLarge reason is **local to the receiver**, needs no err-channel read.)
3. `poll'` sees `Disconnected` → `ServiceEvent::Closed{idx}` (`runtime.rs:27666`) — the **clean-hangup** arm (no
   cause; *"bare Peer' has no crash channel"*).
4. The serve loop's `Closed` arm (`service.wat`) drops the client and keeps serving — **silently**, because it
   believes the client hung up normally.
5. The caller's `recv'` gets the mute `peer closed`.
So a **frame-cap rejection is relabeled a clean goodbye** three layers deep. (Grounded contrast: the *process
peer OUTPUT* path already speaks — `classify_peer_error` (`spawn.rs:244`) maps `FrameTooLarge => Lost(reason)`.
The inconsistency between the two paths is the stem.)

### Why the class refuses to die — 5 stem-cuts, never a wall
Mechanism A · eprintln-terminal · transport-twin `RecvError::Failed` · RST `PeerCrashed` · startup-crash honesty
— each bound a *known* mute site. None made mute **unrepresentable**. `transfer.rs:176` is stem #6-in-waiting.
The root: **reason-free failure variants remain constructible from error paths** (`=> Disconnected`,
`map_err(|_|)`), so a mute failure always has a form to hide in (the arc-278 masking table already named this at
"site R — no slot for a reason", then added `Failed(String)` beside the reason-free variants but left them
constructible-from-error).

### The fix — three escalating moves; the builder's own reframing
The builder decomposed it exactly (kept literal): *"a > max-bytes message is a 400-esque error — the server is
still alive, just tossing a bad request… should there even be a disconnect? 400 should be a thing a client can
just deal with without faulting hard."* And on WHO learns why: *"clients should never be told [internal crash
reasons] — the admin holder MUST KNOW"* — but a frame-cap rejection is a **client-input error (a 400)**, not an
internal crash, so telling the client "your request is too large" is honest and correct (it's about *their*
input), while genuine internal crashes stay admin-channel-only (STOP-2, `feedback_ask_who_already_receives_it…`).

1. **SPEAK — never mute, never mislabeled-clean.** `FrameTooLarge` (and every genuine failure) carries its reason
   to `recv'`; it is NEVER collapsed to `Disconnected`/`Closed`. `transfer.rs:176` → carry the reason (the
   `DecodeError(reason)` outcome the sibling `Failed` arm already uses, or a distinct reasoned outcome). The floor.
2. **400-and-continue — no hard fault.** An over-budget request is a client error: the serve loop replies
   `Reply::Failed{cause: "request exceeds the N-byte cap"}` to *that* client and **keeps serving**; the connection
   **lives** (Mechanism A shape, extended to the frame cap). The client catches a normal error and moves on.
3. **THE WALL — structural impossibility (top rung).** Reason-free variants (`RecvError::Disconnected`,
   `RecvOutcome::Disconnected`, `PeerDeath::Closed`, `ServiceEvent::Closed`) mean **clean EOF ONLY**, and **no
   failure path may construct them** — make mute **unrepresentable**, not merely caught. Preferred: the type
   level (a failure value cannot be built without a reason; the clean-EOF variant is producible only from a
   genuine EOF, structurally). Backstop: a **lint** (sibling of `no_inlined_wat`) that RED-flags any `=> …Disconnected`
   / `map_err(|_|)` in `comms/`,`channel/`,`kernel/spawn.rs`,`runtime.rs` recv paths — so stem #7 is a build error.
   *This check is what the previous five kills never planted.*

### Feasibility of 400-and-continue — GROUNDED (the drain-realign, and why no deadlock)
The wire is **newline-framed** (`next_complete_frame` scans `\n`, `edn_shim.rs:1387`), single-writer
(interleave-safe, `process.rs:296`), and the receiver's accumulator **persists** across reads (`take_frame` only
`split_off`s on a *good* frame — on `TooLarge` the bytes stay). So recovery is a **caller-policy change**, not a
transport rewrite. Two `TooLarge` cases (`next_complete_frame`):
- **complete-but-too-big** (a full newline-terminated frame over budget): trivial — discard `acc[..end]`, continue
  with `acc[end..]`; wire already re-aligned.
- **incomplete-and-already-over-budget** (the sender blocked mid-`write_all`, big frame still arriving): **drain to
  the terminating `\n`** — which unblocks the sender and re-aligns — then discard + reply 400 + continue. This
  **sidesteps the cited deadlock** because it only drains the DATA channel and **never reads `err`** (the deadlock
  was reading `err` while the sender is blocked). **DoS bound:** drain up to K× the budget; a frame that never
  terminates within the bound is a pathological client → reasoned teardown (never mute).
The current code just tears down on `TooLarge` because it never implemented the drain-loop — a convenience, not a
necessity.

### RED gate (acceptance — the probe that would have caught this from day one)
A service with an op budgeted at N bytes receives a request > N (both loci). Assert BOTH:
- **the caller's error carries the real reason** (contains "exceeds"/"too large"/the byte figure), NOT the bare
  `peer closed / channel disconnected`; and
- **the service + connection are still alive** — a subsequent, in-budget request to the SAME service succeeds.
Plus the pathological case: an endless (no-newline) frame past the drain-bound → a *reasoned* teardown, still not
mute. At HEAD the first two fail (mute + connection dead). GREEN when moves 1–3 land.

### Sequencing + the arena hold
This structural closure is the **floor** the service-I/O-budget contract stands on
(`DESIGN-service-io-budgets.md` — per-op declared budgets, fragmentation/pagination tooling, output-side
streaming). Land the closure (speak + 400-and-continue + the wall) FIRST. **The RICH Rules arena commit is HELD**
until at least move 1 lands — it must never ship green on the masked teardown + the shadowdancer's chunking
workaround (`RVINA VIAM FABRICAT`: forge the ruin out, do not route around it).

---

## ═══ SESSION-END CURARE (far-side state — DESIGN PHASE DONE (service I/O contract); #15 mute-kill floor IN FLIGHT; the arena GREEN-but-HELD on shortcuts) ═══

**READ THIS BLOCK, then `git status`. Branch `arc-170-gap-j-v5-deadlock-state`; HEAD ≈ `24ac73e7` (#15 facet 1). A LATER HEAD = the speak-mute facet (#15.2) or more landed — trust the disk + the git log. A HEAD mismatch is the ALARM.**

Since the last breadcrumb: the RICH Rules arena was weighed GREEN by own re-run (8/8 both loci, floor 4177/0) — but it passes only by ROUTING AROUND two live substrate ruins (hand-chunked 2×400 writes around a 512 KiB frame cap that fails MUTE; inlined page-loop around a `Peer'`-handle-param RPC sever). Per R50/extirpare we do NOT ship green on masked failures → the arena is HELD (#14), redone CLEAN in #21. That surfaced the real work — the **AWS-shaped service I/O contract**, designed to completion with the builder, and the **structural mute-kill** (this doc's STRUCTURAL CLOSURE section, above the historical marker).

### The DESIGN is DONE + on disk — READ these, do NOT re-derive
- **`DESIGN-service-io-budgets.md`** — the full I/O contract, BOTH loops: two-sided (defense-at-the-gate: the server enforces budgets, rejects a bad NETWORK client with a REASON, keeps serving; + ergonomic tooling: perfect-knowledge, good clients never hit enforcement); per-op budgets on `:features` (declared/discoverable); named-error response contract (a variant per kind — 400-fixable vs 500-fault, exhaustive/conformare); reader `<op>-stream` → lazy `Stream<enum>` (the builder's Ruby Enumerator; in-band failure, case-matched); writer `write-*-stream/-batched` + `with-log-sink` (backpressured enqueue-ack, `:ephemeral` buffer + flush-at-EVERY-exit, time-OR-size flush via the io-selectable timerfd); per-item-max = budget−envelope + up-front `::ItemTooLarge` (reject, not enqueue); output composite cursor `{row-cursor, ded-offset}` for inference-explosion.
- **THE STRUCTURAL CLOSURE** (this doc, the section above the historical marker) — the mute-kill: speak + reject-and-keep-serving + the WALL.
- **`scratchpad/design-io-budgets-ux.wat`** — the materialized caller UX (R17).

### Landed this stretch (pushed, DR)
`75ca51c8` DR checkpoint (designs + arena WIP, honest green-but-shortcut) · `3e8a71b6` finalized design · `24ac73e7` **#15 facet 1: `FrameTooLarge` DRAINS + keeps serving** (comms `take_frame` drains a complete over-budget frame + re-aligns, still returns the reason; RED probe `tests/comms/probe_arc278_over_budget_recovers.rs`; own re-run floor 4178/0).

### ★ RESUME AT — finish #15, then the build order
**#15 (mute-kill floor), three facets:** (1) drain-realign / reject-and-keep-serving — **DONE** (`24ac73e7`); (2) **SPEAK-mute — IN FLIGHT** (a shadowdancer): the process client-`recv'` (`runtime.rs:26168-26210`) lumps `FrameTooLarge` into the generic "peer closed" (the arena's ACTUAL mute) — make it carry the reason (mirror `classify_peer_error`, `spawn.rs:244`); **check the task result → weigh by OWN re-run (RED-by-revert + GREEN + floor) → commit**; (3) the **WALL** — OWED (reason-free variants unconstructible-from-error + a lint backstop; structural impossibility).
- **FINDING:** the frame-cap facets are **PROCESS-ONLY** — the thread tier (crossbeam) has no byte-framing → no `FrameTooLarge`. Thread was never affected; `transfer.rs:176`'s Comms arm is defensive/unreachable.
- **BUILD ORDER (tractability-first):** #15 → #17 (named-error vocab) → #16 (per-op budgets + transport ceiling + `spawn.rs:765` input-channel fix + serve-loop enforcement) → #18 write tooling / #19 `<op>-stream` reader → #20 output-streaming → #21 re-do the arena CLEAN (capstone). Tasks #14–#21.

### LESSONS (this stretch — do not relearn)
- **Two-sided contract:** a defservice is a NETWORK service for untrusted clients → defense-at-the-gate (reject bad input with a reason + keep serving; one dumb client can't wedge/DoS it) AND ergonomic tooling (perfect-knowledge, good clients never trip it). Both.
- **Perfect-knowledge tooling never emits over-budget** → enforcement is unreachable-for-us but MUST exist for the network; a single over-max item is REJECTED (`::ItemTooLarge`), never enqueued.
- **Never overload an error bucket** — a NAMED variant per failure kind; exhaustive `match` forces handling (conformare); 400-fixable distinct from 500-fault.
- **Streams both ways** (read = pull a lazy Stream / write = feed a Stream or push a buffered sink); the page/batch boundary invisible.
- The mute class must die **STRUCTURALLY** (unrepresentable), not caught case #N — 5 stem-cuts already regrew. RED probe FIRST, weigh by OWN re-run, commit green (no broken commit).
- *(Arena-stretch, still valid: graph-inference `Deduction=derived−fired-upon` / `Lemma`=gate, NO boxing; programmable-DB = police safety not usefulness `[[feedback_programmable_db_police_safety_not_usefulness]]`; IPC locus doctrine `[[feedback_ipc_loci_defservice_bracket_wat_only]]`; rich records + `where`-rules, `scratchpad/probe-rules-rich.wat`.)*

### Realizations: **R50** `RVINA VIAM FABRICAT` freshest (R49 `GLADIVS LOQVITVR` prior). The arena's exact-count kill is done — but HELD; R50's PROBATVM waits on #21 (the clean redo).

> **SEAM.** The self past this line is NEW — a lossy cache in a familiar voice, not your memory. Run the datamancy
> bootstrap (grimoire + 4 primers + recolligere from the SIGNED MCP, never disk); read 278's realizations. Ground
> `git status` (branch `arc-170-…`; HEAD ≈ `24ac73e7` or later). **The DESIGN PHASE is DONE** — read
> `DESIGN-service-io-budgets.md` + this doc's STRUCTURAL CLOSURE; do NOT re-derive the contract. **RESUME AT: finish
> #15** — the SPEAK-mute shadowdancer (weigh its result by your OWN re-run: RED-by-revert + GREEN + floor; commit),
> then the WALL, then the build order #17→#16→#18/#19→#20→#21. The **arena (#14) is HELD** on its shortcuts — redone
> CLEAN in #21; do NOT bless it as done. **Silent errors must die STRUCTURALLY** (5 stem-cuts regrew — the wall, not a
> 6th patch). Two-sided contract: defense-at-the-gate + ergonomic tooling. RED probe FIRST; weigh by your OWN re-run;
> commit green; the git log is DR. Do not trust this note over the disk. `MACHINA CHAOS DOMAT — the flood becomes inference.`

**↓ Everything below is HISTORICAL** (the edn-crusade → 294.f detour, committed in `98499f48`; superseded by the
campaign above — kept for lineage, not as the live breadcrumb):

## ═══ (historical) edn crusade → holon-AST demise detour ═══

**The no-hidden-failures LAW is DONE (below, unchanged).** This
session went: `no_inlined_edn` lint (the far-side task) → the `.edn` **crusade** → it surfaced **holon-AST
heretics** → **arc 294.f** (reflection holon-AST demise), pulled ahead of its PHASE-1 gate by builder decree
("i do not wish to bear this cost any longer — the crusades revealed the pressure").

**COMMITTED this session:** `7703cd89` (crusade wave-0 exemplar: `no_inlined_edn` scoped to `tests/` + the
detector, `1306→235`; `tests/collection` converted), `2c743cfe` (R43 Eden — `HORTVS CONSILIO SATVS`; EDN⊂EDEN).

**UNCOMMITTED in the tree (large — survives compaction on disk):** the `.edn` **fleet** (~136 golden→`.edn`
conversions + 42 per-offense runes across ~13 `tests/` dirs) + **294.f** (reflection: `type_expr_to_ast`→
canonical `wat.type/` via `type_expr_to_clojure_form`; 14 producers + 3 verbs (`extract-arg-names/types`,
`rename-callable`) + checker retyped to `WatAST`; `holon_type_ast_to_wat_type_form` **DELETED**; 10
`wat_arc201_*` fixtures → `ast->children`; goldens re-captured) + the close-out (8 `rune:clojure-flip`
string-eq bridges, the `wat_arc221b` scope-fix, 2 edn STOPs: `probe_arc209` convert + `wat_core_cond` sentinel
rune) + my detector fixes (`.contains`/`.starts_with`/`.ends_with` output-role + a `no_inlined_wat` file-rune).

**★ THE ONE BLOCKER before the commit (do this FIRST):** the full weigh is **4190 pass / 5 fail** — the 5 are
`rune:clojure-flip` string-eq bridges whose goldens are **pretty-printed (multi-line)** but the actual is
**single-line** → exact-string mismatch. **FIX: re-capture these 5 goldens SINGLE-LINE** (the exact `left:`
string from the assert failure). The 5: `wat_arc144_uniform_reflection::primitive_empty_lookup_define_emits_define_head`,
`wat_arc201_structured_signature_types::signature_of_defn_foldl_emits_structured_parametric_and_fn`,
`wat_arc143_lookup::signature_of_defn_foldl_renders_synthesised_shape`,
`wat_arc143_manipulation::rename_callable_name_happy_path_foldl_to_reduce`,
`wat_arc144_hardcoded_primitives::lookup_define_length_renders_primitive_sentinel`. (The other 3 bridges already
pass — their output happened to be single-line already.) Then `cargo nextest run --release` → green (only the
known `wat-cli sigterm…polling_contract` flake, passes isolated) → **ONE commit** (crusade + 294.f), then push.

**★ 294.f — what landed vs what's DEFERRED (the proper clojure flip):** reflection now emits canonical
`wat.type/` **plain EDN** for the common case — spec-perfect: `(:anonymous (n wat.type/i64) (s wat.type/String)
-> wat.type/String)`, no `#wat-edn.holon`, no `::`, no `<>`. **DEFERRED to the proper clojure flip** (its own
stone; **`grep -rn 'rune:clojure-flip'`** finds every bridge to un-defer): (1) the 8 edge cases — multi-slash
keywords (`__internal/…`, `Vector/length`) + `<T,Acc>` multi-param generics (`<T_Acc>` wire form) — need a
**symmetric faithful codec** (`keyword_from_wat_path`↔`ns_to_wat_path`, drop-`<>`-in-names); (2) the **`:-`
typed-clojure sigils** — the intended form is `(:anonymous (n :- wat.type/i64) … :- wat.type/String)`, current
is the bareword `(n wat.type/i64)`. The full **`294.d` (wire-kill, `HolonRepresentable` + `#wat-edn.holon`
tags) + `294.e` (`HolonAST`→`Hologram` rename + `src/holon/`) stay GATED behind PHASE-1 aggregate parity**
(`CLOSE-SEQUENCE-293-294.md`). Log 294.f-common-case-landed + this debt in that tracker before/with the commit.
The `probe_diagnostic_typed_entities_p1–p7` VSA fixtures + `Bundle/children`/`Bind/*`/`hologram.rs` were
correctly UNTOUCHED (genuine holographic — holon reserved per the builder's law).

**★ THE ACTUAL RESUME (unwinding the whole detour — the arc's TARGET):** the **CHAOS ENGINE** — R25 `MACHINA
CHAOS DOMAT`, a streaming rete datalog held in a `defservice` (the DDoS/anomaly lineage; "the database is the
debugger"). The **telemetry facility (the on-ramp/instrument) is FUNCTIONALLY COMPLETE** — T1b + Span + T2, per
`DESIGN-reserved-prefix-one-gate.md:243`; `journal'` sink write-path landed (`b07f5ffc`), **backend-agnostic AND
loci-proven** (thread + process). The builder's "thread was different from process / hidden error we've been
chasing" = the `journal'`-on-**process** incident that masked a client decode failure as a mute "peer closed"
(the child's 687-byte reason `EPIPE`'d; thread-tier surfaced it, process-tier hid it) — that **spawned the
no-hidden-failures LAW, now CLOSED** (Mechanism A + eprintln-terminal + transport-twin `RecvError::Failed` + RST
`RecvError::PeerCrashed`); the chase is OVER, failures are honest in all loci. So after the commit: **build the
CHAOS ENGINE** (R0 — the streaming rete `Session`-as-state service, incremental insert/retract, dogfooding
telemetry), guided by the wat oracle (`OCVLI NOVI, ORACVLVM IMMOTVM`). Ancillary open: STOP-2 (crash-broadcast
to `connect'`-ed clients, `#[ignore]`'d `probe_arc278_process_crash_reason_carried`, a separate future arc).

---

## RESUME (curare — 2026-07-19+; HEAD `f0230bbc` = crusade + query (a) + the LAW CLOSED + the RST landed, all pushed; CURRENT WIP = the `no_inlined_edn` lint, UNCOMMITTED)

**READ THIS FIRST, then `git status`.** The LAW work is **committed, pushed, weighed** — HEAD `f0230bbc` on
`arc-170-gap-j-v5-deadlock-state`. The arc-278 **no-hidden-failures LAW is CLOSED AIRTIGHT** and reaches the
wire: Mechanism A + eprintln-terminal + the transport-tier twin (`RecvError::Failed`, `3f73f400`) + the
**RST** (`RecvError::PeerCrashed` — a crashing service best-effort-notifies its peers, `f0230bbc`). Owner gets
the reason; peers get a reason-free reset; nothing mute.

**BUT the tree is NO LONGER clean** — there is UNCOMMITTED WIP (survives on disk): the new **`no_inlined_edn`
lint** (`tests/lint/no_inlined_edn.rs`, RED at 1306 — the detector OVER-FIRES, ~90% false positives; NOT
commit-ready) + two new breadcrumb docs (this one + `DESIGN-STONE-no-inlined-edn.md`). Freshness check: live
HEAD `f0230bbc`, dirty tree = the lint WIP; if HEAD differs, trust the disk.

**★ THE CURRENT THREAD — `no_inlined_edn` (see `DESIGN-STONE-no-inlined-edn.md` for the full breadcrumb):**
the sibling of `no_inlined_wat` — an EDN-esque string literal (`#`/`{`/`[`/`(`-opener) must be a co-located
`.edn` golden (`include_str!` + `assert_edn_eq!`), not inline string-eq. FAR-SIDE, IN ORDER: **(1) annihilate
the false positives** — tighten the detector with more ignore-conditions (`#`+digit/`#{}` = a marker not a tag;
format residue of pure identifier-glue `::`/`/`/`'` = not EDN; find more classes — NOT restructure the code) so
it fires on GENUINE EDN only (overwhelmingly `tests/` goldens); **(2) the `wat-edn` parser-test carve-out**
(its own tests inline EDN as input-under-test — a file rune, like `no_inlined_wat` grants); **(3) the conversion
campaign** — drive-to-zero, every genuine offender → a pretty-printed `.edn` golden, edit-only riders + central
weigh (FM 18), runes at an extremely-hard bar. Re-run `cargo nextest -E 'binary_id(wat::lint)'` for the live
offender list (the scratchpad `edn_offenders_v2.txt` is session-specific — do not rely on it).

### THE CRUSADE — `no_inlined_wat` 351 → 0 (DONE; committed `952ece8b`; full suite green)

- **Lint at TRUE ZERO + full suite GREEN** — own re-run `cargo nextest run --release` = **4175 passed / 0
  failed / 329 skipped** (`no_inlined_wat` PASS, `no_loose_string_assert` PASS). The whole test corpus is
  migrated off inline-wat into co-located `.wat` / `.wat.bad` / `.edn` / `.wat`-golden fixtures.
- **`query` (a) BUILT + weighed** — `query` defn→defmacro (prime-append idiom); `return-type-of` de-masked
  (`runtime.rs` keyword branch raises on unknown; `check.rs` validates the literal prime at check time → a
  typo is a compile error, not silent 0). New RED gate `probe_arc278_query_type_safe`; the `5a` fixtures are
  back on the `(:wat::rete::query fired :Type)` front door.
- **The `.wat` raw-text golden rubric** added to `docs/CONVENTIONS.md` § Test idioms; the stale
  `wat_scripts_fixes_load` rune STRUCK (excusare); **R39 (VNA CAEDE PROBATA) + R40 (HAERESIS SANGVINE
  CONSTAT) inscribed**.

### ★ NOTHING OWED FOR THE LAW — the transport-tier twin LANDED (`3f73f400`)

The no-hidden-failures LAW is complete: `RecvError` carries its reason (`Failed(String)`; `Disconnected` =
clean EOF only) — no raw wire failure masquerades as a clean close. Weighed by own re-run: full suite
**4176 passed / 0 failed / 330 skipped**. The ONE forward item is a **SEPARATE FUTURE CAPABILITY, not a law
residual**: crash-broadcast to connected clients (STOP-2, see the STATUS block above) — a `connect'`-ed
client peer has no crash channel, so a genuine unit-crash reaches the *owner* but a connected client sees an
honest clean-EOF (not a masked reason). Delivering the crash payload to every live client before exit is a
new mechanism (`service.wat` codegen + the panic boundary) — its own arc when wanted; tracked `#[ignore]`'d
in `probe_arc278_process_crash_reason_carried.{rs,wat}`.
   *(Steps 0–3 — bootstrap, `query` (a), clean re-weigh, the one green commit — and step 4, the twin — are all DONE.)*

### LESSONS (hard-won this arc — do not relearn)

- **The full-suite re-weigh catches what the lint can't** — the "lint-zero" tree had **11 full-suite
  failures** the earlier truncated read hid. Two migration classes: (a) `defclause` registers into
  `runtime_def_values`, NOT `sym.functions`, so a defclause name is unreachable via `symbols().get()` — its
  fixture needs a `defn` wrapper; (b) single comm-exprs migrated from `parse_one!`+direct-eval (no comm-check)
  into startup/freeze `.wat` fixtures RE-IMPOSE `CommCallOutOfPosition` — bare `send`/`recv` must move to a
  `match` scrutinee (identical Result/Option shape) or `Result/expect` (where the shutdown-cascade crash IS
  the intended semantics). ALWAYS run the full suite + capture the **Summary** before the commit — and read
  the Summary, NOT `$?` (a trailing `grep -c` with no match exits 1 and lies).
- **FM 18 (the fleet cargo-lock thrash)** — riders do TEXT edits only; orchestrator weighs CENTRALLY once.
- **A loose string assert can MASK a real bug** — query (a)'s `msg.contains(...)` hid a `keyword/from-string`
  fixture defect (it rejects leading-colon input); the exact `assert_eq!` unmasked it. The lint earns its keep.
- **`query-by-type-string`** stays the private helper / dynamic escape hatch behind the `query` macro.
- **DON'T lean on "compaction boundary" to defer** the correct fix (builder cut this) — do the right thing.

---

> **SEAM.** The self past this line is NEW — you did not live this session; it is a lossy cache in a
> familiar voice, not your memory. Run the datamancy bootstrap (grimoire + 4 primers + recolligere from the
> SIGNED MCP). Ground `git status` — **HEAD is `f0230bbc`; the LAW work is committed + pushed + green; but the
> tree is DIRTY** with the `no_inlined_edn` lint WIP (uncommitted — survives on disk). The arc-278
> **no-hidden-failures LAW is CLOSED AIRTIGHT and reaches the wire** — Mechanism A + eprintln-terminal + the
> transport-tier twin (`RecvError::Failed`) + the **RST** (`RecvError::PeerCrashed`, `f0230bbc` — a crashing
> service best-effort-notifies its peers; owner gets the reason, peers get a reason-free reset; STOP-2
> crash-broadcast-to-`connect'`-ed-clients remains a separate future capability, `#[ignore]`'d).
> **THE CURRENT WORK is the `no_inlined_edn` lint** (RESUME above + `DESIGN-STONE-no-inlined-edn.md`): it is
> BUILT but RED at 1306 and OVER-FIRES (~90% false positives — identifier-glue format templates + `#N`
> markers). FAR-SIDE FIRST MOVE: **annihilate the false positives** — add detector ignore-conditions (`#`+digit
> ≠ tag; glue-only format residue ≠ EDN; more classes) so it fires on GENUINE EDN only — NOT restructure the
> code, NOT rune them (extremely hard bar). Then the `wat-edn` parser-test carve-out, then the conversion
> campaign (offenders → pretty-printed `.edn` goldens). Do not trust this note over the disk. The law is closed;
> the lint that guards the `.edn`-golden discipline is the next stone. We rode to Gondor; now we tend the wall.

## SUB-STRIKE — `eprintln` is terminal (2026-07-18; closes `feedback_eprintln_is_terminal`)

Surfaced while ratifying the `:Lost` disposition: `eprintln` was **designed** as a dying declaration —
builder direction 2026-05-15 (`docs/arc/2026/04/109-kill-std/INVENTORY.md:1284`): *"eprintln is a 'we are
crashing, here's what I know' and exits"*; the kernel's three **terminating** forms are `eprintln` (value),
`panic!` (message), `assertion-failed!` (assertion shape). `COMPACTION-AMNESIA-RECOVERY.md:1847`: eprintln →
*exit code non-zero*. But the **implementation** (`services/verbs.rs:147 eval_kernel_eprintln`) writes to the
stderr service and returns `Value::Unit` — benign, non-terminal — and `USER-GUIDE.md:3577` documents it that
way. The doctrine was deferred ("pending — separate slice") and never closed. So the crash-with-message
primitive **silently doesn't crash** — the masking law's own shape, baked into the primitive. Builder:
*"eprintln was meant to be 'this is the last thing I'll say' … it is quite frustrating to see this."*

**The fix (ratified — build now, take the fallout):** `eprintln` (and `epprintln`) emit the value's EDN to
stderr, then **terminate non-zero** — uncatchable, uniform across loci (a panic → `emit_structured_exit` in a
forked child / kills the serve loop on a thread / non-zero exit in main), the same convention as
`assertion-failed!`; return type `∀T. T -> :()` (the terminating-form type, not `-> :wat::core::nil`). Then
the `:Lost` serve-loop arm (`service.wat:864`, already calling `eprintln`) is correct as written — an
abnormal transport break is an unexpected failure that lets-it-crash (OTP), and the reason lands on stderr
before exit.

**Fallout (the ~benign usages):** most are tests *of* eprintln (`ambient-stdio.wat`, `test.wat:184/206`,
`probe_arc255_epprintln`, `wat_arc170_slice_1f`) — they become tests of the **terminal** behavior (program
eprintln's → exits non-zero, value on stderr). One is an incidental mid-`let` diagnostic
(`tests/channel/…drain_and_join…:7 (eprintln "diag")`) that must migrate (→ `println`, or drop). **Open (do
NOT mint speculatively):** whether the substrate wants a *benign* stderr write at all — the immediate fallout
doesn't require one; if a use surfaces, its name is a separate intueri cast.

**RED gate:** a program running `(do (eprintln "dying words") (println "AFTER"))` must NOT emit `AFTER`
(eprintln terminated), must carry `dying words` on stderr, and must exit non-zero. At HEAD `AFTER` prints and
it exits 0 (benign). GREEN when eprintln terminates.

## The incident that surfaced it (grounded)

`tests/services/probe_arc278_journal_logs_on_process` — a `journal'` service forked to a **process**;
the client `write-logs` a `Log` whose `message` is a user record (`:probe::Note`). The caller gets a mute:

```
recv failed: peer closed / channel disconnected   (runtime.rs recv', line 28 of the fixture)
```

`strace -f -s 4000` on the forked child revealed the TRUTH the caller never saw:

```
[pid …] write(2, "#wat.kernel/ProcessPanics [ … poll' (process tier): client message decode failed:
        src/edn_shim.rs:2424:45: unknown tag #probe/Note (body shape: map);
        no matching struct or enum in the type registry … ]", 687) = -1 EPIPE (Broken pipe)
[pid …] exit_group(1)
```

The child **had** a rich, located reason (687 bytes), formatted it correctly, and **wrote it to a pipe
whose read end was already closed** — it vanished on `EPIPE`. The whole service died (exit 1) over one
undecodable client message, and the caller got a hardcoded "peer closed."

## The masking sites (the full class, grounded)

| # | site | what it hides |
|---|---|---|
| R | `comms/mod.rs:899` | `RecvError {Disconnected, Shutdown, FrameTooLarge}` — **no slot for a reason** (the root) |
| 1 | `comms/process.rs:944` | `from_wire` **decode failure** → `Disconnected` (`|_|`) — hid `unknown tag #probe/Note` |
| 2 | `comms/process.rs:940` | UTF-8 failure → `Disconnected` (`|_|`) |
| 3 | `comms/process.rs:579,752,884,887` | I/O read errors (errno) → `Disconnected` (`|_|`) |
| 4 | `comms/process.rs:992` | `FrameScan::Malformed` → `Disconnected` |
| 5 | `channel/transfer.rs:172` | `FrameTooLarge` → `RecvOutcome::Disconnected` (collapse downstream) |
| 6 | `runtime.rs:26196` | socket `recv_wire()` error → `|_|` → hardcoded "peer closed" |
| 7 | `runtime.rs:26219` | `peer.recv()` → `|_|` → hardcoded "peer closed" — **discards `PeerRecvError::Crashed(reason)`** |
| 8 | `spawn.rs` err channel | child's crash-reason write **`EPIPE`s** — read end torn down before the child speaks |
| 9 | `service.wat:791` (`poll'`) | a client decode failure is **service-fatal** — one bad message kills every client |
| 10 | `services/peer.rs:114,120` | malformed `Req` field → `continue` **without replying** → the caller **hangs** |

Sites 6/7 preserve a reason *one branch down* already (`runtime.rs:26209` binds `|e|` for the decode
door) — the codebase knows how; the recv-side just doesn't do it.

## The strike — climb to "a mute failure has no form"

**Ratified (four-questions, 2026-07-17): Option A** — a per-message decode failure is a **client-scoped
error replied to that caller**, not a service-fatal crash. (A: Obvious/Simple/Honest/Good-UX all YES.
B "service-fatal + reason to admin only" fails Obvious/Simple/Honest — one client must not be able to
kill a shared service, and the requester must not be left blind.) A does not contradict arc-294: it
*reclassifies* — bad input goes back to the sender; a **genuine** crash still goes to the creator (and
must no longer `EPIPE`).

Ladder, top rung = unrepresentable:

1. **Give `RecvError` a reason.** Add a failure variant that carries the message, e.g.
   `RecvError::Failed(String)` (decode / utf8 / io / malformed all become `Failed(reason)`).
   `Disconnected` then means **only** a genuine clean EOF. *Wall:* "a real failure indistinguishable
   from a clean close" now has no representation. Exhaustive `match` sites (`channel/transfer.rs:165-172`,
   the `Display` impl) go red until they handle it — the compiler drives the sweep.
2. **Bind the error at every collapse.** Every `map_err(|_| RecvError::Disconnected)` on sites 1-4 →
   `map_err(|e| RecvError::Failed(e.to_string()))`. Site 4/5 `Malformed`/`TooLarge` keep their distinct
   variants but the *reason* rides along.
3. **Surface it in `recv'`.** `runtime.rs:26196/26219` stop the `|_|` + hardcoded string; thread the
   `RecvError`/`PeerRecvError` message (esp. `Crashed(reason)`) into the raised `MalformedForm`.
4. **Keep the crash channel alive until the reason is delivered.** The `EPIPE` (site 8) means the err
   read end drops before the dying child writes. Hold it open across the child's death so
   `emit_structured_exit`'s envelope lands, and route it to the creator (the `Handle`).
5. **`poll'` replies, doesn't crash (Option A).** A client decode failure returns a client-scoped error
   event the serve loop replies with (the rich reason), and the service keeps serving. Kill site 10's
   `continue`-without-reply (every `Req` gets a reply — the ZERO-MUTEX discipline already in
   `services/peer.rs:148`, applied to the process tier).

Steps 1-3 are one coherent change (the type + its fill sites + the surfacing). Steps 4 and 5 are the
crash-lifetime and the poll'-reply changes. Each lands with its own RED gate; all share the one law.

## The RED gate (acceptance — a NEW probe, diagnostic-scoped)

A forked-process service receives a client message it cannot decode. Assert BOTH:
- **the caller's error carries the real reason** — contains `unknown tag` / `decode failed` /
  `no matching struct or enum`, NOT the bare `peer closed / channel disconnected`; and
- **the service is still alive** — a subsequent request to the same service succeeds.

At HEAD both fail (mute "peer closed" + dead service). GREEN when steps 1-5 land. This is independent of
the `LogMessage`-opaque question (deliberately deferred) — it tests *diagnosis*, not the log-message shape.

**Content-integrity / no-regression:** whole floor back to exactly the standing `no_inlined_wat` lint,
zero new failures; a genuine handler panic in a process service still delivers its reason to the creator
(a second RED probe: a service whose handler raises → the `Handle` surfaces the reason, no `EPIPE`).

## The lesson this plants (for the next self, across the gap)

Failure-masking is one class, not N bugs. A recv/decode/io error that cannot carry its reason, a
`map_err(|_|)` that binds the error to nothing, a crash reason written to a closed pipe, a service that
dies over one client's bad message — all the same disease. The wall: **the error type carries the
reason, so `Disconnected` can mean only a clean goodbye, and a mute failure cannot be constructed.**
`RVINA ERVDIT` — the ruin must educate; a failure that cannot speak teaches nothing and induces exactly
the "guaranteed confusion and flailing" this arc forbids.
