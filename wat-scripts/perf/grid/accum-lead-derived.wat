;; wat-scripts/perf/grid/accum-lead-derived.wat — COMPOUND CORRECTNESS AXIS: cell
;; (record, absent, derived, leading, na) of the peragrare grid —
;; docs/arc/2026/06/278-rules-engine/strike-grid-first-compound-cell/DESIGN.md.
;;
;; A LEADING accumulate (no parent condition — same shape as accum-lead-rule-cascade's
;; and leading-exists's `L` mechanism) whose `:from` type is itself DERIVED within the
;; same ruleset, needing cross-round supersession (accum-over-derived's `A` mechanism).
;; THIS AXIS IS THEIR INTERSECTION, not a new invention — read both first:
;;
;;   accum-over-derived   (record, absent, derived,    nonleading, na)  — Seed anchors
;;                        the accumulate, so it has a parent condition.
;;   leading-exists       (record, absent, none,       leading,    na)  — the leading
;;                        condition's `:from`/witness type (Wind) is a PLAIN seeded fact,
;;                        never derived.
;;   THIS AXIS            (record, absent, derived,    leading,    na)  — Seed is DROPPED
;;                        entirely: the accumulate is the rule's ONLY condition (leading,
;;                        exactly like accum-lead-rule-cascade's "no parent condition"
;;                        `busy-rule`, and syntactically legal — where-accum-lead.wat's
;;                        `count-zero`/`count-three` rules already run an accumulate as
;;                        the sole LHS condition), and its `:from` is `Step`, which the
;;                        SAME ruleset derives via a `Step(k) :- Step(k-1)` cascade.
;;
;; Seed;  Step(0);  Step(k) :- Step(k-1) for k in [1,depth];
;; Tally(n) :- [?n <- (acc::count) :from Step]   ;; LEADING — no join, no anchor.
;;
;; WHY THIS IS THE RISK, not a formality: accum-over-derived's cure makes native put Tally
;; at stratum 1 and evaluate it ONCE, after Step closes — proven where the accumulate has a
;; parent (Seed). leading-exists's cure makes a leading non-monotonic condition stop
;; re-emitting one token per fixpoint round — proven where the witnessed type (Wind) is a
;; plain seeded fact. NEITHER fixture drives a leading accumulate whose OWN `:from` type is
;; still being derived out from under it, round after round, while it has no parent token
;; to anchor a single join evaluation. If the two cures do not compose, this axis is where
;; that would show: either as leaked intermediate Tallys (accum-over-derived's failure
;; shape) or as one Tally per fixpoint round (leading-exists's failure shape) — or both.
;;
;; ⛔ ONE CORRECTNESS SIZE, not a sweep — same reasoning as accum-over-derived.wat: a
;; `gen-accum-lead-derived.sh` would drag a correctness proof onto the perf ladder
;; (`run-all.sh:81-87`, `check-grid-speed.sh` treats `:accuracy :MISMATCH` as a gate
;; failure). Static `accum-lead-derived.clj`.
;;
;; Depth is a stated choice: 9 — same as accum-over-derived, for the same reason (past the
;; probe's two intermediate states, 4.5x). If the oracle does not terminate, hits the round
;; cap, or takes minutes here, that is STOP-3 — do not shrink until it is quiet.
;;
;; :derived = sorted, NOT deduped: enc(0,k,0) per derived Step level k in [1,depth] plus
;; enc(1,0,n) per Tally. A leaked intermediate tally (accum-over-derived's failure shape)
;; OR a per-round duplicate (leading-exists's failure shape) is an EXTRA ELEMENT either way.
;; Expected count = depth + 1, re-derived from this formula, not read off a run.
;; Tally n = count of ALL Step facts including seeded Step(0) = depth + 1.
;;
;; Variable MUST be named `staged` so GRID_SKIP_ORACLE / axes_live rewrite
;; `fire-rules$oracle staged`.
;;
;; Usage (stdin = an i64 vector [depth]; stdout = one #grid/Result EDN line):
;;   echo '[9]' | cargo wat ./wat-scripts/perf/grid/accum-lead-derived.wat

(:wat::core::defrecord :ald::Step  [level <- :wat::core::i64])
(:wat::core::defrecord :ald::Tally [n <- :wat::core::i64])

(:wat::core::defrecord :grid::Result
  [axis      <- :wat::core::String
   size      <- (:wat::core::PersistentVector :- [:wat::core::i64])
   derived   <- (:wat::core::PersistentVector :- [:wat::core::i64])
   native-ns      <- :wat::core::i64
   oracle-derived <- (:wat::core::PersistentVector :- [:wat::core::i64])
   oracle-ns      <- :wat::core::i64])

(:wat::rete::defquery :ald::q-Step
  :params []
  :when [(?fact <- :ald::Step)])


(:wat::rete::defquery :ald::q-Tally
  :params []
  :when [(?fact <- :ald::Tally)])


;; build-step k — Step(k) :- Step(k-1). Level literals spliced via quasiquote, same as
;; accum-over-derived's per-level rule. One generated rule per level.
(:wat::core::defn :ald::build-step [k <- :wat::core::i64] -> :wat::rete::Rule
  (:wat::core::let [prev (:wat::core::i64::- k 1)
                    c (:wat::core::quasiquote (:ald::Step (?l <- :level) (:wat::rete::core::i64::= ?l (:wat::core::unquote prev))))
                    t (:wat::core::quasiquote (:ald::Step (:wat::core::unquote k)))]
    (:wat::rete::Rule :name (:wat::core::i64::to-string k)
      :lhs (:wat::core::PersistentVector c)
      :rhs (:wat::core::PersistentVector t))))

;; ★ THE CELL. tally is a LEADING accumulate — its ONLY condition, no anchor, no join —
;; over Step, which the SAME ruleset derives via build-step above.
(:wat::core::defn :ald::tally-rule [] -> :wat::rete::Rule
  (:wat::rete::Rule :name "tally"
    :lhs (:wat::core::PersistentVector
      (:wat::core::quote (?n <- (:wat::rete::acc::count) :from (:ald::Step))))
    :rhs (:wat::core::PersistentVector
      (:wat::core::quote (:ald::Tally ?n)))))

;; build-rules depth — tally plus one Step(k):-Step(k-1) per k in [1,depth].
(:wat::core::defn :ald::build-rules [depth <- :wat::core::i64] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::rete::Rule])  k <- :wat::core::i64]
                    -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
      (:wat::core::PersistentVector/conj acc (:ald::build-step k)))
    (:wat::core::PersistentVector (:ald::tally-rule))
    (:wat::core::range 1 (:wat::core::i64::+ depth 1))))

(:wat::core::defn :ald::seed-facts [] -> (:wat::core::PersistentVector :- [:wat::core::Record])
  (:wat::core::PersistentVector/conj
    (:wat::core::PersistentVector)
    (:ald::Step :level 0)))

(:wat::core::defn :ald::seed [session <- :wat::rete::Session] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::insert-all session (:ald::seed-facts))
    ((:wat::rete::InsertOutcome::Inserted __staged) __staged)
    ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count)
     (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None))))

