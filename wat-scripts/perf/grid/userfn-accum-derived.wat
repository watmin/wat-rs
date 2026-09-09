;; wat-scripts/perf/grid/userfn-accum-derived.wat — COMPOUND CORRECTNESS AXIS: cell
;; (userfn, absent, derived, nonleading, na) of the peragrare grid — cell #1 of
;; docs/arc/2026/06/278-rules-engine/strike-grid-remaining-compound-cells/DESIGN.md.
;;
;; A user-fn `:then` head (userfn-head's `H` mechanism — stratify by PRODUCED type, not fn
;; name) whose LHS accumulates over a type the SAME ruleset derives (accum-over-derived's
;; `A` mechanism — an AccumulateNode's `:from` type itself derived within the ruleset,
;; needing cross-round supersession). THIS AXIS IS THEIR INTERSECTION — read both first:
;;
;;   userfn-head          (userfn, absent, none,    none,       consumed) — the `:then` head
;;                        is a user fn (`mk-rate`) constructing Rate from a plain Src join;
;;                        Rate is NOT itself a source of any accumulate.
;;   accum-over-derived   (record, absent, derived,  nonleading, na)      — the accumulate's
;;                        `:then` head directly constructs `(:aod::Tally ?n)`; no user fn.
;;   THIS AXIS            (userfn, absent, derived,  nonleading, na)      — `accum-over-
;;                        derived`'s EXACT shape (Seed anchors the accumulate; Step(k) :-
;;                        Step(k-1) makes Step derived), with the tally rule's `:then`
;;                        replaced by a call to a user fn `mk-tally`, exactly userfn-head's
;;                        `(mk-rate ?k)` idiom — a bound var in, one record out, no compute.
;;
;; Seed;  Step(0);  Step(k) :- Step(k-1) for k in [1,depth];
;; Tally(n) :- Seed AND [?n <- (acc::count) :from Step], :then [(mk-tally ?n)]
;;
;; WHY THIS IS THE RISK, not a formality: `userfn-head`'s cure stratifies by the type a
;; `:then` head ACTUALLY CONSTRUCTS, not the fn's name. `accum-over-derived`'s cure puts an
;; accumulate over a derived `:from` type at the stratum AFTER that type closes, evaluating
;; it once. NEITHER fixture drives an accumulate whose OWN `:then` head is a level of
;; indirection (a user fn) rather than a literal record constructor — if the produced-type
;; stratifier resolves the type through `mk-tally`'s return annotation only when the RHS is a
;; bare quoted form (not when it is itself a user-fn call sitting on an accumulate's output),
;; this is where that would show: either a stale/leaked intermediate Tally
;; (accum-over-derived's failure shape) or the Tally never appearing at all (userfn-head's
;; pre-cure failure shape, "Out dropped").
;;
;; ⛔ ONE CORRECTNESS SIZE, not a sweep — same reasoning as both parents: a
;; `gen-userfn-accum-derived.sh` would drag a correctness proof onto the perf ladder
;; (`run-all.sh:81-87`, `check-grid-speed.sh` treats `:accuracy :MISMATCH` as a gate
;; failure). Static `userfn-accum-derived.clj`.
;;
;; Depth is a stated choice: 9 — same as accum-over-derived/accum-lead-derived, for the same
;; reason (past the probe's two intermediate states, 4.5x). If the oracle does not
;; terminate, hits the round cap, or takes minutes here, that is STOP-3 — do not shrink
;; until it is quiet.
;;
;; :derived = sorted, NOT deduped: enc(0,k,0) per derived Step level k in [1,depth] plus
;; enc(1,0,n) per Tally. Expected count = depth + 1, re-derived from this formula (identical
;; to accum-over-derived's — mk-tally is a pure 1:1 wrapper and changes no cardinality), not
;; read off a run. Tally n = count of ALL Step facts including seeded Step(0) = depth + 1.
;;
;; mk-tally is `:wat::rete::core::defn` returning Tally, body exactly `(:cad::Tally :n n)` —
;; a bound var, not a computed mint (userfn-head's own contract: a computed mint IS refused
;; by rete_fn_body_mints).
;;
;; Variable MUST be named `staged` so GRID_SKIP_ORACLE / axes_live rewrite
;; `fire-rules$oracle staged`.
;;
;; Usage (stdin = an i64 vector [depth]; stdout = one #grid/Result EDN line):
;;   echo '[9]' | cargo wat ./wat-scripts/perf/grid/userfn-accum-derived.wat

(:wat::core::defrecord :cad::Seed  [id <- :wat::core::i64])
(:wat::core::defrecord :cad::Step  [level <- :wat::core::i64])
(:wat::core::defrecord :cad::Tally [n <- :wat::core::i64])

(:wat::core::defrecord :grid::Result
  [axis      <- :wat::core::String
   size      <- (:wat::core::PersistentVector :- [:wat::core::i64])
   derived   <- (:wat::core::PersistentVector :- [:wat::core::i64])
   native-ns      <- :wat::core::i64
   oracle-derived <- (:wat::core::PersistentVector :- [:wat::core::i64])
   oracle-ns      <- :wat::core::i64])

(:wat::rete::core::defn :cad::mk-tally
  [n <- :wat::core::i64]
  -> :cad::Tally
  (:cad::Tally :n n))

(:wat::rete::defquery :cad::q-Step
  :params []
  :when [(?fact <- :cad::Step)])


(:wat::rete::defquery :cad::q-Tally
  :params []
  :when [(?fact <- :cad::Tally)])


;; build-step k — Step(k) :- Step(k-1). Level literals spliced via quasiquote, same as
;; accum-over-derived's per-level rule. One generated rule per level.
(:wat::core::defn :cad::build-step [k <- :wat::core::i64] -> :wat::rete::Rule
  (:wat::core::let [prev (:wat::core::i64::- k 1)
                    c (:wat::core::quasiquote (:cad::Step (?l <- :level) (:wat::rete::core::i64::= ?l (:wat::core::unquote prev))))
                    t (:wat::core::quasiquote (:cad::Step (:wat::core::unquote k)))]
    (:wat::rete::Rule :name (:wat::core::i64::to-string k)
      :lhs (:wat::core::PersistentVector c)
      :rhs (:wat::core::PersistentVector t))))

;; ★ THE CELL. tally is Seed-anchored (nonleading, like accum-over-derived) over Step, which
;; the SAME ruleset derives — but its `:then` head is a call to the user fn `mk-tally`, not a
;; bare record constructor.
(:wat::core::defn :cad::tally-rule [] -> :wat::rete::Rule
  (:wat::rete::Rule :name "tally"
    :lhs (:wat::core::PersistentVector
      (:wat::core::quote (:cad::Seed (?id <- :id)))
      (:wat::core::quote (?n <- (:wat::rete::acc::count) :from (:cad::Step))))
    :rhs (:wat::core::PersistentVector
      (:wat::core::quote (:cad::mk-tally ?n)))))

;; build-rules depth — tally plus one Step(k):-Step(k-1) per k in [1,depth].
(:wat::core::defn :cad::build-rules [depth <- :wat::core::i64] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::rete::Rule])  k <- :wat::core::i64]
                    -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
      (:wat::core::PersistentVector/conj acc (:cad::build-step k)))
    (:wat::core::PersistentVector (:cad::tally-rule))
    (:wat::core::range 1 (:wat::core::i64::+ depth 1))))

