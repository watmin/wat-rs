;; wat-scripts/perf/grid/retract-lead-accum.wat — COMPOUND CORRECTNESS AXIS: cell
;; (record, present, base, leading, na) of the peragrare grid — cell #2 of
;; docs/arc/2026/06/278-rules-engine/strike-grid-remaining-compound-cells/DESIGN.md.
;;
;; A duplicate-retract (retract-multiplicity's `R` mechanism — remove-one, not remove-all)
;; feeding a LEADING accumulate (accum-lead-rule-cascade's `L` mechanism — a parentless
;; accumulate, then a join, plus an inert cascade forcing extra fixpoint rounds). THIS AXIS
;; IS THEIR INTERSECTION — read both first:
;;
;;   retract-multiplicity     (record, present, none, none,    na) — F(0)×2 feeds a plain
;;                            two-condition JOIN (F AND G); the retract's target has a LEFT
;;                            token (G) to anchor against.
;;   accum-lead-rule-cascade  (record, absent,  base, leading, na) — Busy(k,n) :- [?n <-
;;                            acc::count :from Reading] AND Anchor(k); the accumulate has NO
;;                            parent condition, but Reading is never retracted.
;;   THIS AXIS                (record, present, base, leading, na) — accum-lead-rule-
;;                            cascade's EXACT shape, but Reading(0) is duplicated (like
;;                            retract-multiplicity's F(0)), then ONE copy is retracted and
;;                            the session re-fired.
;;
;; Reading(v) for v in [0,items), PLUS one extra Reading(0);  Anchor(k) for k in [0,anchors);
;; Link(0) seeded;  Link(k) :- Link(k-1) for k in [1,depth] — INERT cascade, read by NOTHING;
;; Busy(k,n) :- [?n <- (acc::count) :from Reading] AND Anchor(k)  ;; LEADING — no anchor token
;; fire;  retract Reading(0) ONCE (remove-one);  re-fire.
;;
;; WHY THIS IS THE RISK, not a formality: retract-multiplicity's remove-one cure has only
;; ever been driven where the retracted type feeds a JOIN with a LEFT token to anchor
;; against (G, in that fixture). A LEADING accumulate has NO parent condition — nothing
;; anchors which token the retract's supersession targets against. If remove-one's
;; bookkeeping assumes (even implicitly) a left-token frame that a leading node does not
;; have, this is where that would show: either BOTH Reading(0) copies surviving the retract
;; (remove-ALL when remove-ONE was asked), or NEITHER surviving (over-retraction), or the
;; leading accumulate's own re-emission-per-round defect (accum-lead-rule-cascade's own
;; failure shape) resurfacing now that its input population is churning under a retract.
;;
;; ⛔ THE CASCADE IS INERT, same as accum-lead-rule-cascade's — Link is derived and never
;; read by Busy or the query. Any spread in :derived across depth (not tested here as a
;; sweep, but the shape is retained so the round-independence property stays load-bearing
;; through the retract/re-fire) is the engine leaking its round count into an answer.
;;
;; ⛔ ONE CORRECTNESS SIZE, not a sweep. No `gen-retract-lead-accum.sh` — a generator would
;; drag a correctness proof onto the perf ladder (`run-all.sh:81-87`, `check-grid-speed.sh`
;; treats `:accuracy :MISMATCH` as a gate failure). Static `retract-lead-accum.clj`.
;;
;; Size vector is [items anchors depth]. Stated choice: [3 2 3] — same as accum-lead-rule-
;; cascade's, for the same reason (small, already non-trivial: 2 anchors resolve the join,
;; depth=3 still forces 3 extra fixpoint rounds via the inert cascade).
;;
;; :derived = sorted, NOT deduped: enc(k,n) per Busy. Pre-retract Reading population =
;; items+1 (the duplicate). remove-one leaves ONE Reading(0), so post-retract Reading
;; population = items. Expected n (post-retract, re-fired) = items = 3. Expected count =
;; anchors = 2, re-derived from the formula (constancy in depth, exactly accum-lead-rule-
;; cascade's own assertion), not read off a run.
;;
;; Variable MUST be named `staged` so GRID_SKIP_ORACLE / axes_live rewrite
;; `fire-rules$oracle staged`.
;;
;; Usage (stdin = [items anchors depth]; stdout = one #grid/Result EDN line):
;;   echo '[3 2 3]' | cargo wat ./wat-scripts/perf/grid/retract-lead-accum.wat

(:wat::core::defrecord :rla::Reading [v <- :wat::core::i64])
(:wat::core::defrecord :rla::Anchor  [k <- :wat::core::i64])
(:wat::core::defrecord :rla::Link    [level <- :wat::core::i64])
(:wat::core::defrecord :rla::Busy    [k <- :wat::core::i64  n <- :wat::core::i64])

(:wat::core::defrecord :grid::Result
  [axis      <- :wat::core::String
   size      <- (:wat::core::PersistentVector :- [:wat::core::i64])
   derived   <- (:wat::core::PersistentVector :- [:wat::core::i64])
   native-ns      <- :wat::core::i64
   oracle-derived <- (:wat::core::PersistentVector :- [:wat::core::i64])
   oracle-ns      <- :wat::core::i64])

(:wat::rete::defquery :rla::q-Busy
  :params []
  :when [(?fact <- :rla::Busy)])


;; Link(k) :- Link(k-1). Generated per depth; Busy never mentions Link. Same as
;; accum-lead-rule-cascade's build-link.
(:wat::core::defn :rla::build-link [k <- :wat::core::i64] -> :wat::rete::Rule
  (:wat::core::let [prev (:wat::core::i64::- k 1)
                    c (:wat::core::quasiquote (:rla::Link (?l <- :level) (:wat::rete::core::i64::= ?l (:wat::core::unquote prev))))
                    t (:wat::core::quasiquote (:rla::Link (:wat::core::unquote k)))]
    (:wat::rete::Rule :name (:wat::core::i64::to-string k)
      :lhs (:wat::core::PersistentVector c)
      :rhs (:wat::core::PersistentVector t))))

;; ★ THE CELL. leading accumulate, then a join — accum-lead-rule-cascade's exact shape.
;; Reading (the accumulate's :from source) is what gets duplicated + retracted.
(:wat::core::defn :rla::busy-rule [] -> :wat::rete::Rule
  (:wat::rete::Rule :name "busy"
    :lhs (:wat::core::PersistentVector
      (:wat::core::quote (?n <- (:wat::rete::acc::count) :from (:rla::Reading)))
      (:wat::core::quote (:rla::Anchor (?k <- :k))))
    :rhs (:wat::core::PersistentVector
      (:wat::core::quote (:rla::Busy ?k ?n)))))

(:wat::core::defn :rla::build-rules [depth <- :wat::core::i64] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::rete::Rule])  k <- :wat::core::i64]
                    -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
      (:wat::core::PersistentVector/conj acc (:rla::build-link k)))
    (:wat::core::PersistentVector (:rla::busy-rule))
    (:wat::core::range 1 (:wat::core::i64::+ depth 1))))

