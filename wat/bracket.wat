;; wat/bracket.wat — the brackets layer (Ruby's Parallel) built over spawn-program.
;;
;; This stone ships the runner server-loop — the multi-message peer that a
;; brackets pool stands on.  The pool coordinator + `brackets/map` come next.
;;
;; ── Design ───────────────────────────────────────────────────────────────────
;;
;; Today's spawn-program peers are single-shot: recv once → send once → return.
;; The brackets pool needs a peer that STREAMS: recv' item → work-fn → send'
;; result, looping until its channel drains.  The loop is a NAMED tail-recursive
;; defn so wat's TCO (arc 003 — apply_function replaces the top frame in place)
;; keeps the stack constant at any item count.
;;
;; Exit discipline: recv' raises (EvalBreak) when the parent's Thread' is
;; dropped → the runner's recursion is broken by the raise → it exits cleanly.
;; No explicit termination condition is needed; the channel drain IS the signal.
;;
;; Loads AFTER wat/spawn.wat (uses :wat::kernel::Peer', recv', send').
;;
;; ── Rendezvous convention ───────────────────────────────────────────────────
;;
;; `:user::` is the RENDEZVOUS NAMESPACE — the known-location coordinates where
;; a program exposes what a substrate consumer looks up.  Not private/internal
;; space; a rendezvous space.  `:user::main` is wat-program's coordinate (the
;; kernel-required entry, `[] -> :nil`).  Bracket installs a second one:
;; `:user::bracket::work-fn` — the work function a process-pool child's
;; baked runner (`:wat::bracket::process-runner<I,O>` below) applies.  The
;; runner itself is baked/reserved (never shipped); the child's user.program
;; only ever ships the user's own work-fn, reified at this coordinate, plus a
;; generated `:user::main` that passes the coordinate's value into the runner.

(:wat::core::defn :wat::bracket::runner-loop<I,O>
  [self    <- :wat::kernel::ThreadSelfPeer'<O,I>
   work-fn <- :wat::core::Fn(I)->O]
  -> :wat::core::nil
  (:wat::core::let [item (:wat::kernel::recv' self)
                    _    (:wat::kernel::send' self (work-fn item))]
    (:wat::bracket::runner-loop self work-fn)))

;; ── process-runner — the BAKED, reserved process-pool runner (259 S3c) ───────
;;
;; Generic index-wrapping runner for the process (not-shared) locus tier: recv
;; (idx,I) → work-fn item → send (idx,O), tail-recursing forever.  Established
;; in the child's phase-one stdlib load — privileged, reserved, zero user
;; input.  A user can never allocate it (`:wat::` is undefinable anywhere) and
;; it is never shipped, so nothing can collide with it.  The work-fn is taken
;; as a VALUE (not referenced by name) so the runner stays generic/baked with
;; no stdlib -> user.program forward reference; the process arm's spawn-runner
;; ships only the work-fn (at the :user::bracket::work-fn rendezvous
;; coordinate) and a generated :user::main that passes it in here.
(:wat::core::defn :wat::bracket::process-runner<I,O>
  [self    <- :wat::kernel::Peer'<(wat::core::i64,O),(wat::core::i64,I)>
   work-fn <- :wat::core::Fn(I)->O]
  -> :wat::core::nil
  (:wat::core::let
    [pair (:wat::kernel::recv' self)
     out  (:wat::core::Tuple (:wat::core::first pair) (work-fn (:wat::core::second pair)))
     _    (:wat::kernel::send' self out)]
    (:wat::bracket::process-runner self work-fn)))

;; ── spawn-runner — the per-tier runner spawn, lifted onto the :Locus surface ──
;;
;; The bracket coordinator (map-worker) is now loci-agnostic: it holds an abstract
;; :wat::spawn::Locus and calls (spawn-runner locus work-fn) once per pool tier.
;; The RAW work fn Fn(I)->O is passed — NOT an index-wrapping closure. Each tier
;; does its own index-wrapping over the raw fn:
;;
;;   THREAD (shared memory): build the (idx,I)->(idx,O) wrapper inline as a thread
;;   closure (captures work-fn freely — no reification), run runner-loop on it.
;;
;;   PROCESS (not-shared): fn-forms the RAW work-fn (top-level, no captured fn —
;;   the one shape fn-forms/closure_extract slice-1 CAN reify) into :__pool-work,
;;   then ship a NAMED index-wrapping pool-runner as source (the defservice fork
;;   trick). Mirrors scratchpad/probe-s3-process-runner.wat.
;;
;; Both return :wat::kernel::Peer'<(i64,I),(i64,O)> so collect-loop drains a
;; uniform Vector<Peer'<…>> (select' accepts Peer' as of S3a).

(:wat::core::extend-type :wat::spawn::ThreadOpts :wat::spawn::Locus
  (spawn-runner [self work-fn]
    (:wat::kernel::spawn-program' self
      (:wat::core::fn [sp <- :wat::kernel::ThreadSelfPeer'<(wat::core::i64,O),(wat::core::i64,I)>] -> :wat::core::nil
        (:wat::bracket::runner-loop sp
          (:wat::core::fn [pair <- :(wat::core::i64,I)] -> :(wat::core::i64,O)
            (:wat::core::Tuple (:wat::core::first pair) (work-fn (:wat::core::second pair)))))))))

;; The PROCESS arm (not-shared) — bakes the runner, ships only the user's code
;; (259 S3c; supersedes the S3b shipped-runner shape).
;;
;; The runner is BAKED (`:wat::bracket::process-runner<I,O>` above) — nothing
;; reserved is shipped, so the `ReservedPrefix` problem S3b fought (an
;; un-squattable shipped name has nowhere safe to live in `:wat::`) is simply
;; gone.  We ship only: the user's work-fn, reified at the rendezvous
;; coordinate `:user::bracket::work-fn` (fn-forms), plus a generated
;; `:user::main` that calls the baked runner, passing that coordinate's VALUE
;; in (the runner is baked, so `:user::main` passes the value — it cannot look
;; the coordinate up from stdlib; that would be a stdlib -> user.program
;; forward reference the resolver rejects).
;;
;; `:user::main`'s `self-peer` call still needs CONCRETE peer types — a
;; generic runtime method can't monomorphize spawn-runner's `:I`/`:O`
;; type-params into shipped `forms` (they'd land literal and unbound in the
;; child universe). So we DERIVE the concrete arg/return types off the
;; reified work-fn: `fn-forms` emits a `(def :user::bracket::work-fn (fn [n <-
;; :ArgT] -> :RetT …))` whose ArgT/RetT are literal AST nodes. We AST-walk
;; them out (def → fn → argspec[after <-] + [after ->]), build the concrete
;; tuple-type keywords via `keyword-node`, and splice them into the shipped
;; `self-peer` tuple types via quasiquote.  The generic baked runner itself
;; needs no concrete types (it monomorphizes at the call).
;;
;; The fn-forms bind-name is a COMPUTED keyword (not a source literal): a
;; literal `:user::bracket::work-fn` here would, when the child re-typechecks
;; THIS file with that name shipped-as-a-def, resolve to the shipped fn (a Fn,
;; not a keyword) and fail fn-forms' `name` param. A computed keyword is
;; unresolvable at check → safe.
(:wat::core::extend-type :wat::spawn::ProcessOpts :wat::spawn::Locus
  (spawn-runner [self work-fn]
    (:wat::core::let
      [work-name (:wat::core::keyword/from-string "user::bracket::work-fn")
       forms     (:wat::kernel::fn-forms work-fn work-name)
       ;; ── derive the concrete arg/return type keywords off the reified work-fn ──
       def-node  (:wat::core::Option/expect (:wat::core::last forms) "spawn-runner: fn-forms produced no define")
       fn-form   (:wat::core::first (:wat::core::drop (:wat::core::ast->children def-node) 2))
       fn-ch     (:wat::core::ast->children fn-form)
       argspec   (:wat::core::first (:wat::core::drop fn-ch 1))
       arg-ty    (:wat::core::Option/expect (:wat::core::last (:wat::core::ast->children argspec)) "spawn-runner: work-fn has no arg type")
       ret-ty    (:wat::core::first (:wat::core::drop fn-ch 3))
       ;; ast-name → ":wat::core::i64"; strip the leading ':' for the tuple bodies.
       arg-nm    (:wat::core::ast-name arg-ty)
       ret-nm    (:wat::core::ast-name ret-ty)
       arg-t     (:wat::core::string::subs arg-nm 1 (:wat::core::string::length arg-nm))
       ret-t     (:wat::core::string::subs ret-nm 1 (:wat::core::string::length ret-nm))
       ;; ── build the CONCRETE self-peer tuple-type keyword nodes (I=arg, O=ret) ──
       ;; self-peer :(i64,O) :(i64,I) — output tuple first, input tuple second.
       sp-out    (:wat::core::keyword-node
                   (:wat::core::string::concat ":(wat::core::i64,"
                     (:wat::core::string::concat ret-t ")")))
       sp-in     (:wat::core::keyword-node
                   (:wat::core::string::concat ":(wat::core::i64,"
                     (:wat::core::string::concat arg-t ")")))
       ;; ── generated :user::main — passes the rendezvous work-fn VALUE into the baked runner ──
       main-def  `(:wat::core::defn :user::main [] -> :wat::core::nil
                    (:wat::bracket::process-runner
                      (:wat::program::self-peer ~sp-out ~sp-in)
                      :user::bracket::work-fn))]
      (:wat::kernel::spawn-program' self
        (:wat::core::concat forms (:wat::core::Vector :wat::WatAST main-def))))))

;; ── collect-loop — tail-recursive collector; drains M results from N runners ──
;;
;; State: peers (the live Thread' vector), items (the full input vector),
;; pairs-acc (accumulator of (idx,result) pairs so far), cursor (next item
;; to dispatch), collected (how many results have arrived), m (total item count).
;;
;; Invariant: cursor ≤ m; collected ≤ m.  When collected == m every result
;; has arrived; return pairs-acc (unsorted — the caller sorts).
;;
;; Dynamic balance: after select' returns the ServiceEvent::Message{idx=peer-pos, msg=pair}
;; for whichever runner finished first, that runner's channel is empty again
;; and we immediately feed it the next pending item (if cursor < m).  Runners
;; that had no item sent to them (when M < N) are simply never select'ed —
;; the channel-drain RAII at scope exit joins them cleanly.
;;
;; select' now returns ServiceEvent<I,O> (Stone 259 Lost-locus).  :Message is
;; the normal case.  :Closed/:Lost are honest arms — a bracket runner should
;; never disconnect or crash in normal operation; if it does, raise via
;; assertion-failed! so the failure is visible rather than silently swallowed.

(:wat::core::defn :wat::bracket::collect-loop<I,O>
  [peers     <- :wat::core::Vector<wat::kernel::Peer'<(wat::core::i64,I),(wat::core::i64,O)>>
   items     <- :wat::core::Vector<I>
   pairs-acc <- :wat::core::Vector<(wat::core::i64,O)>
   cursor    <- :wat::core::i64
   collected <- :wat::core::i64
   m         <- :wat::core::i64]
  -> :wat::core::Vector<(wat::core::i64,O)>
  (:wat::core::if (:wat::core::= collected m)
    pairs-acc
    (:wat::core::let
      [event    (:wat::kernel::select' peers)]
      (:wat::core::match event
        -> :wat::core::Vector<(wat::core::i64,O)>
        ((:wat::spawn::ServiceEvent::Message peer-pos pair)
          (:wat::core::let
            [cursor'  (:wat::core::if (:wat::core::< cursor m)
                        (:wat::core::let [_ (:wat::kernel::send'
                                              (:wat::core::nth peers peer-pos)
                                              (:wat::core::Tuple cursor (:wat::core::nth items cursor)))]
                          (:wat::core::+ cursor 1))
                        cursor)]
            (:wat::bracket::collect-loop peers items
              (:wat::core::conj pairs-acc pair) cursor' (:wat::core::+ collected 1) m)))
        ((:wat::spawn::ServiceEvent::Closed idx)
          (:wat::kernel::assertion-failed!
            (:wat::core::string::interpolate
              "bracket collect-loop: runner {idx} closed unexpectedly"
              :idx idx)
            :wat::core::None :wat::core::None))
        ((:wat::spawn::ServiceEvent::Lost idx cause)
          (:wat::kernel::assertion-failed!
            (:wat::core::string::interpolate
              "bracket collect-loop: runner {idx} crashed: {cause}"
              :idx idx :cause (:wat::kernel::Failure/message cause))
            :wat::core::None :wat::core::None))
        (:wat::spawn::ServiceEvent::Shutdown
          (:wat::kernel::assertion-failed!
            "bracket collect-loop: unexpected Shutdown event"
            :wat::core::None :wat::core::None))
        ((:wat::spawn::ServiceEvent::Connection _peer)
          (:wat::kernel::assertion-failed!
            "bracket collect-loop: unexpected Connection event"
            :wat::core::None :wat::core::None))
        ((:wat::spawn::ServiceEvent::Admin _msg)
          (:wat::kernel::assertion-failed!
            "bracket collect-loop: unexpected Admin event (select' has no self-peer)"
            :wat::core::None :wat::core::None))))))

;; ── map-worker — general pool engine (per-runner state via worker-init) ───────
;;
;; Each runner i is built from `(worker-init i)`: the OUTER call is per-runner
;; setup (once, when the runner is built — the place to allocate a resource
;; reused across that runner's items); the INNER result is the per-item work-fn.
;; `worker-id` is the runner index passed to `worker-init`.  The coordinator
;; (spawn+prime+collect+sort) lives here ONCE; `map` and `each` are thin wrappers.

(:wat::core::defn :wat::bracket::map-worker<I,O>
  [locus       <- :wat::spawn::Locus
   items       <- :wat::core::Vector<I>
   worker-init <- :wat::core::Fn(wat::core::i64)->wat::core::Fn(I)->O]
  -> :wat::core::Vector<O>
  (:wat::core::let
    [m  (:wat::core::length items)
     rc (:wat::spawn::runner-count locus)
     n  (:wat::core::if (:wat::core::< rc m) rc m)
     ;; Arc 170 capability circuit, stone 2 — the Grantables this locus carries. Empty for
     ;; thread/remote (the firm boundary); the process locus's :grants field otherwise. Read
     ;; ONCE; grant-boot below folds over it before each worker's first item, revoke-shutdown
     ;; folds over it after the drain. A foldl over an empty vector is a no-op, so a plain
     ;; (process) (no :grants) takes no grant path — same as thread.
     grantables (:wat::spawn::grants locus)
     ;; Arc 118.2a — `map` flipped LAZY; `peers` feeds `collect-loop` (Vector<Peer'<...>> param
     ;; — repeatedly `select'`-ed, must be eager) and later `sort-by`, so materialize here.
     peers (:wat::core::mapv
             (:wat::core::fn [i <- :wat::core::i64]
                 -> :wat::kernel::Peer'<(wat::core::i64,I),(wat::core::i64,O)>
               (:wat::core::let
                 [work-fn (worker-init i)                          ;; per-runner setup, once
                  p (:wat::spawn::Locus/spawn-runner locus work-fn)
                  ;; GRANT-BOOT: if the far end is a process (peer-pid → Some pid), grant that
                  ;; kernel-vouched pid to each Grantable (ack'd request/reply) BEFORE the first
                  ;; item is sent — so the grant lands before the worker's work-fn dials. A
                  ;; thread peer (peer-pid → None) skips: the in-process handle IS the capability.
                  _ (:wat::core::match (:wat::kernel::peer-pid p) -> :wat::core::nil
                      ((:wat::core::Some pid)
                        (:wat::core::foldl
                          (:wat::core::fn [_acc <- :wat::core::nil  g <- :wat::capability::Grantable] -> :wat::core::nil
                            (:wat::capability::Grantable/grant g (:wat::core::Vector :wat::core::i64 pid)))
                          nil
                          grantables))
                      (:wat::core::None nil))
                  _ (:wat::kernel::send' p (:wat::core::Tuple i (:wat::core::nth items i)))]
                 p))
             (:wat::core::range 0 n))
     pairs  (:wat::bracket::collect-loop peers items
              (:wat::core::Vector :(wat::core::i64,O)) n 0 m)
     ;; REVOKE-SHUTDOWN: the drain is complete but the peers are still alive (still in scope,
     ;; still hold their Pidfd → peer-pid still Some). For each process peer, revoke its pid
     ;; from each Grantable (ack'd) — the grant a worker held cannot outlive its reaping. A
     ;; thread peer (None) skips. Runs BEFORE the return so no grant escapes the bracket.
     _revoke (:wat::core::foldl
               (:wat::core::fn [_acc <- :wat::core::nil
                                p    <- :wat::kernel::Peer'<(wat::core::i64,I),(wat::core::i64,O)>]
                 -> :wat::core::nil
                 (:wat::core::match (:wat::kernel::peer-pid p) -> :wat::core::nil
                   ((:wat::core::Some pid)
                     (:wat::core::foldl
                       (:wat::core::fn [_a <- :wat::core::nil  g <- :wat::capability::Grantable] -> :wat::core::nil
                         (:wat::capability::Grantable/revoke g (:wat::core::Vector :wat::core::i64 pid)))
                       nil
                       grantables))
                   (:wat::core::None nil)))
               nil
               peers)
     sorted (:wat::core::sort-by
              (:wat::core::fn [pr <- :(wat::core::i64,O)] -> :wat::core::i64
                (:wat::core::first pr))
              pairs)]
    ;; Arc 118.2a — `map` flipped LAZY; the function's declared return type is `Vector<O>`.
    (:wat::core::mapv
      (:wat::core::fn [pr <- :(wat::core::i64,O)] -> :O
        (:wat::core::second pr))
      sorted)))

;; ── map — thin wrapper over map-worker (Ruby's Parallel.map) ─────────────────
;;
;; Passes a constant `worker-init` that ignores the runner id and returns the
;; shared work-fn.  The coordinator (spawn+prime+collect+sort) lives in map-worker.

(:wat::core::defn :wat::bracket::map<I,O>
  [locus   <- :wat::spawn::Locus
   items   <- :wat::core::Vector<I>
   work-fn <- :wat::core::Fn(I)->O]
  -> :wat::core::Vector<O>
  (:wat::bracket::map-worker locus items
    (:wat::core::fn [_worker-id <- :wat::core::i64] -> :wat::core::Fn(I)->O
      work-fn)))

;; ── each-worker — general side-effect pool (per-runner state via worker-init) ─
;;
;; `map-worker` that DISCARDS: run worker-init-derived per-item fns over every
;; item through the pool, then return nil.

(:wat::core::defn :wat::bracket::each-worker<I,O>
  [locus       <- :wat::spawn::Locus
   items       <- :wat::core::Vector<I>
   worker-init <- :wat::core::Fn(wat::core::i64)->wat::core::Fn(I)->O]
  -> :wat::core::nil
  (:wat::core::do (:wat::bracket::map-worker locus items worker-init) nil))

;; ── each — thin wrapper over each-worker (Ruby's Parallel.each) ──────────────
;;
;; Passes a constant `worker-init` that ignores the runner id.

(:wat::core::defn :wat::bracket::each<I,O>
  [locus   <- :wat::spawn::Locus
   items   <- :wat::core::Vector<I>
   work-fn <- :wat::core::Fn(I)->O]
  -> :wat::core::nil
  (:wat::bracket::each-worker locus items
    (:wat::core::fn [_worker-id <- :wat::core::i64] -> :wat::core::Fn(I)->O
      work-fn)))
