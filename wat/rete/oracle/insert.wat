;; wat/rete/oracle/insert.wat — interpreted insert / retract oracle.
;;
;; insert$oracle / insert-all$oracle / public insert / insert-all.
;; `retract` is a wat-only Session rebuild (no $oracle / $native pair yet).
;; Zero activation: facts stay staged until fire-rules. Loads after wat/rete.wat
;; (Session). insert / insert-all public names call $native.
;;
;; Namespace: :wat::rete::

;; ─── insert + retract ────────────────────────────────────────────────────────

;; insert$oracle — the wat reference engine (the SPEC / differential oracle). Stages a fact into
;; the session's `facts` vector. Zero activation.
;; WHY zero activation: facts stay staged while the caller inserts multiple facts;
;; fire-rules is the lock that runs them through the network all at once.
;; WHY reconstruct Session: Record/assoc returns the base :wat::core::Record type; the
;; typed Session constructor preserves the concrete return type for the checker.
;; ⛔ SAME TYPE AS THE NATIVE by the dual-impl contract. The oracle enforces no ceiling, so it can
;; only ever answer `Inserted` — the standing accepted asymmetry ("the $oracle is the reference an
;; embedder never runs"); answering a bare Session would make a differential harness unwrap one
;; side and not the other, i.e. compare two different things.
(:wat::core::defn :wat::rete::insert$oracle
  [session <- :wat::rete::Session
   fact    <- :wat::core::Record]
  -> :wat::rete::InsertOutcome
  (:wat::rete::InsertOutcome::Inserted
   (:wat::rete::Session
    :network (:wat::rete::Session/network           session)
    :rules (:wat::rete::Session/rules             session)
    :alpha-memory (:wat::rete::Session/alpha-memory      session)
    :beta-memory (:wat::rete::Session/beta-memory       session)
    :production-memory (:wat::rete::Session/production-memory session)
    :facts (:wat::core::PersistentVector/conj (:wat::rete::Session/facts session) fact)
    :next-id (:wat::rete::Session/next-id           session)
    :query-memory (:wat::rete::Session/query-memory session))))

;; insert-all$oracle — the wat reference engine (the SPEC / differential oracle) for BATCH insert.
;; Stages every fact in `facts`: N chained insert$oracle calls, folded left→right so caller
;; order is preserved. Zero activation — the exact insert$oracle contract, N times over
;; (facts stay staged until fire-rules).
;; ⛔ THE FOLD STEP NOW UNWRAPS. `insert$oracle` answers an `InsertOutcome`, so it can no longer be
;; handed to `foldl` bare — the accumulator is a Session and the step yields an outcome. The
;; ceiling arm is UNREACHABLE here (the oracle enforces none) and says so loudly rather than being
;; swallowed: if it ever fires, the oracle has grown a ceiling and this comment is what was wrong.
(:wat::core::defn :wat::rete::insert-all$oracle
  [session <- :wat::rete::Session
   facts   <- (:wat::core::PersistentVector :- [:wat::core::Record])]
  -> :wat::rete::InsertOutcome
  (:wat::rete::InsertOutcome::Inserted
    (:wat::core::foldl
      (:wat::core::fn [acc <- :wat::rete::Session  f <- :wat::core::Record] -> :wat::rete::Session
        (:wat::core::match (:wat::rete::insert$oracle acc f)
          ((:wat::rete::InsertOutcome::Inserted __s) __s)
          ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __l __u __st)
            (:wat::kernel::assertion-failed!
              "insert-all$oracle: session memory ceiling — the oracle enforces none"
              :wat::core::None :wat::core::None))))
      session
      facts)))

;; insert-all — public batch verb. Keyword-head calls are intercepted by rust
;; (`insert-all`). This defn exists so `:wat::rete::insert-all` is a first-class Fn.
;; ⛔ RETURNS A MATCHABLE `(:wat::rete::InsertOutcome)` — arc 278 the fire-outcome wall, S2c.
;; STAGING IS NOT FREE: `insert` holds every fact until a fire consumes it, so a fold that inserts
;; without firing grows the session exactly as derivation does (measured: 2.5M staged facts reached
;; 4.0 GB against a 1 GiB contract). The session ceiling is enforced at BOTH doors, and a breach it
;; cannot prove at load is a VALUE the caller must handle, never a raise.
(:wat::core::defn :wat::rete::insert-all
  [session <- :wat::rete::Session
   facts   <- (:wat::core::PersistentVector :- [:wat::core::Record])]
  -> :wat::rete::InsertOutcome
  (:wat::rete::insert-all$native session facts))

;; insert — public production verb. Runtime intercepts the keyword head
;; (`eval_insert_public`: 2-ary native, 3+ insert-all). This defclause is the
;; type surface and the first-class Fn; bodies call `$native`.
;; ⛔ BOTH ARITIES ANSWER `(:wat::rete::InsertOutcome)` — arc 278, S2c. The 3+ arity is sugar over
;; `insert-all`, so it returns the outcome unchanged: a pass-through, nothing to unwrap here.
(:wat::core::defclause :wat::rete::insert
  ([session <- :wat::rete::Session
    fact    <- :T] -> :wat::rete::InsertOutcome
    (:wat::rete::insert$native session fact))
  ([session <- :wat::rete::Session
    fact    <- :T
    & rest  <- (:wat::core::Vector :- [:wat::core::Record])] -> :wat::rete::InsertOutcome
    (:wat::rete::insert-all session
      (:wat::core::foldl
        (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::core::Record])
                         f   <- :T] -> (:wat::core::PersistentVector :- [:wat::core::Record])
          (:wat::core::PersistentVector/conj acc f))
        (:wat::core::PersistentVector/conj (:wat::core::PersistentVector) fact)
        rest))))

;; retract — stage a fact removal from Session.facts, by value equality. Zero activation.
;; Symmetric with insert: the caller re-fires (fire-rules recomputes from the reduced input).
;; WHY foldl + not-equals guard: mirrors merge-facts' foldl + contains? idiom; structural = on
;; records makes removal type-safe and value-precise (not identity/pointer removal).
;; WHY stage-only (no fire): same discipline as insert — facts stay staged for multiple
;; removals before the caller locks them in with fire-rules.
(:wat::core::defn :wat::rete::retract
  [session <- :wat::rete::Session
   fact    <- :wat::core::Record]
  -> :wat::rete::Session
  (:wat::core::let [old-facts (:wat::rete::Session/facts session)
                    new-facts (:wat::core::foldl
                                 (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::core::Record])
                                                  f   <- :wat::core::Record]
                                   -> (:wat::core::PersistentVector :- [:wat::core::Record])
                                   (:wat::core::if (:wat::core::not (:wat::core::= f fact))
                                     (:wat::core::PersistentVector/conj acc f)
                                     acc))
                                 (:wat::core::PersistentVector)
                                 old-facts)]
    (:wat::rete::Session
      :network (:wat::rete::Session/network           session)
      :rules (:wat::rete::Session/rules             session)
      :alpha-memory (:wat::rete::Session/alpha-memory      session)
      :beta-memory (:wat::rete::Session/beta-memory       session)
      :production-memory (:wat::rete::Session/production-memory session)
      :facts new-facts
      :next-id (:wat::rete::Session/next-id           session)
      :query-memory (:wat::rete::Session/query-memory session))))