;; seed-facts items anchors — Reading(i) once each for i in [0,items), then ONE extra
;; Reading(0), Anchor(k) for k in [0,anchors), and Link(0). Duplicate ONLY the retracted
;; key, exactly retract-multiplicity's own care ("the justified derived-multiplicity split
;; ... cannot dominate") — Reading is a plain base fact, never derived.
(:wat::core::defn :rla::seed-facts [items <- :wat::core::i64  anchors <- :wat::core::i64]
  -> (:wat::core::PersistentVector :- [:wat::core::Record])
  (:wat::core::let
    [with-read (:wat::core::PersistentVector/conj
                  (:wat::core::foldl
                    (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::core::Record])  v <- :wat::core::i64]
                                    -> (:wat::core::PersistentVector :- [:wat::core::Record])
                      (:wat::core::PersistentVector/conj acc (:rla::Reading :v v)))
                    (:wat::core::PersistentVector)
                    (:wat::core::range 0 items))
                  (:rla::Reading :v 0))
     with-anch (:wat::core::foldl
                  (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::core::Record])  k <- :wat::core::i64]
                                  -> (:wat::core::PersistentVector :- [:wat::core::Record])
                    (:wat::core::PersistentVector/conj acc (:rla::Anchor :k k)))
                  with-read
                  (:wat::core::range 0 anchors))]
    (:wat::core::PersistentVector/conj with-anch (:rla::Link :level 0))))

