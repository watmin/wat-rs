;; wat/spawn.wat — the HOST opts for spawn-program (arc 259, The Forced Hand).
;;
;; The Keymaker.  (The Matrix Reloaded, 2003 — the little man in the Château who
;; cuts a different key for every door, and the right key is the only thing that
;; opens the backdoor.)  Each constructor below cuts exactly one key, for exactly
;; one hosting-door:
;;
;;   (thread)   — cuts a trivial key; the door is right here in this process.
;;   (process)  — cuts a trivial key; the door is a forked child universe.
;;
;; A host's TYPE is the whole message (where to host); spawn-program is a clause-set
;; that matches on the key's type and opens the matching door. Every new kind of
;; host that ever reveals itself is one new key + one new clause, the 2-arg
;; (spawn-program <host> <prog>) sig unmoved.
;;
;; ⛔ THE REMOTE DOOR IS PERPETUALLY AWAITING ITS KEY.  `:remote` is the forcing
;; function (like `spawn-program :remote` itself): we agree a remote host *must
;; materialize eventually* — and that whatever its opts record turns out to be, its
;; constructor's arity will be the lock (a remote host that cannot reach its host is
;; unrepresentable, the forced hand). But its STRUCT SHAPE IS NOT AGREED and must
;; NOT be guessed here — leaving the key uncut is the point. When the remote door's
;; lock is finally specified, `RemoteOpts` + its `(remote …)` constructor + a new
;; clause arrive together, the sig unmoved. Until then: deliberately absent.
;;
;; See docs/arc/2026/06/259-forced-hand/DESIGN.md § "The spawn primitive".
;; Loads AFTER wat/Record.wat (uses :wat::Record::def).

;; ── Per-env launch records (what each env hands the post-spawn hook) ─────────
;; ThreadLaunch is empty — no fields yet; grows if a need appears (don't build
;; the forcing function). ProcessLaunch carries the child pid, owner-side.
(:wat::Record::def :wat::spawn::ThreadLaunch [])
(:wat::Record::def :wat::spawn::ProcessLaunch [pid <- :wat::core::i64])

;; ── The keys (host opts records) ─────────────────────────────────────────────
;; ThreadOpts carries an init-fn: a 0-arg fn returning a :wat::Record.
;; The init-fn runs at the peer's start and populates user.program.
;; ProcessOpts carries no config — its TYPE is the whole message.
;; Both opts records carry post-spawn-fn: an owner-side fn that runs after
;; the peer is spawned, before spawn-program' returns, for effects. Receives
;; the per-env launch record. Required with a no-op default on the bare ctors.
(:wat::Record::def :wat::spawn::ThreadOpts
  [init-fn       <- :wat::core::Fn()->wat::Record
   post-spawn-fn <- :wat::core::Fn(wat::spawn::ThreadLaunch)->wat::core::nil])
(:wat::Record::def :wat::spawn::ProcessOpts
  [post-spawn-fn <- :wat::core::Fn(wat::spawn::ProcessLaunch)->wat::core::nil
   env-fn        <- :wat::core::String])

;; ── The Keymaker's friendly hand (ergonomic constructors) ────────────────────
;; (thread)             — default init-fn + no-op post-spawn-fn.
;; (thread/init f)      — init-fn is f; post-spawn-fn defaults to no-op.
;; (thread/post-spawn g)— init-fn defaults to EmptyEnv; post-spawn-fn is g.
;; (process)            — no-op post-spawn-fn; env-fn defaults to EmptyEnv ctor.
;; (process/post-spawn f)— post-spawn-fn is f; env-fn defaults to EmptyEnv ctor.
;; (process/env s)       — env-fn is s; post-spawn-fn defaults to no-op.
(:wat::core::defn :wat::spawn::thread [] -> :wat::spawn::ThreadOpts
  (:wat::spawn::ThreadOpts
    (:wat::core::fn [] -> :wat::Record (:wat::program::EmptyEnv))
    (:wat::core::fn [_l <- :wat::spawn::ThreadLaunch] -> :wat::core::nil nil)))

(:wat::core::defn :wat::spawn::thread/init [f <- :wat::core::Fn()->wat::Record] -> :wat::spawn::ThreadOpts
  (:wat::spawn::ThreadOpts f
    (:wat::core::fn [_l <- :wat::spawn::ThreadLaunch] -> :wat::core::nil nil)))

