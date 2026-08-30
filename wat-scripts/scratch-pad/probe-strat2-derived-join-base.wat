;; REGRESSION PROBE (was RED) — a POST-NEGATION derived fact must join with a BASE fact
;; downstream. FOUND RED 2026-08-13, CLOSED the same day in ff581b6f: the stratifier now
;; propagates POSITIVE dependencies. Both arms must read 1; a 0 in SUBJECT is that regression.
;;
;; FOUND 2026-08-13 by building the migration's gate/unlock chain (rules-corpus-02), where
;; negation IS the gate: "a concept is settled iff nothing derived it inconsistent", and the
;; next rule joins that settled concept against a base-fact ruling table. The settled facts
;; derive correctly; the downstream join silently yielded nothing.
;;
;; The two arms below differ in EXACTLY ONE THING — whether `:z::S` is derived through a
;; `(:wat::rete::not …)` condition. BEFORE the fix the downstream join flipped 1 -> 0 on that
;; single difference; that is what named the mechanism.
;;
;;   CONTROL  (settled without negation, stratum 1):  S = 1   Out = 1
;;   SUBJECT  (settled through negation, stratum 2):  S = 1   Out = 0   <- WRONG
;;
;; `S` is correct in both arms, so this is not a derivation bug — it is the DOWNSTREAM JOIN
;; failing to see a fact that demonstrably exists. Nothing about the join itself is exotic:
;; single variable, string key, base-fact partner, and the identical join works in the control.
;;
;; WHY THE GRID NEVER CAUGHT IT: the negation axes assert on the negation's OWN output. None
;; of them chains a THIRD rule off a post-negation fact. The engine's stratified negation is
;; proven (R18/R20, native == oracle == Clara); what was NEVER proven — and was broken — was
;; CONSUMING a stratum-2 fact in stratum 3.
;;
;; ⚠ This BLOCKED forward-chaining gate/unlock designs generally: "derive a gate by negation,
;; then join the gate against a table" is the shape. It works as of ff581b6f.
;;
;; Both arms MUST read 1. This file is loader-gated, so it stays alive; the run keeps it honest.
;; A 0 in SUBJECT is the regression returning.

(:wat::core::defrecord :z::A   [c <- :wat::core::String])
(:wat::core::defrecord :z::Bad [c <- :wat::core::String])
(:wat::core::defrecord :z::R   [c <- :wat::core::String  t <- :wat::core::String])
(:wat::core::defrecord :z::S   [c <- :wat::core::String])
(:wat::core::defrecord :z::Out [c <- :wat::core::String  t <- :wat::core::String])

;; SUBJECT — stratum 2: derived THROUGH a negation. `:z::Bad` is never seeded, so the
;; negation is satisfied and `S` must (and does) derive.
(:wat::rete::defrule :z::settled-neg
  :when [(:z::A (?c <- :c))
         (:wat::rete::not (:z::Bad (?c <- :c)))]
  :then [(:z::S :c ?c)])

;; CONTROL — stratum 1: the same conclusion with no negation in its path.
(:wat::core::defrecord :z::S2  [c <- :wat::core::String])
(:wat::core::defrecord :z::Out2 [c <- :wat::core::String  t <- :wat::core::String])
(:wat::rete::defrule :z::settled-plain
  :when [(:z::A (?c <- :c))]
  :then [(:z::S2 :c ?c)])

;; The IDENTICAL downstream join, once per arm.
(:wat::rete::defrule :z::out-neg
  :when [(:z::S (?c <- :c)) (:z::R (?c <- :c) (?t <- :t))]
  :then [(:z::Out :c ?c :t ?t)])

(:wat::rete::defrule :z::out-plain
  :when [(:z::S2 (?c <- :c)) (:z::R (?c <- :c) (?t <- :t))]
  :then [(:z::Out2 :c ?c :t ?t)])

(:wat::rete::defquery :z::q-S
  :params []
  :when [(?fact <- :z::S)])


(:wat::rete::defquery :z::q-S2
  :params []
  :when [(?fact <- :z::S2)])


(:wat::rete::defquery :z::q-Out2
  :params []
  :when [(?fact <- :z::Out2)])


(:wat::rete::defquery :z::q-Out
  :params []
  :when [(?fact <- :z::Out)])


(:wat::core::defn :z::show [label <- :wat::core::String n <- :wat::core::i64] -> :wat::core::nil
  (:wat::kernel::println (:wat::core::string::concat label (:wat::core::str n))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [f (:wat::core::match (:wat::rete::fire-rules
         (:wat::core::match (:wat::rete::insert
           (:wat::core::match (:wat::rete::insert
             (:wat::core::match (:wat::rete::compile-all (:wat::core::PersistentVector
               (:z::settled-neg) (:z::settled-plain) (:z::out-neg) (:z::out-plain)) (:wat::core::PersistentVector (:z::q-S) (:z::q-S2) (:z::q-Out2) (:z::q-Out))) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __fact-type) (:wat::kernel::assertion-failed! "compile: the rule set may not terminate" :wat::core::None :wat::core::None)))
             (:z::A :c "i64")) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))
           (:z::R :c "i64" :t "wat.core.i64")) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))]
    (:wat::core::do
      ;; non-vacuity: both gates must derive, or the Out rows below mean nothing
      (:z::show "S  via negation (want 1): " (:wat::core::length (:wat::rete::query f (:z::q-S))))
      (:z::show "S2 plain         (want 1): " (:wat::core::length (:wat::rete::query f (:z::q-S2))))
      ;; the differential — these two must agree
      (:z::show "Out2 CONTROL     (want 1): " (:wat::core::length (:wat::rete::query f (:z::q-Out2))))
      (:z::show "Out  SUBJECT     (want 1; GREEN since ff581b6f closed #94): "
        (:wat::core::length (:wat::rete::query f (:z::q-Out)))))))