(:wat::core::defn :rla::seed [session <- :wat::rete::Session  items <- :wat::core::i64  anchors <- :wat::core::i64] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::insert-all session (:rla::seed-facts items anchors))
    ((:wat::rete::InsertOutcome::Inserted __staged) __staged)
    ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count)
     (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None))))

(:wat::core::defn :rla::fire [s <- :wat::rete::Session] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::fire-rules s)
    ((:wat::rete::FireOutcome::Fired __fired) __fired)
    ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds)
     (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None))
    ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still)
     (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None))))

(:wat::core::defn :rla::enc [k <- :wat::core::i64  n <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::i64::+ (:wat::core::i64::* k 1000000000000000) n))

(:wat::core::defn :rla::vec->pvec [v <- (:wat::core::Vector :- [:wat::core::i64])] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::into (:wat::core::PersistentVector) v))

;; Busy only. Link is derived and unqueried.
(:wat::core::defn :rla::derived-vector [fired <- :wat::rete::Session] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::let [codes (:wat::core::into (:wat::core::Vector :wat::core::i64)
                            (:wat::core::map
                              (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::i64
                                (:wat::core::let [f (:wat::core::Option/expect (:wat::core::PersistentMap/get p "?fact") "query: ?fact")]
                                  (:rla::enc (:rla::Busy/k f) (:rla::Busy/n f))))
                              (:wat::rete::query fired (:rla::q-Busy))))]
    (:rla::vec->pvec (:wat::core::sort codes))))

(:wat::core::defn :rla::ns-between [t0 <- :wat::time::Instant  t1 <- :wat::time::Instant] -> :wat::core::i64
  (:wat::core::i64::- (:wat::time::epoch-nanos t1) (:wat::time::epoch-nanos t0)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [params  (:wat::core::match (:wat::kernel::readln ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))
                    items   (:wat::core::Option/expect (:wat::core::get params 0) "stdin: [items anchors depth]")
                    anchors (:wat::core::Option/expect (:wat::core::get params 1) "stdin: [items anchors depth]")
                    depth   (:wat::core::Option/expect (:wat::core::get params 2) "stdin: [items anchors depth]")
                    rules   (:rla::build-rules depth)
                    seeded  (:rla::seed (:wat::core::match (:wat::rete::compile-all rules (:wat::core::PersistentVector (:rla::q-Busy))) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __fact-type) (:wat::kernel::assertion-failed! "compile: the rule set may not terminate" :wat::core::None :wat::core::None))) items anchors)
                    ;; fire; retract Reading(0) ONCE; the retracted session is what both
                    ;; engines re-fire. Variable MUST be named `staged` so GRID_SKIP_ORACLE /
                    ;; axes_live rewrite `fire-rules$oracle staged` (the re-fire).
                    pre     (:rla::fire seeded)
                    staged  (:wat::rete::retract pre (:rla::Reading 0))
                    n0      (:wat::time::now)
                    fired   (:rla::fire staged)
                    n1      (:wat::time::now)
                    derived (:rla::derived-vector fired)
                    nat-ns  (:rla::ns-between n0 n1)
                    o0      (:wat::time::now)
                    ofired  (:wat::core::match (:wat::rete::fire-rules$oracle staged) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))
                    o1      (:wat::time::now)]
    (:wat::kernel::println
      (:grid::Result :axis "retract-lead-accum" :size (:wat::core::PersistentVector items anchors depth) :derived derived :native-ns nat-ns :oracle-derived (:rla::derived-vector ofired) :oracle-ns (:rla::ns-between o0 o1)))))
