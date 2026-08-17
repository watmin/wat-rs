;; wat/spawn.wat — the LOCUS opts for spawn-program (arc 259, The Forced Hand).
;;
;; The Keymaker.  (The Matrix Reloaded, 2003 — the little man in the Château who
;; cuts a different key for every door, and the right key is the only thing that
;; opens the backdoor.)  Each constructor below cuts exactly one key, for exactly
;; one locus-door:
;;
;;   (thread)   — cuts a trivial key; the door is right here in this process.
;;   (process)  — cuts a trivial key; the door is a forked child universe.
;;
;; A locus's TYPE is the whole message (where to execute); spawn-program is a clause-set
;; that matches on the key's type and opens the matching door. Every new kind of
;; locus that ever reveals itself is one new key + one new clause, the 2-arg
;; (spawn-program <locus> <prog>) sig unmoved.
;;
;; ⛔ THE REMOTE DOOR IS PERPETUALLY AWAITING ITS KEY.  `:remote` is the forcing
;; function (like `spawn-program :remote` itself): we agree a remote locus *must
;; materialize eventually* — and that whatever its opts record turns out to be, its
;; constructor's arity will be the lock (a remote locus that cannot reach its locus is
;; unrepresentable, the forced hand). But its STRUCT SHAPE IS NOT AGREED and must
;; NOT be guessed here — leaving the key uncut is the point. When the remote door's
;; lock is finally specified, `RemoteOpts` + its `(remote …)` constructor + a new
;; clause arrive together, the sig unmoved. Until then: deliberately absent.
;;
;; See docs/arc/2026/06/259-forced-hand/DESIGN.md § "The spawn primitive".
;; Loads AFTER wat/Record.wat (uses :wat::core::Record::def).

;; ── Arc 272 6c.2 — SocketAddressWire (the portable address capability record) ──
;; The portable form of a process-tier Address': minter-pid + autobind name bytes
;; (as Vector<i64>, since wat has no byte scalar). Encodes as:
;;   #wat.kernel/Address #wat.kernel/SocketAddressWire {:minter-pid 4242 :name [1 2 3 4 5]}
;; The cap codec builds/reads this record; the connect gate verifies minter-pid.
(:wat::core::defrecord :wat::kernel::SocketAddressWire
  [minter-pid <- :wat::core::i64
   name       <- :wat::core::Vector<wat::core::i64>])

;; ── Per-env launch records (what each env hands the post-spawn hook) ─────────
;; ThreadLaunch is empty — no fields yet; grows if a need appears (don't build
;; the forcing function). ProcessLaunch carries the child pid, owner-side.
(:wat::core::defrecord :wat::spawn::ThreadLaunch [])
(:wat::core::defrecord :wat::spawn::ProcessLaunch [pid <- :wat::core::i64])

;; ── The keys (locus opts records) ───────────────────────────────────────────
;; ThreadOpts carries an init-fn: a 0-arg fn returning a :wat::core::Record.
;; The init-fn runs at the peer's start and populates user-data.
;; ProcessOpts carries no config — its TYPE is the whole message.
;; Both opts records carry post-spawn-fn: an owner-side fn that runs after
;; the peer is spawned, before spawn-program' returns, for effects. Receives
;; the per-env launch record. Required with a no-op default on the bare ctors.
(:wat::core::defstruct :wat::spawn::ThreadOpts
  [init-fn       <- :wat::core::Fn()->wat::core::Record
   post-spawn-fn <- :wat::core::Fn(wat::spawn::ThreadLaunch)->wat::core::nil
   runner-count  <- :wat::core::i64])
