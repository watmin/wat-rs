;; arc 278 / conferre L2-3 — DO THE TWO STRATIFIERS AGREE ON NUMBERS?
;;
;; `src/rete/kernel/stratify.rs:205` claims it "Mirrors `stratify-sweep`
;; (wat/rete/oracle/stratify.wat)". The native sweep carries a term the oracle's has no
;; counterpart for: `exists_and_from_types` gets `+1` when the bagged type is derived by
;; this rule set (`stratify.rs:221-227`). The oracle folds `:exists` inner and accumulate
;; `:from` into `rule-consumes` instead, and `req-pos` is explicitly NOT +1
;; (`stratify.wat:157-158`, `:233-234`).
;;
;; Both engines record only RAISED strata (`if required > cur`), so an empty map means
;; "every produced type is stratum 0" and the two maps are directly comparable.
;;
;; ⛔ THE ORACLE HALF ONLY. This drives `:wat::rete::stratify`, which has one definition —
;; the oracle's. The native stratifier is `pub(crate)` and is reached through compile, so it
;; needs a Rust driver. Do NOT read agreement into this file: it cannot see the native side,
;; which is exactly the "compares a verb to itself" blindness that hid F2.
;;
;; ANCHOR: `neg` is a known-positive control — negation over a derived type must raise a
;; stratum on BOTH engines (+1 is the one term they agree on). If `neg` prints an empty map
;; the instrument is inert and the `bag` reading below means nothing.
;;
;; Run: cargo install --path . --force   (wat/ is include_str!'d — a stale binary answers
;;      from an older stdlib with no error) then:  wat wat-scripts/scratch-pad/arc278-l2-3-stratify-numbers.wat

(:wat::core::defrecord :l23::A    [k <- :wat::core::i64])
(:wat::core::defrecord :l23::Ok   [k <- :wat::core::i64])
(:wat::core::defrecord :l23::Ok2  [k <- :wat::core::i64])
(:wat::core::defrecord :l23::Seed [id <- :wat::core::i64])
(:wat::core::defrecord :l23::Tally [n <- :wat::core::i64])

;; Ok is DERIVED — this is what makes the bag below "a type THIS SET derives".
(:wat::rete::defrule :l23::ok
  :when [(:l23::A (?k <- :k))]
  :then [(:l23::Ok :k ?k)])

;; THE ANCHOR — negation over the derived Ok. Both engines +1 here.
(:wat::rete::defrule :l23::neg
  :when [(:l23::A (?k <- :k))
         (:wat::rete::not (:l23::Ok (?k <- :k)))]
  :then [(:l23::Ok2 :k ?k)])

;; THE MEASUREMENT — accumulate :from over the derived Ok.
(:wat::rete::defrule :l23::tally
  :when [(:l23::Seed (?id <- :id))
         (?n <- (:wat::rete::acc::count) :from (:l23::Ok))]
  :then [(:l23::Tally :n ?n)])

;; ─── strike-oracle-negation-recurses PROBE — a `not` nested under `or`/`and`, as a RULE ──────
;;
;; `wat-scripts/perf/grid/where-nested-combinators.wat` q6 has this exact LHS shape —
;; `(or A (and B (not C)))` — but as a `defquery`, and stratification orders PRODUCERS
;; (a query has no `:then`). Whether a query's own stratum is computed or read at all is
;; NOT established by that file. This is the same LHS shape on a `defrule`, with C DERIVED
;; (produced by `mkc`), which IS what `rule-produces`/`rule-negates` are computed over.
(:wat::core::defrecord :l24::A    [k <- :wat::core::i64])
(:wat::core::defrecord :l24::B    [k <- :wat::core::i64])
(:wat::core::defrecord :l24::C    [k <- :wat::core::i64])
(:wat::core::defrecord :l24::Seed [k <- :wat::core::i64])
(:wat::core::defrecord :l24::Out  [k <- :wat::core::i64])

;; C is DERIVED — this is what makes the nested :not below a negation over a produced type.
(:wat::rete::defrule :l24::mkc
  :when [(:l24::Seed (?k <- :k))]
  :then [(:l24::C :k ?k)])

;; THE NESTED SHAPE — (or A (and B (not C))). Oracle's rule-negates only recurses when the
;; TOP-LEVEL LHS form's head is literally :not; here the top-level head is :or, so on the
;; oracle side this contributes NOTHING however deep the :not sits. The native rule_negates
;; recurses through And/Or unconditionally (negate_types, stratify.rs).
(:wat::rete::defrule :l24::nested
  :when [(:wat::rete::or (:l24::A) (:wat::rete::and (:l24::B) (:wat::rete::not (:l24::C))))]
  :then [(:l24::Out :k 1)])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [anchor (:wat::rete::stratify (:wat::core::PersistentVector (:l23::ok) (:l23::neg)))
     bag    (:wat::rete::stratify (:wat::core::PersistentVector (:l23::ok) (:l23::tally)))
     nested (:wat::rete::stratify (:wat::core::PersistentVector (:l24::mkc) (:l24::nested)))]
    (:wat::core::do
      (:wat::kernel::println "ANCHOR  not-over-derived  (expect a raised stratum):")
      (:wat::kernel::println anchor)
      (:wat::kernel::println "MEASURE acc:from-over-derived (oracle):")
      (:wat::kernel::println bag)
      (:wat::kernel::println "NESTED  or(A, and(B, not(C))), C derived (oracle):")
      (:wat::kernel::println nested))))
