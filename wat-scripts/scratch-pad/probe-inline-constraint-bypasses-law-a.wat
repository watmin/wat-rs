;; probe-inline-constraint-bypasses-law-a.wat — RECONNAISSANCE, 2026-08-06.
;;
;; The run that found the hole behind #83. Kept as the located record of HOW it was found;
;; the durable gate is `tests/rete/probe_arc278_inline_constraint_law_a.rs` (4 RED rows) and
;; the reasoning is `DESIGN-STONE-inline-constraint-admits-non-rete.md`.
;;
;; QUESTION: is `where` the ONLY expression surface on a rule's LHS?  ANSWER: no.
;;
;; `compile-condition` (wat/rete.wat:679) branches on four shapes — where / not / exists /
;; accumulate — and only the first and last carry a fence. A fact pattern's own children are
;; classified by a SEPARATE grammar in Rust, `classify_rete_clause` (matcher.rs:331), matching six
;; literal strings: `:wat::core::{= not= < > <= >=}`. Law A never sees them.
;;
;; TWO RESULTS, both kept — the negative one is what located the real door:
;;
;;   (a) TOP-LEVEL in the `:when`  ->  REFUSED, but NOT by law A:
;;         #wat.rete/UnknownFactType — "`:wat::core::>` is not a registered fact type"
;;       compile-condition has no branch for it, so it fell to the fact-pattern else-branch and the
;;       fact-type registry caught it. A refusal for an unrelated reason.
;;
;;   (b) NESTED INSIDE the fact pattern  ->  ADMITTED. Compiles, fires, and discriminates.
;;         #wat.core/PersistentMap {:rule-count 2 :flagged-count 1}
;;       Two rules compiled; the value-3 fact was correctly excluded, so the constraint did not
;;       merely parse — it filtered. THAT is the hole.
;;
;; Run: target/release/wat wat-scripts/scratch-pad/probe-inline-constraint-bypasses-law-a.wat

(:wat::core::defrecord :probe::Reading [location <- :wat::core::String  value <- :wat::core::i64])
(:wat::core::defrecord :probe::Flagged [location <- :wat::core::String])

;; CONTROL — the same predicate through the FENCED surface. Rete-spelled, law-A clean.
(:wat::rete::defrule :probe::via-where
  :when
  [(:probe::Reading (?loc <- :location) (?v <- :value))
   (:wat::rete::where (:wat::rete::core::i64::> ?v 10))]
  :then
  [(:probe::Flagged :location ?loc)])

;; SUBJECT — the same predicate as an inline constraint NESTED in the pattern, core-spelled
;; generic `>`. It compiles. Law A does not govern the whole LHS.
(:wat::rete::defrule :probe::via-inline-constraint
  :when
  [(:probe::Reading (?loc <- :location) (?v <- :value) (:wat::rete::core::i64::> :value 10))]
  :then
  [(:probe::Flagged :location ?loc)])

(:wat::rete::defquery :probe::q-Flagged
  :params []
  :when [(?fact <- :probe::Flagged)])


(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [rules   (:wat::rete::collect-rules :probe)
     session (:wat::rete::compile-all rules (:wat::core::PersistentVector (:probe::q-Flagged)))
     session (:wat::rete::insert session (:probe::Reading :location "Oslo"   :value 42))
     session (:wat::rete::insert session (:probe::Reading :location "Bergen" :value 3))
     fired   (:wat::rete::fire-rules session)
     flagged (:wat::rete::query fired (:probe::q-Flagged))]
    ;; :rule-count 2 = BOTH compiled (the top-level form (a) never reached this point).
    ;; :flagged-count 1 = only Oslo; Bergen (value 3) was filtered, so the constraint discriminated.
    ;;
    ;; THIS PAIR CANNOT ANSWER A SECOND QUESTION, and two runs were wasted learning that: with the
    ;; control rule live, `flagged-count` is 1 whether the SUBJECT fired or not. It measures "the
    ;; hole exists", never "what a cross-type compare does". The Rust probe uses a distinct fixture
    ;; per question for exactly this reason.
    (:wat::kernel::println
      (:wat::core::PersistentMap
        :rule-count    (:wat::core::length rules)
        :flagged-count (:wat::core::length flagged)))))