;; Arc 170 gap J — `uses` (the locus-carried Vector<(keyword,Capability)>) is RETIRED. The
;; bracket's per-worker provisioning (grant + Setup dial) is now an ORTHOGONAL layer riding on
;; `map`/`each` themselves (wat/bracket.wat's `map-worker`, absorbing the former `uses'`), not a
;; field baked into the locus opts. A locus is once again just "where to execute" (thread vs
;; process) — the pool coordinator decides what to provision, per call, not the key.
;; `label` — arc 170 closure #6: the ps-visible identity. Named `label`, not
;; `identity` — cast + builder-ratified: `exec_plan.rs:29-35` rejects a
;; `--forms-server` routing flag because it would be "a CLAIM where this is a
;; WITNESS"; `identity` reads as a claim, `label` reads as a witness. THE NAME
;; CARRIES THE INVARIANT. `Option`, not a bare Record — "no label declared" is
;; a real state (a test harness spawning a bare program) and `None` must mean
;; NO label at all (today's bare `wat` line), never an empty `{}` that says
;; nothing. Set ONCE at spawn time, fixed for the process's whole lifetime
;; (builder ruling: "these would be fixed at boot — the procs are purpose
;; built"). It is a VALUE, unlike `env-fn` (a source string the CHILD evals in
;; a world that does not exist parent-side — see the arc 170 closure #6 STOP
;; that killed evaluating env-fn parent-side) — the label is a static fact the
;; SPAWNER already knows, so it reaches `ExecPlan::build()` directly.
;; DESCRIBES only; never ROUTES — see `:wat::spawn::with-label` and
;; `src/process/exec_plan.rs`'s wall doc. The record's TYPE is a closed set
;; the substrate owns (`:wat::process::Bracket` | `:wat::process::Service`,
;; wat/process.wat) — no caller mints its own tag, so `ps` output stays a set
;; an operator can learn once and match exhaustively.
(:wat::core::defstruct :wat::spawn::ProcessOpts
  [post-spawn-fn    <- :wat::core::Fn(wat::spawn::ProcessLaunch)->wat::core::nil
   env-fn           <- :wat::core::String
   max-message-bytes <- :wat::core::i64
   runner-count      <- :wat::core::i64
   label             <- (:wat::core::Option :wat::core::Record)])

;; Default max-message-bytes budget for process peers — mirrors DEFAULT_MAX_FRAME_BYTES
;; in src/edn_shim.rs:1008.  Do NOT scatter the literal: change it here and there
;; together.  512 KiB = 524288 bytes.
(:wat::core::def :wat::spawn::DEFAULT-MAX-MESSAGE-BYTES 524288)

;; ── The Keymaker's friendly hand (ergonomic constructors) ────────────────────
;; (thread)             — default init-fn + no-op post-spawn-fn; runner-count defaults to cpu-count.
;; (thread/init f)      — init-fn is f; post-spawn-fn defaults to no-op; runner-count defaults to cpu-count.
;; (thread/post-spawn g)— init-fn defaults to EmptyEnv; post-spawn-fn is g; runner-count defaults to cpu-count.
;; (thread/runner-count n) — init-fn + post-spawn-fn default; runner-count is n.
;; (process)            — no-op post-spawn-fn; env-fn defaults to EmptyEnv ctor; default budget; runner-count defaults to cpu-count.
;; (process/post-spawn f)— post-spawn-fn is f; env-fn defaults to EmptyEnv ctor; default budget; runner-count defaults to cpu-count.
;; (process/env s)       — env-fn is s; post-spawn-fn defaults to no-op; default budget; runner-count defaults to cpu-count.
;; (process/max-message-bytes n) — budget is n; post-spawn-fn + env-fn default; runner-count defaults to cpu-count.
;; (process/runner-count n) — post-spawn-fn/env-fn/max-message-bytes default; runner-count is n.
(:wat::core::defn :wat::spawn::thread [] -> :wat::spawn::ThreadOpts
  (:wat::spawn::ThreadOpts
    :init-fn (:wat::core::fn [] -> :wat::core::Record (:wat::program::EmptyEnv))
    :post-spawn-fn (:wat::core::fn [_l <- :wat::spawn::ThreadLaunch] -> :wat::core::nil nil)
    :runner-count (:wat::program::cpu-count)))

(:wat::core::defn :wat::spawn::thread/init [f <- :wat::core::Fn()->wat::core::Record] -> :wat::spawn::ThreadOpts
  (:wat::spawn::ThreadOpts :init-fn f
    :post-spawn-fn (:wat::core::fn [_l <- :wat::spawn::ThreadLaunch] -> :wat::core::nil nil)
    :runner-count (:wat::program::cpu-count)))

(:wat::core::defn :wat::spawn::thread/post-spawn [g <- :wat::core::Fn(wat::spawn::ThreadLaunch)->wat::core::nil] -> :wat::spawn::ThreadOpts
  (:wat::spawn::ThreadOpts
    :init-fn (:wat::core::fn [] -> :wat::core::Record (:wat::program::EmptyEnv))
    :post-spawn-fn g
    :runner-count (:wat::program::cpu-count)))

(:wat::core::defn :wat::spawn::thread/runner-count [n <- :wat::core::i64] -> :wat::spawn::ThreadOpts
  (:wat::spawn::ThreadOpts
    :init-fn (:wat::core::fn [] -> :wat::core::Record (:wat::program::EmptyEnv))
    :post-spawn-fn (:wat::core::fn [_l <- :wat::spawn::ThreadLaunch] -> :wat::core::nil nil)
    :runner-count n))

(:wat::core::defn :wat::spawn::process [] -> :wat::spawn::ProcessOpts
  (:wat::spawn::ProcessOpts
    :post-spawn-fn (:wat::core::fn [_l <- :wat::spawn::ProcessLaunch] -> :wat::core::nil nil)
    :env-fn "(:wat::program::EmptyEnv)"
    :max-message-bytes :wat::spawn::DEFAULT-MAX-MESSAGE-BYTES
    :runner-count (:wat::program::cpu-count)
    :label :wat::core::None))

(:wat::core::defn :wat::spawn::process/post-spawn [f <- :wat::core::Fn(wat::spawn::ProcessLaunch)->wat::core::nil] -> :wat::spawn::ProcessOpts
  (:wat::spawn::ProcessOpts :post-spawn-fn f :env-fn "(:wat::program::EmptyEnv)" :max-message-bytes :wat::spawn::DEFAULT-MAX-MESSAGE-BYTES :runner-count (:wat::program::cpu-count) :label :wat::core::None))

(:wat::core::defn :wat::spawn::process/env [s <- :wat::core::String] -> :wat::spawn::ProcessOpts
  (:wat::spawn::ProcessOpts
    :post-spawn-fn (:wat::core::fn [_l <- :wat::spawn::ProcessLaunch] -> :wat::core::nil nil)
    :env-fn s
    :max-message-bytes :wat::spawn::DEFAULT-MAX-MESSAGE-BYTES
    :runner-count (:wat::program::cpu-count)
    :label :wat::core::None))

(:wat::core::defn :wat::spawn::process/max-message-bytes [n <- :wat::core::i64] -> :wat::spawn::ProcessOpts
  (:wat::spawn::ProcessOpts
    :post-spawn-fn (:wat::core::fn [_l <- :wat::spawn::ProcessLaunch] -> :wat::core::nil nil)
    :env-fn "(:wat::program::EmptyEnv)"
    :max-message-bytes n
    :runner-count (:wat::program::cpu-count)
    :label :wat::core::None))

(:wat::core::defn :wat::spawn::process/runner-count [n <- :wat::core::i64] -> :wat::spawn::ProcessOpts
  (:wat::spawn::ProcessOpts
    :post-spawn-fn (:wat::core::fn [_l <- :wat::spawn::ProcessLaunch] -> :wat::core::nil nil)
    :env-fn "(:wat::program::EmptyEnv)"
    :max-message-bytes :wat::spawn::DEFAULT-MAX-MESSAGE-BYTES
    :runner-count n
    :label :wat::core::None))

;; ── The tier-blind reader (runner-count as a defclause) ──────────────────────
;; A caller holding an abstract :wat::spawn::Locus value reads the pool count without a
;; per-type accessor: the defclause dispatches on the concrete locus class (exactly as
;; spawn-program' dispatches on ThreadOpts | ProcessOpts). A new locus type joins as one
;; more clause here; the 1-arg sig is unmoved.
(:wat::core::defclause :wat::spawn::runner-count
  ([locus <- :wat::spawn::ThreadOpts]  -> :wat::core::i64  (:wat::spawn::ThreadOpts/runner-count locus))
  ([locus <- :wat::spawn::ProcessOpts] -> :wat::core::i64  (:wat::spawn::ProcessOpts/runner-count locus)))

;; ── ServiceEvent<I,O> — the poll' return type ───────────────────────────────
;;
;; Arc 209 Stone C0b.1b / C0b.2e-i-c.  A service `poll'` over `(self-peer, listener, peers)`
;; — the service multiplexer — returns one of five events:
;;   :Shutdown    — the owner dropped the service handle; RAII drain disconnected
;;                  the self-peer → the loop exits. DEADLOCK-FREE termination,
;;                  structural: dropping the handle IS the shutdown (no Stop op).
;;   :Connection  — a dialing client was accepted; the new Peer' is ready.
;;   :Message     — peers[idx] sent an op; `msg` is the received value.
;;   :Closed      — peers[idx] left gracefully (clean EOF, no diagnostic).
;;   :Lost        — transport broke abnormally; `cause` is the first-class
;;                  Failure diagnostic (ECONNRESET / ETIMEDOUT / …).
;;                  Emitted by the remote tier; thread poll' emits only
;;                  Shutdown/Connection/Message/Closed — :Lost is built for the union.
;;   :Malformed   — peers[idx] sent a message the service COULD NOT DECODE (arc 278
;;                  no-hidden-failures). The peer is STILL ALIVE — a bad message is
;;                  NOT a death; `cause` is the first-class Failure carrying the rich
;;                  decode reason (`unknown tag #probe/Note … no matching struct or
;;                  enum in the type registry`). The serve loop replies the cause to
;;                  the originating client and KEEPS SERVING (does NOT evict — distinct
;;                  from :Lost). Emitted by the process/socket tier poll' (the only
;;                  tier that decodes a wire; thread peers pass Values in-process).
;;
;; Type params: I = the type the server SENDS to peers (peer's recv type);
;;              O = the type the server RECEIVES from peers (peer's send type);
;;              A = the type the server RECEIVES from the owner (admin ops).
;; Mirror Peer'<I,O>: the accepted peer is Peer'<I,O>, message is O.
;; Arc 291 3a-i: A is the self-peer's receive type (owner→service admin channel).
;;
(:wat::core::defenum :wat::spawn::ServiceEvent<I,O,A> :wat::enum::Impure
  :Shutdown                                                              ;; owner dropped the handle (self-peer drained) — exit; deadlock-free termination
  :Admin      [msg   <- :A]                                             ;; owner sent an admin op over the lineage peer (Ok path); A = self-peer's recv type
  :Connection [peer  <- :wat::kernel::Peer<I,O>]
  :Message    [idx   <- :wat::core::i64  msg   <- :O]
  :Closed     [idx   <- :wat::core::i64]
  :Lost       [idx   <- :wat::core::i64  cause <- :wat::kernel::Failure]
  :Malformed  [idx   <- :wat::core::i64  cause <- :wat::kernel::Failure]   ;; arc 278: peer ALIVE, message undecodable — reply cause + keep serving
  :Rejected   [idx   <- :wat::core::i64  cause <- :wat::kernel::Failure])   ;; arc 278 Stone 1a: over-FOO (400-class) — reply cause (non-blocking) + EVICT + keep serving

;; ── PoolMsg<D,I> — the universal pool wire message (arc 170 M1-pool) ──────────
;;
;; Every bracket pool runner recv's PoolMsg<D,I>, never a raw (i64,I) pair, so the
;; SAME peer type carries the dial handshake AND the work stream — the one shape
;; that lets a tier-agnostic map-worker send both. `:Pure` — proven to cross the
;; wire by scratchpad/probe-m1-worker-setup.wat (a :Pure enum, Address' payload).
;; It lives HERE (not wat/bracket.wat) because the :wat::spawn::Locus surface's
;; `spawn-runner` return type names it, and that surface loads before bracket.wat.
;;
;;   :Setup [deps <- :D]        — hand a granted service ADDRESS (a capability crossing
;;                                the WIRE, never as closure data — ocap). D = the
;;                                dial-target Address' type.
;;   :Work  [pair <- (i64,I)]   — one indexed unit of work (idx round-trips order).
;;
;; The enum NAME is the wire tag (`#wat.bracket.PoolMsg/Setup`); the D/I type-params
;; are NOT in the tag. So the PARENT (map-worker) holds a type-ERASED
;; `PoolMsg<Address',(i64,I)>` (bare Address' — derived per-handle from the locus's `:uses`
;; field via each handle's `coordinate` method, keeping ProcessOpts non-parametric) while the
;; CHILD's baked dial-runner holds the CONCRETE `PoolMsg<Address'<S,R>,I>` (S,R off the
;; work-fn's peer param). Same name ⇒ the wire round-trips; the Setup payload encodes as
;; SocketAddressWire either way. A thread/non-dial pool simply never sends :Setup (D stays
;; phantom).
(:wat::core::defenum :wat::bracket::PoolMsg<D,I> :wat::enum::Pure
  :Setup [deps <- :D]
  :Work  [pair <- :(wat::core::i64,I)])

;; ── Spawned — the owner-side spawn-handle marker ────────────────────────────
;; Spawned — the owner-side spawn-handle marker (typesub/derive axis; no methods). Thread'/Process'/
;; future-remote derive it so the locus-agnostic Handle field + Locus/spawn return can bind any of them.
;; Lifecycle = close'/join (intrinsics). A new transport's handle joins with one more `derive`.
(:wat::core::derive :wat::kernel::Thread  :wat::spawn::Spawned)
(:wat::core::derive :wat::kernel::Process :wat::spawn::Spawned)

;; ── arc 291 3a-ii-β: Thread'/Process' ARE Peer's ────────────────────────────
;; The owner-side spawn handle IS the parent end of the lineage channel — a peer.
;; send'/recv'/poll' already operate on it (process `launch` does `recv' svc`/`send' svc`);
;; these derives make the TYPE model say so, so a locus-agnostic `Handle.handle <- Peer'<…>`
;; field binds ANY spawn handle. N-LOCI-GENERAL: a future remote locus joins the peer family
;; with ONE more `derive` line — zero edits to the assignable rule, which is driven by THIS
;; derive graph (check.rs `assignable`, the Parametric<:Parametric arm). Never a 2-only assumption.
(:wat::core::derive :wat::kernel::Thread  :wat::kernel::Peer)
(:wat::core::derive :wat::kernel::Process :wat::kernel::Peer)

;; ── arc 293.W.2d / arc 278 — a wire-safe Peer' IS usable in-locus ────────────
;; THE LINE IS SHARED MEMORY OR NOT, and it is DIRECTIONAL:
;;   ThreadSelfPeer'<S,R>  in-locus, ANY I/O   (the escape hatch for peers holding live handles)
;;   Peer'<S,R>            wire-safe, PURE I/O only
;; `Peer'` is STRICTLY STRICTER, so it satisfies every constraint a `ThreadSelfPeer'` position
;; imposes — one derive states the relation the checker previously enumerated by hand at ~7
;; sites (check.rs 9835 / 10159 / 11091 / poll'-select' self / the 10176 error string).
;; Args stay INVARIANT (the Parametric<:Parametric arm unifies them); this is a HEAD edge only.
;;
;; ⛔ ONE-WAY, AND THE OMISSION IS THE WALL. The reverse — ThreadSelfPeer' derives Peer' — must
;; NEVER be written: it would launder an in-locus peer (live crossbeam handles) into a wire-safe
;; position and arc 293.W's mobility guarantee is gone. An un-written rule is invisible, so the
;; absence is made enforceable by a negative gate:
;;   tests/services/probe_arc293w_peer_derives_threadselfpeer.wat.bad  (must stay RED, forever)
;; Same discipline that keeps `:wat::core::Value` from degrading into an `any` (278 R7).
;;
;; NOTE this cannot weaken the WIRE wall: `is_pure_type` (check.rs ~12979) refuses ALL FOUR peer
;; heads by NAME in an exhaustive match — "they are resources — they are not pure" (builder,
;; 2026-08-03). A subtype edge does not touch a head-keyed match; only ADDRESSES cross (293.W).
;;
;; WHY IT WAS NEEDED (arc 278): defservice's generated child main reached `serve` through
;; `(apply (keyword/from-string …))` — a call that existed BECAUSE it did not resolve statically,
;; so no closure walk could follow it. The process tier holds a `Peer'` and `serve` declares a
;; `ThreadSelfPeer'`; this edge is what lets that call be STATIC.
(:wat::core::derive :wat::kernel::Peer :wat::kernel::ThreadSelfPeer)

;; ── Shared / Wire — phantom transport markers (293.W.2f) ─────────────────────
;; Type arguments only. Not values. The third argument of Address<S,R,T>:
;;   Shared — in-locus (crossbeam). A process may never dial this.
;;   Wire   — portable (SocketAddressWire). A process may hold and dial this.
(:wat::core::defstruct :wat::kernel::Shared [])
(:wat::core::defstruct :wat::kernel::Wire [])

;; ── Bound<S,R,T> — the listening state minted by (listener' (thread) :S :R) ─────
;; A STRUCT, not a record: its fields are non-EDN RustOpaque kernel entities
;; (Listener'/Address'). `listener` is the server accept-side; `address` is what
;; clients dial via connect'. Replaces the bare Tuple the thread tier returned.
;; T is the transport marker (Shared | Wire); 2-arg Bound<S,R> still means T unknown.
(:wat::core::defstruct :wat::spawn::Bound<S,R,T>
  [listener <- :wat::kernel::Listener<S,R>
   address  <- :wat::kernel::Address<S,R,T>])

;; ── Launched<S,R,Sh,Lu,T> — what Locus/launch returns: the spawn handle + the dial address ──
;; A STRUCT, not a record (address is an Address' RustOpaque; handle is :Spawned).
;; `handle` is the owner-side spawn handle (Thread'/Process'/future-remote all derive :Spawned).
;; `address` is what clients dial via connect'.
;; `start` unwraps Launched into the Handle record — locus-agnostic launch, locus-agnostic start.
;; arc 291 3a-ii-β: handle is the lineage PEER (Peer'<Sh,Lu> — sends Sh=Admin, recvs
;; Lu=LineageUp), no longer the opaque :Spawned marker. Thread'<Sh,Lu>/Process'<Sh,Lu>
;; bind it via the `derive …Peer'` foundation. This is what makes owner-only `stop` able
;; to send'/recv' on the Handle's handle. S,R = the client (listener/dial) channel.
;; T is the transport marker (Shared | Wire); 4-arg Launched<S,R,Sh,Lu> still means T unknown.
(:wat::core::defstruct :wat::spawn::Launched<S,R,Sh,Lu,T>
  [handle  <- :wat::kernel::Peer<Sh,Lu>
   address <- :wat::kernel::Address<S,R,T>])

;; ── The Keymaker's masterwork (the spawn-program' defclause) ─────────────────
;;
;; Arc 259 S2c-ii-b — `spawn-program'` as a locus-type defclause.
;;
;; 2-arg `(locus prog)` — the key's TYPE (ThreadOpts | ProcessOpts) selects
;; the matching locus door and delegates to the S2c-i tier primitives.
;; The env arg of the 3-arg intrinsic is gone (it was discarded at runtime;
;; the defclause makes the absence structural).
;;
;; Thread clause: prog MUST be the self-peer model `[Peer'<S,R>] -> nil`
;; (apply-loop purged by S2c-ii-a; the true form remains).
;; Process clause: prog is a `(:wat::core::forms ...)` block — a forms-server
;; program (`Vector<wat::WatAST>`) for the forked child universe.
;;
;; A new locus type (e.g. RemoteOpts when its door is finally specified)
;; arrives as one new key + one new clause here; the 2-arg sig is unmoved.
(:wat::core::defclause :wat::kernel::spawn-program
  ;; ── The IPC wall (arc 170 #13) ───────────────────────────────────────────
  ;; Spawning a locus is a CAPABILITY, not a verb anyone may reach for. The
  ;; whitelist is the two namespaces that legitimately hold it:
  ;;   :wat::spawn:: — the Locus surface impls (extend-type ThreadOpts/ProcessOpts
  ;;                   :wat::spawn::Locus; their method FQDNs are
  ;;                   `:wat::spawn::<T>/<method>`, so the prefix matches even for
  ;;                   the impls that live in wat/bracket.wat)
  ;;   :wat::test::  — the harness capability holders (spawn-thread-program /
  ;;                   spawn-hermetic-program in wat/test.wat)
  ;; NOT :wat::kernel:: — that is where spawn-program is DEFINED, not called from.
  ;; The tier primitives below it (spawn-thread / spawn-process) are separately
  ;; walled in Rust via #[restricted_to] + the inventory drain.
  ;;
  ;; The check attributes a call to its ENCLOSING FN, and for a macro-spliced
  ;; call that is the EXPANSION SITE — so a macro may not emit this call into
  ;; user code (capability laundering); it must route through a named fn inside
  ;; the whitelist. wat/test.wat's run-thread/run-hermetic do exactly that.
  {:restricted-to [:wat::spawn:: :wat::test::]}
  ;; thread — the ONE true form (self-peer; apply-loop is the annihilated heresy).
  ;; The locus's init-fn (extracted via ThreadOpts/init-fn) runs at the peer's start.
  ;; The locus's post-spawn-fn (extracted via ThreadOpts/post-spawn-fn) runs owner-side
  ;; after the peer is spawned, before spawn-program' returns, for effects.
  ;; Arc 293.W.2d: thread programs take ThreadSelfPeer' (in-locus, any I/O) as the self
  ;; parameter. Peer' is the wire-capable peer (pure I/O only); ThreadSelfPeer' is the
  ;; in-locus escape hatch for thread workers that carry Sender/Receiver or other impure types.
  ([locus <- :wat::spawn::ThreadOpts
    prog <- [:wat::kernel::ThreadSelfPeer<S,R> :-> :wat::core::nil]] -> :wat::kernel::Thread<R,S>
    (:wat::kernel::spawn-thread prog (:wat::spawn::ThreadOpts/init-fn locus) (:wat::spawn::ThreadOpts/post-spawn-fn locus)))
  ;; process — forms (Vector<wat::WatAST>); I,O are the forms-server's free request/response vars.
  ;; The locus's post-spawn-fn (extracted via ProcessOpts/post-spawn-fn) runs owner-side
  ;; after the child is forked, with a ProcessLaunch{pid} carrying the child pid.
  ;; The locus's env-fn (extracted via ProcessOpts/env-fn) is a source string the child
  ;; evals in its own frozen world to produce user-data.
  ;; The locus's label (extracted via ProcessOpts/label) is arc 170 closure #6's
  ;; ps-visible identity — a VALUE (unlike env-fn), read straight off the locus.
  ([locus <- :wat::spawn::ProcessOpts
    prog <- :wat::core::Vector<wat::WatAST>] -> :wat::kernel::Process<I,O>
    (:wat::kernel::spawn-process prog (:wat::spawn::ProcessOpts/post-spawn-fn locus) (:wat::spawn::ProcessOpts/env-fn locus) (:wat::spawn::ProcessOpts/max-message-bytes locus) (:wat::spawn::ProcessOpts/label locus))))

;; ── Locus — the locus-agnostic service-launch surface (arc 209 host-parity-4a) ─
;;
;; defservice's `start [locus <- :Locus]` routes the per-tier service launch through
;; this surface. `listener'` is locus-blind on its own (its checker accepts an
;; abstract :Locus and dispatches the Bound shape on arity; the runtime dispatches
;; on the concrete value) — but the PROGRAM handed to spawn-program' is
;; shared-vs-not-shared specific: thread captures a closure over the in-memory
;; listener/state; process ships forms ([[project_shared_memory_partition_hosting]]).
;; So `launch` MINTS THE LISTENER INSIDE the concrete impl (arc 272 6a: the child
;; must mint its own listener; parent-minting is wrong for the process tier) and
;; returns a Launched<S,R>{handle,address}. `start` unwraps Launched — locus-agnostic.
;; A new transport joins as one `extend-type`, zero edit to `start`.
;;
;; Generic over S,R (the listener/peer channel types) and St (service state).
;; `serve` is the per-service serve loop, passed by NAME (a runtime keyword) so
;; the impl invokes it tier-neutrally via `apply` — the thread impl captures and
;; applies; a future process impl ships forms that apply the same keyword.
;; serve's shape: (serve self-peer listener clients next-id state) -> nil. (arc 278 the call
;; context added `next-id`, the monotonic conn-id counter, as the 4th positional arg.)
(:wat::core::defsurface :wat::spawn::Locus :nature :wat::core::Struct
  ;; arc 291 3a-ii-β: Lu = the lineage UP type (LineageUp); Sh = the ship/admin DOWN type.
  ;; The returned Launched carries the lineage peer as Peer'<Sh,Lu>.
  :features
  [(launch<S,R,St,Sh,Lu> [self          <- :wat::spawn::Locus
                          ship          <- :Sh
                          init          <- :wat::core::keyword
                          serve         <- :wat::core::keyword
                          service-forms <- :wat::core::Vector<wat::WatAST>
                          lu-addr-kw    <- :wat::core::keyword
                          ;; arc 278 startup-crash parity: lu-mk-kw is the CONSTRUCTOR twin of
                          ;; lu-addr-kw (which extracts the addr FROM the lineage-up value). It builds
                          ;; the lineage-up value FROM the address — for defservice, Status::Started.
                          ;; The thread tier uses it so its serve closure (built generically here, with
                          ;; no per-service Status ctor in scope) can send Status::Started AFTER :init
                          ;; runs, making an :init crash surface over the crash-aware launch handshake
                          ;; instead of deadlocking the owner's connect'. Process ignores it (its
                          ;; child-main-form owns the ctor).
                          lu-mk-kw      <- :wat::core::keyword] -> :wat::spawn::Launched<S,R,Sh,Lu>)
   ;; Arc 170 M1-pool — work-fn is a GENERIC W (not `Fn(I)->O`): the thread/non-dial
   ;; tiers pass a 1-param `Fn(I)->O`, the process DIAL tier a 2-param `Fn(Peer'<S,R>,I)->O`.
   ;; The impl reifies (process, fn-forms) or applies (thread, unifying W~Fn(I)->O locally)
   ;; it. The runner recv type is PoolMsg<D,I> (the universal pool wire): send it the
   ;; work-stream AND the dial Setup over ONE peer type.
   ;;
   ;; Arc 170 gap J — D-GENERIC (was a fixed bare `Address'`): the Setup carrier is now
   ;; whatever `map-worker` (the sole caller, wat/bracket.wat) is instantiated with — `nil`
   ;; for a plain pool (no dial ever sent), the work-fn's own `<base>::Coords` record for a
   ;; kwargs pool. This is what lets ONE pool coordinator carry both provisionings: the
   ;; carrier is never welded into this surface's return type, only named by it.
   (spawn-runner<D,I,O,W> [self    <- :wat::spawn::Locus
                           work-fn <- :W]
     -> :wat::kernel::Peer<wat::bracket::PoolMsg<D,I>,(wat::core::i64,O)>)])

;; ── with-label — attach the ps-visible identity to a locus (arc 170 closure #6) ──
;; Locus-agnostic so both a defservice's `start`/`resume` and bracket's `map-worker`
;; can set a label without knowing the concrete locus type. ThreadOpts arm is a
;; no-op (a thread peer shares the parent's `ps` line — there is nothing to label,
;; and ThreadOpts carries no `label` field). ProcessOpts arm rebuilds with
;; `:label (Some r)`, overwriting whatever was there (last call wins — matches
;; "fixed at boot", since nothing calls this after a locus is actually spawned).
;; The INTENDED vocabulary is the two substrate-owned identity types
;; (`:wat::process::Bracket` | `:wat::process::Service`, wat/process.wat) — SHAPE 2 was
;; ratified over SHAPE 1 precisely so `ps` output is a set an operator learns once and
;; matches exhaustively.
;;
;; ⚠ THAT RESTRICTION IS A CONVENTION HERE, NOT A WALL — say so rather than let a comment
;; claim a guarantee the code does not make. NOTHING closes the set today: `:R` is a
;; wildcard at runtime dispatch (`is_type_var` -> `return true`, runtime.rs) and a free
;; type-var to the checker, and `ProcessOpts/label` is `(Option Record)` — the record-TOP,
;; which by construction admits every record. So ANY record type-checks and dispatches as a
;; ps label; `ps` output is, as it stands, the OPEN set SHAPE 1 was rejected for.
;; PROVEN, not asserted: wat-scripts/scratch-pad/probe-label-closed-set.wat mints a rogue
;; `:probe::Rogue` record, hands it to this clause, and type-checks GREEN. That probe is the
;; live witness — it goes RED the day the set is genuinely closed, which is its whole job.
;;
;; `r`'s param type is the bare type-var `:R`, NOT `:wat::core::Record` — a runtime-dispatch
;; gap, grounded, not a style choice: `defclause`'s dispatcher (`value_matches_type_by_name`,
;; runtime.rs) matches a concrete-record-typed param against the value's EXACT `class`
;; (`bare_p == a.class`), the same rule that lets it discriminate `:user::Tag`-shaped clauses
;; — it does NOT walk the assignability/subtype lattice the STATIC checker does. Declaring
;; this param `:wat::core::Record` type-checks fine (the checker DOES know Bracket/Service are
;; Record subtypes) but then fails EVERY call at runtime (`Bracket`/`Service` != the literal
;; string `"wat::core::Record"`). A bare type-var is a WILDCARD at runtime dispatch by the
;; same function (`is_type_var` arm) — exactly the posture `process-work-forms`'s generic `:W`
;; clause already relies on (bracket.wat) — so dispatch here rests entirely on the FIRST
;; param (the locus's concrete type), which is the one that actually needs to discriminate.
;; DESCRIBES only, never crosses as anything but inert EDN (see ProcessOpts' `label` field doc).
(:wat::core::defclause :wat::spawn::with-label
  ([locus <- :wat::spawn::ThreadOpts   _r <- :R] -> :wat::spawn::Locus locus)
  ([locus <- :wat::spawn::ProcessOpts  r  <- :R] -> :wat::spawn::Locus
    (:wat::spawn::ProcessOpts
      :post-spawn-fn     (:wat::spawn::ProcessOpts/post-spawn-fn locus)
      :env-fn            (:wat::spawn::ProcessOpts/env-fn locus)
      :max-message-bytes (:wat::spawn::ProcessOpts/max-message-bytes locus)
      :runner-count      (:wat::spawn::ProcessOpts/runner-count locus)
      :label             (:wat::core::Some r))))

;; ── Arc 278 Strike A — the ONE canonical Failure constructor ─────────────────
;; `:wat::kernel::Failure` is canonically a Record (Nature::Record, pure EDN — arc 293.W.2b:
;; a crash cause crosses the wire and only a Record round-trips EDN), but construction was
;; never unified — several sites hand-rolled it via `:wat::core::struct-new`, which mints the
;; WRONG nature (a Struct), so `Failure/message` (a Record accessor) can't read it back
;; (TypeMismatch). This is the one message-only constructor (the common reason-free case,
;; e.g. a client-side peer-lost cause that deliberately scrubs the owner's real reason) for
;; every call site to route through instead of hand-rolling. Field values mirror Rust's
;; `message_only_failure` exactly (arc 278 the string-wrap annihilation): the mandatory `error`
;; carries a SYNTHESIZED `:wat::core::Fault` (from `msg`; `Failure/message` derives back to it),
;; actual/expected empty, frames empty. Bare-positional construction of a builtin record is retired
;; (arc-294 9a's kwargs flip) — this is the kwargs ctor, proven in
;; wat-scripts/scratch-pad/probe-failure-record-ctor.wat. Homed here (loads well before
;; wat/service.wat, its first client) because this file already owns the recv'-outcome /
;; Failure/message crash-parity pattern (see the two `assertion-failed!` sites below).
(:wat::core::defn :wat::kernel::message-only-failure [msg <- :wat::core::String] -> :wat::kernel::Failure
  (:wat::kernel::Failure
    :error (:wat::core::Fault/of msg)
    :frames (:wat::core::Vector :wat::kernel::Frame)
    :actual :wat::core::None
    :expected :wat::core::None))

;; Thread (shared-memory) impl — mints the listener internally via (listener' self :S :R)
;; (the method's type-params S,R flow as type-args — arc-232 dep proven GREEN).
;; Builds the serve closure capturing the minted listener + empty clients vector + state0;
;; spawn-program' (thread) runs it on a freshly-spawned peer. serve is invoked by keyword
;; via apply so this generic impl never names the per-service serve fn.
;; Returns Launched{handle=Thread', address=Bound/address}.
;; service-forms: thread arm ignores it (serve is already in the parent universe).
(:wat::core::extend-type :wat::spawn::ThreadOpts :wat::spawn::Locus
  (launch [self ship init serve service-forms lu-addr-kw lu-mk-kw]
    (:wat::core::let
      ;; arc 278 startup-crash parity: the thread tier gains a Status::Started handshake it
      ;; previously LACKED (it returned the parent-minted address immediately, so an :init
      ;; crash left a bound-but-never-accepted address → the owner's connect' deadlocked on
      ;; the rendezvous). Now the child runs :init FIRST, then sends Status::Started UP; the
      ;; parent blocks on the crash-aware `recv' sp` before returning. An :init crash EOFs the
      ;; self-peer output + puts the reason on crash_tx (kernel/spawn.rs) → `recv' sp` RAISES
      ;; the reason (parity with the honest serve-loop-crash path), instead of hanging.
      [b  (:wat::kernel::listener self :S :R)
       sp (:wat::kernel::spawn-program self
            (:wat::core::fn [self-peer <- :wat::kernel::ThreadSelfPeer<Lu,Sh>] -> :wat::core::nil
              (:wat::core::let
                ;; :init runs BEFORE Started is sent — a crash here dies before the send.
                [st (:wat::core::apply  init ship [])
                 ;; arc 278 the send'-outcome wall — the crash-aware `recv' sp` right below
                 ;; (parent side) faces Closed/Lost on this handshake; the child's own send'
                 ;; here just needs to proceed regardless (never a `_`-swallow).
                 _  (:wat::core::match (:wat::kernel::send self-peer
                        (:wat::core::apply  lu-mk-kw (:wat::spawn::Bound/address b) []))
                      (:wat::kernel::SendOutcome::Sent   nil)
                      (:wat::kernel::SendOutcome::Closed nil)   ;; parent's recv' already faces this
                      ;; arc 278 #73 — a stop arrived mid-handshake. Same body as the two
                      ;; above, and the PRECONDITION is why that is legal here rather than a
                      ;; discard: this is the CHILD announcing readiness, and the parent's
                      ;; crash-aware `recv' sp` below faces every terminal outcome of this
                      ;; handshake — including its own Stopped. Deciding here would decide it
                      ;; twice. The child proceeds into `serve`, whose poll' faces the stop.
                      (:wat::kernel::SendOutcome::Stopped nil)
                      ((:wat::kernel::SendOutcome::Lost _c) nil))]
                ;; arc 278 the call context — `serve`'s wiring contract is now 5 args, not 4:
                ;; `(serve self-peer listener clients next-id state) -> nil`. The extra `0` is
                ;; the initial monotonic conn-id counter (defservice's serve loop threads it as
                ;; pure state from here; a hand-rolled serve ignoring it is unaffected).
                (:wat::core::apply  serve self-peer
                  (:wat::spawn::Bound/listener b)
                  (:wat::core::Vector :wat::kernel::Peer<R,S>)
                  0
                  st []))))
       ;; Crash-aware readiness barrier: value discarded (the parent already holds the address).
       ;; arc 278 the recv'-outcome wall — recv' returns a matchable RecvOutcome. ::Message → the
       ;; child reached readiness (discard + proceed); ::Lost (an :init crash) → eprintln the
       ;; cause (loud, terminal); ::Closed (the child exited before Started) → eprintln (terminal).
       _  (:wat::core::match (:wat::kernel::recv sp)
            ((:wat::kernel::RecvOutcome::Message _m) nil)
            ((:wat::kernel::RecvOutcome::Lost cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
            ;; arc 278 #73 — the substrate began stopping before the child reached
            ;; readiness. Terminal, but NOT the same fact as the two arms around it: no
            ;; crash (Lost) and no premature exit (Closed). The launch simply cannot
            ;; complete, and the message says so instead of blaming the child.
            (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "spawn (thread): stop requested before the child reached readiness — launch abandoned, the child was alive" :wat::core::None :wat::core::None))
            (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "spawn (thread): child exited before readiness" :wat::core::None :wat::core::None)))]
      (:wat::spawn::Launched :handle sp :address (:wat::spawn::Bound/address b)))))

;; Process (separate-memory) impl — assembles the child program from service-forms:
;; prepend `(def :user::spawn::service-locus (process))` (the transport literal lives HERE,
;; not in defservice), concat service-forms (which contains the agnostic child :user::main
;; that binds on :user::spawn::service-locus), spawn via spawn-program', handshake:
;;
;; The coordinate is `:user::`, not `:wat::` — and that is the doctrine, not a preference.
;; `:user::` is the RENDEZVOUS COORDINATE SPACE (see bracket.wat's header: "not a user's
;; namespace; a rendezvous space"), direction-agnostic: a name a parent PLANTS and a child
;; RESOLVES at startup. The exact precedent is `:user::bracket::work-fn`. And it must be
;; `:user::`: privilege does NOT survive a process boundary — by the time the child freezes
;; these forms they are the post-register_defines USER residue, so a `def` minting into the
;; RESERVED `:wat::` tree is what `resolve::gate -> Reserved` exists to refuse. It compiled
;; for as long as it did only because a scalar `def` never reached that gate.
;;   recv' the child-minted Address' (capability handoff — arc 272 6a)
;;   send' state0 to the child over the lineage (arc 272 6b-ii-α)
;; Returns Launched{handle=Process', address=child-minted Address'}.
;; The (process) literal lives ONLY here — the per-locus arm owns its transport.
(:wat::core::extend-type :wat::spawn::ProcessOpts :wat::spawn::Locus
  ;; arc 278 startup-crash parity: lu-mk-kw is accepted (surface arity) but UNUSED here — the
  ;; process child-main-form owns the Status::Started ctor. The handshake is REORDERED so :init
  ;; runs before Status::Started is sent: send' the ship (Admin::Init) DOWN first, THEN recv'
  ;; the Started UP. The child (child-main-form) now recvs ship → runs :init → sends Started, so
  ;; an :init crash dies BEFORE Started → the crash-aware `recv' svc` RAISES the child's reason
  ;; (the ProcessPanics envelope) instead of /start succeeding and the owner's later connect'
  ;; collapsing to a bare ECONNREFUSED with the reason discarded.
  (launch [self ship init serve service-forms lu-addr-kw lu-mk-kw]
    (:wat::core::let
      [prog (:wat::core::concat
              (:wat::core::forms
                (:wat::core::def :user::spawn::service-locus (:wat::spawn::process)))
              service-forms)
       svc  (:wat::kernel::spawn-program self prog)
       ;; arc 278 the send'-outcome wall — the crash-aware `recv' svc` right below faces
       ;; Closed/Lost on this handshake; the send' here just needs to proceed regardless.
       _    (:wat::core::match (:wat::kernel::send svc ship)
              (:wat::kernel::SendOutcome::Sent   nil)
              (:wat::kernel::SendOutcome::Closed nil)   ;; the recv' below already faces this
              ;; arc 278 #73 — same body, same precondition as the thread arm above: the
              ;; crash-aware `recv' svc` on the next line faces this handshake's terminal
              ;; outcomes, Stopped included. One decision point, not two.
              (:wat::kernel::SendOutcome::Stopped nil)
              ((:wat::kernel::SendOutcome::Lost _c) nil))
       ;; arc 278 the recv'-outcome wall — recv' returns a matchable RecvOutcome<Lu>. ::Message →
       ;; the child-minted launch status (extract-addr consumes it); ::Lost (the child crashed
       ;; before Started — the ProcessPanics envelope) → eprintln the cause (loud, terminal);
       ;; ::Closed (the child exited before Started) → eprintln (terminal).
       lu   (:wat::core::match (:wat::kernel::recv svc)
              ((:wat::kernel::RecvOutcome::Message m) m)
              ((:wat::kernel::RecvOutcome::Lost cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
              ;; arc 278 #73 — the process-tier twin of the thread arm above. Note this arm
              ;; was UNREACHABLE before today on this tier: `classify_peer_error`'s wildcard
              ;; folded the stop into Closed, so a stopped process launch blamed the child
              ;; for exiting. `spawn.rs` now carries `PeerDeath::Shutdown` and it arrives here.
              (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "spawn (process): stop requested before the child reached readiness — launch abandoned, the child was alive" :wat::core::None :wat::core::None))
              (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "spawn (process): child exited before readiness" :wat::core::None :wat::core::None)))
       addr (:wat::core::apply  lu-addr-kw lu [])]
      (:wat::spawn::Launched :handle svc :address addr))))

;; ── recv-all' — the honest peer-drain (arc 278 IPC de-prime) ─────────────────
;; Drains ALL output values from a spawned peer, honestly. The primed replacement
;; for the retired non-prime `:wat::test::run-hermetic-drain-outputs` (which returned
;; a bare `Vector<O>` and SWALLOWED the peer's death — `((:wat::core::Err _died) acc)`,
;; the exact swallow this prime fixes). Reads until the peer signals a terminal
;; RecvOutcome, matching the recv'-outcome wall exactly:
;;   Message[v]  -> accumulate v, continue.
;;   Closed      -> a GENUINE clean EOF; return (Ok <collected Vector<O>>).
;;   Lost[cause] -> the peer DIED; return (Err cause) — the LociDiedError rides in
;;                  the Err, surfaced, NEVER dropped (that is the whole point).
;; Composed from `recv'` (wat-first; recv' is native but recv-all' is a wat stdlib
;; defn). wat has no loop/recur, so the drain is a tail-recursive private helper
;; (`recv-all-loop'`) that recv-all' seeds with an empty vector. `p` is typed
;; `Peer'<I,O>`; Thread'/Process' derive Peer' (see the derives above), so a spawned
;; process/thread peer drains through here unchanged.
(:wat::core::defn :wat::kernel::recv-all-loop<I,O>
  [p   <- :wat::kernel::Peer<I,O>
   acc <- :wat::core::Vector<O>]
  -> :wat::core::Result<wat::core::Vector<O>,wat::kernel::LociDiedError>
  (:wat::core::match (:wat::kernel::recv p)
    ((:wat::kernel::RecvOutcome::Message v)
      (:wat::kernel::recv-all-loop p (:wat::core::conj acc v)))
    ((:wat::kernel::RecvOutcome::Lost cause)
      (:wat::core::Err cause))
    ;; arc 278 #73 — THE ARM THIS DRAIN EXISTS TO GET RIGHT. A stop cut the drain
    ;; short: the peer is ALIVE, more values may be pending, and `acc` is a PARTIAL
    ;; collection. Returning `(Ok acc)` here would be this fn's original sin restored —
    ;; it was written to replace a drain that SWALLOWED the peer's death and handed back
    ;; a bare Vector, and "I collected everything" over a truncated read is the same lie
    ;; in a different coat. So: NOT Ok. `Err` carries the fact by name.
    ;;
    ;; (Naming debt, stated not buried: the Err type is `LociDiedError` and nothing died.
    ;; That enum has already outgrown its name — it also carries StartupError, BadReturn
    ;; and MainSignature, none of them deaths. Renaming it is its own stone, not this one;
    ;; the VARIANT here is exact.)
    (:wat::kernel::RecvOutcome::Stopped
      (:wat::core::Err :wat::kernel::LociDiedError::Stopped))
    ;; the drain's SUCCESS path: a genuine clean EOF, everything collected.
    (:wat::kernel::RecvOutcome::Closed (:wat::core::Ok acc))))

(:wat::core::defn :wat::kernel::recv-all<I,O>
  [p <- :wat::kernel::Peer<I,O>]
  -> :wat::core::Result<wat::core::Vector<O>,wat::kernel::LociDiedError>
  (:wat::kernel::recv-all-loop p (:wat::core::Vector :O)))