(:wat::core::defn :wat::spawn::thread/post-spawn [g <- :wat::core::Fn(wat::spawn::ThreadLaunch)->wat::core::nil] -> :wat::spawn::ThreadOpts
  (:wat::spawn::ThreadOpts
    (:wat::core::fn [] -> :wat::Record (:wat::program::EmptyEnv))
    g))

(:wat::core::defn :wat::spawn::process [] -> :wat::spawn::ProcessOpts
  (:wat::spawn::ProcessOpts
    (:wat::core::fn [_l <- :wat::spawn::ProcessLaunch] -> :wat::core::nil nil)
    "(:wat::program::EmptyEnv)"))

(:wat::core::defn :wat::spawn::process/post-spawn [f <- :wat::core::Fn(wat::spawn::ProcessLaunch)->wat::core::nil] -> :wat::spawn::ProcessOpts
  (:wat::spawn::ProcessOpts f "(:wat::program::EmptyEnv)"))

(:wat::core::defn :wat::spawn::process/env [s <- :wat::core::String] -> :wat::spawn::ProcessOpts
  (:wat::spawn::ProcessOpts
    (:wat::core::fn [_l <- :wat::spawn::ProcessLaunch] -> :wat::core::nil nil)
    s))

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
;;              O = the type the server RECEIVES from peers (peer's send type).
;; Mirror Peer'<I,O>: the accepted peer is Peer'<I,O>, message is O.
;;
(:wat::core::defenum :wat::kernel::ServiceEvent<I,O>
  :Shutdown                                                              ;; owner dropped the handle (self-peer drained) — exit; deadlock-free termination
  :Connection [peer  <- :wat::kernel::Peer'<I,O>]
  :Message    [idx   <- :wat::core::i64  msg   <- :O]
  :Closed     [idx   <- :wat::core::i64]
  :Lost       [idx   <- :wat::core::i64  cause <- :wat::kernel::Failure])

;; ── The Keymaker's masterwork (the spawn-program' defclause) ─────────────────
;;
;; Arc 259 S2c-ii-b — `spawn-program'` as a host-type defclause.
;;
;; 2-arg `(host prog)` — the key's TYPE (ThreadOpts | ProcessOpts) selects
;; the matching hosting door and delegates to the S2c-i tier primitives.
;; The env arg of the 3-arg intrinsic is gone (it was discarded at runtime;
;; the defclause makes the absence structural).
;;
;; Thread clause: prog MUST be the self-peer model `[Peer'<S,R>] -> nil`
;; (apply-loop purged by S2c-ii-a; the true form remains).
;; Process clause: prog is a `(:wat::core::forms ...)` block — a forms-server
;; program (`Vector<wat::WatAST>`) for the forked child universe.
;;
;; A new host type (e.g. RemoteOpts when its door is finally specified)
;; arrives as one new key + one new clause here; the 2-arg sig is unmoved.
(:wat::core::defclause :wat::kernel::spawn-program'
  ;; thread — the ONE true form (self-peer; apply-loop is the annihilated heresy).
  ;; The host's init-fn (extracted via ThreadOpts/init-fn) runs at the peer's start.
  ;; The host's post-spawn-fn (extracted via ThreadOpts/post-spawn-fn) runs owner-side
  ;; after the peer is spawned, before spawn-program' returns, for effects.
  ([host <- :wat::spawn::ThreadOpts
    prog <- [:wat::kernel::Peer'<S,R> :-> :wat::core::nil]] -> :wat::kernel::Thread'<R,S>
    (:wat::kernel::spawn-thread' prog (:wat::spawn::ThreadOpts/init-fn host) (:wat::spawn::ThreadOpts/post-spawn-fn host)))
  ;; process — forms (Vector<wat::WatAST>); I,O are the forms-server's free request/response vars.
  ;; The host's post-spawn-fn (extracted via ProcessOpts/post-spawn-fn) runs owner-side
  ;; after the child is forked, with a ProcessLaunch{pid} carrying the child pid.
  ;; The host's env-fn (extracted via ProcessOpts/env-fn) is a source string the child
  ;; evals in its own frozen world to produce user.program.
  ([host <- :wat::spawn::ProcessOpts
    prog <- :wat::core::Vector<wat::WatAST>] -> :wat::kernel::Process'<I,O>
    (:wat::kernel::spawn-process' prog (:wat::spawn::ProcessOpts/post-spawn-fn host) (:wat::spawn::ProcessOpts/env-fn host))))
