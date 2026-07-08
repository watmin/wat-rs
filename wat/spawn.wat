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
;;   #wat-edn.cap/address #wat.kernel/SocketAddressWire {:minter-pid 4242 :name [1 2 3 4 5]}
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
;; The init-fn runs at the peer's start and populates user.program.
;; ProcessOpts carries no config — its TYPE is the whole message.
;; Both opts records carry post-spawn-fn: an owner-side fn that runs after
;; the peer is spawned, before spawn-program' returns, for effects. Receives
;; the per-env launch record. Required with a no-op default on the bare ctors.
(:wat::core::defstruct :wat::spawn::ThreadOpts
  [init-fn       <- :wat::core::Fn()->wat::core::Record
   post-spawn-fn <- :wat::core::Fn(wat::spawn::ThreadLaunch)->wat::core::nil
   runner-count  <- :wat::core::i64])
(:wat::core::defstruct :wat::spawn::ProcessOpts
  [post-spawn-fn    <- :wat::core::Fn(wat::spawn::ProcessLaunch)->wat::core::nil
   env-fn           <- :wat::core::String
   max-message-bytes <- :wat::core::i64
   runner-count      <- :wat::core::i64])

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
    (:wat::core::fn [] -> :wat::core::Record (:wat::program::EmptyEnv))
    (:wat::core::fn [_l <- :wat::spawn::ThreadLaunch] -> :wat::core::nil nil)
    (:wat::program::cpu-count)))

(:wat::core::defn :wat::spawn::thread/init [f <- :wat::core::Fn()->wat::core::Record] -> :wat::spawn::ThreadOpts
  (:wat::spawn::ThreadOpts f
    (:wat::core::fn [_l <- :wat::spawn::ThreadLaunch] -> :wat::core::nil nil)
    (:wat::program::cpu-count)))

(:wat::core::defn :wat::spawn::thread/post-spawn [g <- :wat::core::Fn(wat::spawn::ThreadLaunch)->wat::core::nil] -> :wat::spawn::ThreadOpts
  (:wat::spawn::ThreadOpts
    (:wat::core::fn [] -> :wat::core::Record (:wat::program::EmptyEnv))
    g
    (:wat::program::cpu-count)))

(:wat::core::defn :wat::spawn::thread/runner-count [n <- :wat::core::i64] -> :wat::spawn::ThreadOpts
  (:wat::spawn::ThreadOpts
    (:wat::core::fn [] -> :wat::core::Record (:wat::program::EmptyEnv))
    (:wat::core::fn [_l <- :wat::spawn::ThreadLaunch] -> :wat::core::nil nil)
    n))

(:wat::core::defn :wat::spawn::process [] -> :wat::spawn::ProcessOpts
  (:wat::spawn::ProcessOpts
    (:wat::core::fn [_l <- :wat::spawn::ProcessLaunch] -> :wat::core::nil nil)
    "(:wat::program::EmptyEnv)"
    524288  ;; DEFAULT-MAX-MESSAGE-BYTES — mirrors src/edn_shim.rs DEFAULT_MAX_FRAME_BYTES
    (:wat::program::cpu-count)))

(:wat::core::defn :wat::spawn::process/post-spawn [f <- :wat::core::Fn(wat::spawn::ProcessLaunch)->wat::core::nil] -> :wat::spawn::ProcessOpts
  (:wat::spawn::ProcessOpts f "(:wat::program::EmptyEnv)" 524288 (:wat::program::cpu-count)))  ;; DEFAULT-MAX-MESSAGE-BYTES

(:wat::core::defn :wat::spawn::process/env [s <- :wat::core::String] -> :wat::spawn::ProcessOpts
  (:wat::spawn::ProcessOpts
    (:wat::core::fn [_l <- :wat::spawn::ProcessLaunch] -> :wat::core::nil nil)
    s
    524288  ;; DEFAULT-MAX-MESSAGE-BYTES
    (:wat::program::cpu-count)))

(:wat::core::defn :wat::spawn::process/max-message-bytes [n <- :wat::core::i64] -> :wat::spawn::ProcessOpts
  (:wat::spawn::ProcessOpts
    (:wat::core::fn [_l <- :wat::spawn::ProcessLaunch] -> :wat::core::nil nil)
    "(:wat::program::EmptyEnv)"
    n
    (:wat::program::cpu-count)))

