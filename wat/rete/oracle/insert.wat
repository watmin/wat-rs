;; wat/rete/oracle/insert.wat — interpreted insert / retract oracle.
;;
;; insert$oracle / insert-all$oracle / public insert / retract.
;; Zero activation: WM stays open until fire-rules. Loads after wat/rete.wat
;; (Session). Public names call $native.
;;
;; Namespace: :wat::rete::

;; ─── insert + retract ────────────────────────────────────────────────────────

;; insert-spec — the wat reference engine (the SPEC / differential oracle). Stages a fact into
;; the session's working memory. Zero activation.
;; WHY zero activation: the WM stays open while the caller stages multiple facts;
;; fire-rules is the lock that runs them through the network all at once.
;; WHY reconstruct Session: Record/assoc returns the base :wat::core::Record type; the
;; typed Session constructor preserves the concrete return type for the checker.
(:wat::core::defn :wat::rete::insert$oracle
  [session <- :wat::rete::Session
   fact    <- :wat::core::Record]
  -> :wat::rete::Session
  (:wat::rete::Session
    :network (:wat::rete::Session/network           session)
    :rules (:wat::rete::Session/rules             session)
    :alpha-memory (:wat::rete::Session/alpha-memory      session)
    :beta-memory (:wat::rete::Session/beta-memory       session)
    :production-memory (:wat::rete::Session/production-memory session)
    :facts (:wat::core::PersistentVector/conj (:wat::rete::Session/facts session) fact)
    :next-id (:wat::rete::Session/next-id           session)
    :query-memory (:wat::rete::Session/query-memory session)))

;; insert-all-spec — the wat reference engine (the SPEC / differential oracle) for BATCH insert.
;; Stages every fact in `facts` into the session's working memory: N chained insert-spec calls,
;; folded left→right so caller order is preserved. Zero activation — the exact insert-spec
;; contract, N times over (rete.wat:828-830 — WM stays open until fire-rules).
(:wat::core::defn :wat::rete::insert-all$oracle
  [session <- :wat::rete::Session
   facts   <- :wat::core::PersistentVector<wat::core::Record>]
  -> :wat::rete::Session
  (:wat::core::foldl
    :wat::rete::insert$oracle
    session
    facts))

;; insert-all — public batch verb. Keyword-head calls are intercepted by rust
;; (`insert-all`). This defn exists so `:wat::rete::insert-all` is a first-class Fn.
(:wat::core::defn :wat::rete::insert-all
  [session <- :wat::rete::Session
   facts   <- :wat::core::PersistentVector<wat::core::Record>]
  -> :wat::rete::Session
  (:wat::rete::insert-all$native session facts))

;; insert — public production verb. Runtime intercepts the keyword head
;; (`eval_insert_public`: 2-ary native, 3+ insert-all). This defclause is the
;; type surface and the first-class Fn; bodies re-enter the keyword head.
(:wat::core::defclause :wat::rete::insert
  ([session <- :wat::rete::Session
    fact    <- :T] -> :wat::rete::Session
    (:wat::rete::insert$native session fact))
  ([session <- :wat::rete::Session
    fact    <- :T
    & rest  <- :wat::core::Vector<wat::core::Record>] -> :wat::rete::Session
    (:wat::rete::insert-all session
      (:wat::core::foldl
        (:wat::core::fn [acc <- :wat::core::PersistentVector<wat::core::Record>
                         f   <- :T] -> :wat::core::PersistentVector<wat::core::Record>
          (:wat::core::PersistentVector/conj acc f))
        (:wat::core::PersistentVector/conj (:wat::core::PersistentVector) fact)
        rest))))

;; retract — stage a fact removal from Session.facts, by value equality. Zero activation.
;; Symmetric with insert: the caller re-fires (fire-rules recomputes from the reduced input).
;; WHY foldl + not-equals guard: mirrors merge-facts' foldl + contains? idiom; structural = on
;; records makes removal type-safe and value-precise (not identity/pointer removal).
;; WHY stage-only (no fire): same discipline as insert — the WM stays open for multiple staged
;; removals before the caller locks them in with fire-rules.
(:wat::core::defn :wat::rete::retract
  [session <- :wat::rete::Session
   fact    <- :wat::core::Record]
  -> :wat::rete::Session
  (:wat::core::let [old-facts (:wat::rete::Session/facts session)
                    new-facts (:wat::core::foldl
                                 (:wat::core::fn [acc <- :wat::core::PersistentVector<wat::core::Record>
                                                  f   <- :wat::core::Record]
                                   -> :wat::core::PersistentVector<wat::core::Record>
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