(:wat::core::defn :cad::empty-records [] -> (:wat::core::PersistentVector :- [:wat::core::Record])
  (:wat::core::PersistentVector))

(:wat::core::defn :cad::seed-facts [] -> (:wat::core::PersistentVector :- [:wat::core::Record])
  (:wat::core::PersistentVector/conj
    (:wat::core::PersistentVector/conj
      (:cad::empty-records)
      (:cad::Seed :id 0))
    (:cad::Step :level 0)))

(:wat::core::defn :cad::seed [session <- :wat::rete::Session] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::insert-all session (:cad::seed-facts))
    ((:wat::rete::InsertOutcome::Inserted __staged) __staged)
    ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count)
     (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None))))

(:wat::core::defn :cad::fire [s <- :wat::rete::Session] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::fire-rules s)
    ((:wat::rete::FireOutcome::Fired __fired) __fired)
    ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds)
     (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None))
    ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still)
     (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None))))

(:wat::core::defn :cad::enc [kind <- :wat::core::i64  level <- :wat::core::i64  id <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::i64::+
    (:wat::core::i64::+ (:wat::core::i64::* kind 1000000000000000) (:wat::core::i64::* level 1000000000))
    id))

(:wat::core::defn :cad::vec->pvec [v <- (:wat::core::Vector :- [:wat::core::i64])] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::into (:wat::core::PersistentVector) v))