(:wat::core::defn :wat::spawn::process/runner-count [n <- :wat::core::i64] -> :wat::spawn::ProcessOpts
  (:wat::spawn::ProcessOpts
    (:wat::core::fn [_l <- :wat::spawn::ProcessLaunch] -> :wat::core::nil nil)
    "(:wat::program::EmptyEnv)"
    524288  ;; DEFAULT-MAX-MESSAGE-BYTES
    n))

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
  :Connection [peer  <- :wat::kernel::Peer'<I,O>]
  :Message    [idx   <- :wat::core::i64  msg   <- :O]
  :Closed     [idx   <- :wat::core::i64]
  :Lost       [idx   <- :wat::core::i64  cause <- :wat::kernel::Failure])

;; ── Spawned — the owner-side spawn-handle marker ────────────────────────────
;; Spawned — the owner-side spawn-handle marker (typesub/derive axis; no methods). Thread'/Process'/
;; future-remote derive it so the locus-agnostic Handle field + Locus/spawn return can bind any of them.
;; Lifecycle = close'/join (intrinsics). A new transport's handle joins with one more `derive`.
(:wat::core::derive :wat::kernel::Thread'  :wat::spawn::Spawned)
(:wat::core::derive :wat::kernel::Process' :wat::spawn::Spawned)

;; ── arc 291 3a-ii-β: Thread'/Process' ARE Peer's ────────────────────────────
;; The owner-side spawn handle IS the parent end of the lineage channel — a peer.
;; send'/recv'/poll' already operate on it (process `launch` does `recv' svc`/`send' svc`);
;; these derives make the TYPE model say so, so a locus-agnostic `Handle.handle <- Peer'<…>`
;; field binds ANY spawn handle. N-LOCI-GENERAL: a future remote locus joins the peer family
;; with ONE more `derive` line — zero edits to the assignable rule, which is driven by THIS
;; derive graph (check.rs `assignable`, the Parametric<:Parametric arm). Never a 2-only assumption.
(:wat::core::derive :wat::kernel::Thread'  :wat::kernel::Peer')
(:wat::core::derive :wat::kernel::Process' :wat::kernel::Peer')

;; ── Bound<S,R> — the listening state minted by (listener' (thread) :S :R) ─────
;; A STRUCT, not a record: its fields are non-EDN RustOpaque kernel entities
;; (Listener'/Address'). `listener` is the server accept-side; `address` is what
;; clients dial via connect'. Replaces the bare Tuple the thread tier returned.
(:wat::core::defstruct :wat::spawn::Bound<S,R>
  [listener <- :wat::kernel::Listener'<S,R>
   address  <- :wat::kernel::Address'<S,R>])

;; ── Launched<S,R> — what Locus/launch returns: the spawn handle + the dial address ──
;; A STRUCT, not a record (address is an Address' RustOpaque; handle is :Spawned).
;; `handle` is the owner-side spawn handle (Thread'/Process'/future-remote all derive :Spawned).
;; `address` is what clients dial via connect'.
;; `start` unwraps Launched into the Handle record — locus-agnostic launch, locus-agnostic start.
;; arc 291 3a-ii-β: handle is the lineage PEER (Peer'<Sh,Lu> — sends Sh=Admin, recvs
;; Lu=LineageUp), no longer the opaque :Spawned marker. Thread'<Sh,Lu>/Process'<Sh,Lu>
;; bind it via the `derive …Peer'` foundation. This is what makes owner-only `stop` able
;; to send'/recv' on the Handle's handle. S,R = the client (listener/dial) channel.
(:wat::core::defstruct :wat::spawn::Launched<S,R,Sh,Lu>
  [handle  <- :wat::kernel::Peer'<Sh,Lu>
   address <- :wat::kernel::Address'<S,R>])

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
(:wat::core::defclause :wat::kernel::spawn-program'
  ;; thread — the ONE true form (self-peer; apply-loop is the annihilated heresy).
  ;; The locus's init-fn (extracted via ThreadOpts/init-fn) runs at the peer's start.
  ;; The locus's post-spawn-fn (extracted via ThreadOpts/post-spawn-fn) runs owner-side
  ;; after the peer is spawned, before spawn-program' returns, for effects.
  ;; Arc 293.W.2d: thread programs take ThreadSelfPeer' (in-locus, any I/O) as the self
  ;; parameter. Peer' is the wire-capable peer (pure I/O only); ThreadSelfPeer' is the
  ;; in-locus escape hatch for thread workers that carry Sender/Receiver or other impure types.
  ([locus <- :wat::spawn::ThreadOpts
    prog <- [:wat::kernel::ThreadSelfPeer'<S,R> :-> :wat::core::nil]] -> :wat::kernel::Thread'<R,S>
    (:wat::kernel::spawn-thread' prog (:wat::spawn::ThreadOpts/init-fn locus) (:wat::spawn::ThreadOpts/post-spawn-fn locus)))
  ;; process — forms (Vector<wat::WatAST>); I,O are the forms-server's free request/response vars.
  ;; The locus's post-spawn-fn (extracted via ProcessOpts/post-spawn-fn) runs owner-side
  ;; after the child is forked, with a ProcessLaunch{pid} carrying the child pid.
  ;; The locus's env-fn (extracted via ProcessOpts/env-fn) is a source string the child
  ;; evals in its own frozen world to produce user.program.
  ([locus <- :wat::spawn::ProcessOpts
    prog <- :wat::core::Vector<wat::WatAST>] -> :wat::kernel::Process'<I,O>
    (:wat::kernel::spawn-process' prog (:wat::spawn::ProcessOpts/post-spawn-fn locus) (:wat::spawn::ProcessOpts/env-fn locus) (:wat::spawn::ProcessOpts/max-message-bytes locus))))

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
;; serve's shape: (serve self-peer listener clients state) -> nil.
(:wat::core::defsurface :wat::spawn::Locus :nature :wat::core::Struct
  ;; arc 291 3a-ii-β: Lu = the lineage UP type (LineageUp); Sh = the ship/admin DOWN type.
  ;; The returned Launched carries the lineage peer as Peer'<Sh,Lu>.
  :features
  [(launch<S,R,St,Sh,Lu> [self          <- :wat::spawn::Locus
                          ship          <- :Sh
                          init          <- :wat::core::keyword
                          serve         <- :wat::core::keyword
                          service-forms <- :wat::core::Vector<wat::WatAST>
                          lu-addr-kw    <- :wat::core::keyword] -> :wat::spawn::Launched<S,R,Sh,Lu>)
   (spawn-runner<I,O> [self    <- :wat::spawn::Locus
                       work-fn <- :wat::core::Fn(I)->O]
     -> :wat::kernel::Peer'<(wat::core::i64,I),(wat::core::i64,O)>)])

;; Thread (shared-memory) impl — mints the listener internally via (listener' self :S :R)
;; (the method's type-params S,R flow as type-args — arc-232 dep proven GREEN).
;; Builds the serve closure capturing the minted listener + empty clients vector + state0;
;; spawn-program' (thread) runs it on a freshly-spawned peer. serve is invoked by keyword
;; via apply so this generic impl never names the per-service serve fn.
;; Returns Launched{handle=Thread', address=Bound/address}.
;; service-forms: thread arm ignores it (serve is already in the parent universe).
(:wat::core::extend-type :wat::spawn::ThreadOpts :wat::spawn::Locus
  (launch [self ship init serve service-forms lu-addr-kw]
    (:wat::core::let
      [b  (:wat::kernel::listener' self :S :R)
       sp (:wat::kernel::spawn-program' self
            (:wat::core::fn [self-peer <- :wat::kernel::ThreadSelfPeer'<Lu,Sh>] -> :wat::core::nil
              (:wat::core::apply -> :wat::core::nil serve self-peer
                (:wat::spawn::Bound/listener b)
                (:wat::core::Vector :wat::kernel::Peer'<R,S>)
                (:wat::core::apply -> :St init ship []) [])))]
      (:wat::spawn::Launched sp (:wat::spawn::Bound/address b)))))

;; Process (separate-memory) impl — assembles the child program from service-forms:
;; prepend `(def :wat::spawn::service-locus (process))` (the transport literal lives HERE,
;; not in defservice), concat service-forms (which contains the agnostic child :user::main
;; that binds on :wat::spawn::service-locus), spawn via spawn-program', handshake:
;;   recv' the child-minted Address' (capability handoff — arc 272 6a)
;;   send' state0 to the child over the lineage (arc 272 6b-ii-α)
;; Returns Launched{handle=Process', address=child-minted Address'}.
;; The (process) literal lives ONLY here — the per-locus arm owns its transport.
(:wat::core::extend-type :wat::spawn::ProcessOpts :wat::spawn::Locus
  (launch [self ship init serve service-forms lu-addr-kw]
    (:wat::core::let
      [prog (:wat::core::concat
              (:wat::core::forms
                (:wat::core::def :wat::spawn::service-locus (:wat::spawn::process)))
              service-forms)
       svc  (:wat::kernel::spawn-program' self prog)
       lu   (:wat::kernel::recv' svc)
       addr (:wat::core::apply -> :wat::kernel::Address'<S,R> lu-addr-kw lu [])
       _    (:wat::kernel::send' svc ship)]
      (:wat::spawn::Launched svc addr))))