(:wat::core::defn :ald::fire [s <- :wat::rete::Session] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::fire-rules s)
    ((:wat::rete::FireOutcome::Fired __fired) __fired)
    ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds)
     (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None))
    ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still)
     (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None))))

(:wat::core::defn :ald::enc [kind <- :wat::core::i64  level <- :wat::core::i64  id <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::i64::+
    (:wat::core::i64::+ (:wat::core::i64::* kind 1000000000000000) (:wat::core::i64::* level 1000000000))
    id))

(:wat::core::defn :ald::vec->pvec [v <- (:wat::core::Vector :- [:wat::core::i64])] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::into (:wat::core::PersistentVector) v))

;; derived-vector — sorted, NOT deduped. Derived Step levels (level > 0) plus every Tally.
;; A leaked intermediate tally OR a per-round duplicate is an EXTRA ELEMENT.
(:wat::core::defn :ald::derived-vector [fired <- :wat::rete::Session] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::let
    [c0 (:wat::core::into (:wat::core::Vector :wat::core::i64)
          (:wat::core::map
            (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::i64
              (:wat::core::let [f (:wat::core::Option/expect (:wat::core::PersistentMap/get p "?fact") "query: ?fact")]
                (:ald::enc 0 (:ald::Step/level f) 0)))
            (:wat::core::filter
              (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::bool
                (:wat::core::let [f (:wat::core::Option/expect (:wat::core::PersistentMap/get p "?fact") "query: ?fact")]
                  (:wat::core::i64::> (:ald::Step/level f) 0)))
              (:wat::rete::query fired (:ald::q-Step)))))
     c1 (:wat::core::into c0
          (:wat::core::map
            (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::i64
              (:wat::core::let [f (:wat::core::Option/expect (:wat::core::PersistentMap/get p "?fact") "query: ?fact")]
                (:ald::enc 1 0 (:ald::Tally/n f))))
            (:wat::rete::query fired (:ald::q-Tally))))]
    (:ald::vec->pvec (:wat::core::sort c1))))

(:wat::core::defn :ald::ns-between [t0 <- :wat::time::Instant  t1 <- :wat::time::Instant] -> :wat::core::i64
  (:wat::core::i64::- (:wat::time::epoch-nanos t1) (:wat::time::epoch-nanos t0)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [params  (:wat::core::match (:wat::kernel::readln ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))
                    depth   (:wat::core::Option/expect (:wat::core::get params 0) "stdin: [depth]")
                    rules   (:ald::build-rules depth)
                    session (:wat::core::match (:wat::rete::compile-all rules (:wat::core::PersistentVector (:ald::q-Step) (:ald::q-Tally))) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __fact-type) (:wat::kernel::assertion-failed! "compile: the rule set may not terminate" :wat::core::None :wat::core::None)))
                    ;; Variable MUST be named `staged` so GRID_SKIP_ORACLE / axes_live rewrite
                    ;; `fire-rules$oracle staged` and not a first-fire leftover.
                    staged  (:ald::seed session)
                    n0      (:wat::time::now)
                    fired   (:ald::fire staged)
                    n1      (:wat::time::now)
                    derived (:ald::derived-vector fired)
                    nat-ns  (:ald::ns-between n0 n1)
                    o0      (:wat::time::now)
                    ofired  (:wat::core::match (:wat::rete::fire-rules$oracle staged) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))
                    o1      (:wat::time::now)]
    (:wat::kernel::println
      (:grid::Result :axis "accum-lead-derived" :size (:wat::core::PersistentVector depth) :derived derived :native-ns nat-ns :oracle-derived (:ald::derived-vector ofired) :oracle-ns (:ald::ns-between o0 o1)))))