;; derived-vector — sorted, NOT deduped. Derived Step levels (level > 0) plus every Tally.
(:wat::core::defn :cad::derived-vector [fired <- :wat::rete::Session] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::let
    [c0 (:wat::core::into (:wat::core::Vector :wat::core::i64)
          (:wat::core::map
            (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::i64
              (:wat::core::let [f (:wat::core::Option/expect (:wat::core::PersistentMap/get p "?fact") "query: ?fact")]
                (:cad::enc 0 (:cad::Step/level f) 0)))
            (:wat::core::filter
              (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::bool
                (:wat::core::let [f (:wat::core::Option/expect (:wat::core::PersistentMap/get p "?fact") "query: ?fact")]
                  (:wat::core::i64::> (:cad::Step/level f) 0)))
              (:wat::rete::query fired (:cad::q-Step)))))
     c1 (:wat::core::into c0
          (:wat::core::map
            (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::i64
              (:wat::core::let [f (:wat::core::Option/expect (:wat::core::PersistentMap/get p "?fact") "query: ?fact")]
                (:cad::enc 1 0 (:cad::Tally/n f))))
            (:wat::rete::query fired (:cad::q-Tally))))]
    (:cad::vec->pvec (:wat::core::sort c1))))

(:wat::core::defn :cad::ns-between [t0 <- :wat::time::Instant  t1 <- :wat::time::Instant] -> :wat::core::i64
  (:wat::core::i64::- (:wat::time::epoch-nanos t1) (:wat::time::epoch-nanos t0)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [params  (:wat::core::match (:wat::kernel::readln ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))
                    depth   (:wat::core::Option/expect (:wat::core::get params 0) "stdin: [depth]")
                    rules   (:cad::build-rules depth)
                    session (:wat::core::match (:wat::rete::compile-all rules (:wat::core::PersistentVector (:cad::q-Step) (:cad::q-Tally))) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __fact-type) (:wat::kernel::assertion-failed! "compile: the rule set may not terminate" :wat::core::None :wat::core::None)))
                    ;; Variable MUST be named `staged` so GRID_SKIP_ORACLE / axes_live rewrite
                    ;; `fire-rules$oracle staged` and not a first-fire leftover.
                    staged  (:cad::seed session)
                    n0      (:wat::time::now)
                    fired   (:cad::fire staged)
                    n1      (:wat::time::now)
                    derived (:cad::derived-vector fired)
                    nat-ns  (:cad::ns-between n0 n1)
                    o0      (:wat::time::now)
                    ofired  (:wat::core::match (:wat::rete::fire-rules$oracle staged) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))
                    o1      (:wat::time::now)]
    (:wat::kernel::println
      (:grid::Result :axis "userfn-accum-derived" :size (:wat::core::PersistentVector depth) :derived derived :native-ns nat-ns :oracle-derived (:cad::derived-vector ofired) :oracle-ns (:cad::ns-between o0 o1)))))
