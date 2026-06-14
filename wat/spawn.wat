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

;; ── The keys (host opts records) ─────────────────────────────────────────────
;; ThreadOpts carries an init-fn: a 0-arg fn returning a :wat::Record.
;; The init-fn runs at the peer's start and populates user.program.
;; ProcessOpts carries no config — its TYPE is the whole message.
(:wat::Record::def :wat::spawn::ThreadOpts
  [init-fn <- :wat::core::Fn()->wat::Record])
(:wat::Record::def :wat::spawn::ProcessOpts [])

;; ── The Keymaker's friendly hand (ergonomic constructors) ────────────────────
;; (thread)        — default init-fn is the EmptyEnv thunk.
;; (thread/init f) — init-fn is f, a 0-arg fn returning a :wat::Record.
(:wat::core::defn :wat::spawn::thread [] -> :wat::spawn::ThreadOpts
  (:wat::spawn::ThreadOpts (:wat::core::fn [] -> :wat::Record (:wat::program::EmptyEnv))))

(:wat::core::defn :wat::spawn::thread/init [f <- :wat::core::Fn()->wat::Record] -> :wat::spawn::ThreadOpts
  (:wat::spawn::ThreadOpts f))

(:wat::core::defn :wat::spawn::process [] -> :wat::spawn::ProcessOpts
  (:wat::spawn::ProcessOpts))

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
  ([host <- :wat::spawn::ThreadOpts
    prog <- [:wat::kernel::Peer'<S,R> :-> :wat::core::nil]] -> :wat::kernel::Thread'<R,S>
    (:wat::kernel::spawn-thread' prog (:wat::spawn::ThreadOpts/init-fn host)))
  ;; process — forms (Vector<wat::WatAST>); I,O are the forms-server's free request/response vars.
  ([host <- :wat::spawn::ProcessOpts
    prog <- :wat::core::Vector<wat::WatAST>] -> :wat::kernel::Process'<I,O>
    (:wat::kernel::spawn-process' prog)))
