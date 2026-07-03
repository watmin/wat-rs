;; Fixture BESIDE probe_arc278_7strat_native_differential.rs — the canonical stratified-negation world,
;; loaded via startup_beside(file!()). Mirrors wat-scripts/fixes/rete-truth-maintenance-probes/{neg.wat,neg.clj}.
;;
;; A(1),A(2): mark-bad derives Bad for k=2 only; ok = A with NO Bad (negation over a DERIVED fact →
;; needs stratification). Correct closure = {Bad:1, Ok:1} — only k=1 has no Bad. Native raw leaks Ok=2.
;;
;; NO :user::main — a world-under-test needs none (freeze does not require it; cf. startup_bare, freeze.rs:738).

(:wat::core::defrecord :n::A   [k <- :wat::core::i64])
(:wat::core::defrecord :n::Bad [k <- :wat::core::i64])
(:wat::core::defrecord :n::Ok  [k <- :wat::core::i64])

;; derive Bad for k=2 only
(:wat::rete::defrule :n::mark-bad
  :when [(:n::A (?k <- :k)) (:wat::rete::where (:wat::core::= ?k 2))]
  :then (:wat::rete::insert (:n::Bad ?k)))

;; Ok = A with NO Bad (negation over a DERIVED fact — needs stratification)
(:wat::rete::defrule :n::ok
  :when [(:n::A (?k <- :k)) (:wat::rete::not (:n::Bad (?k <- :k)))]
  :then (:wat::rete::insert (:n::Ok ?k)))
