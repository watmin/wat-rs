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

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [anchor (:wat::rete::stratify (:wat::core::PersistentVector (:l23::ok) (:l23::neg)))
     bag    (:wat::rete::stratify (:wat::core::PersistentVector (:l23::ok) (:l23::tally)))]
    (:wat::core::do
      (:wat::kernel::println "ANCHOR  not-over-derived  (expect a raised stratum):")
      (:wat::kernel::println anchor)
      (:wat::kernel::println "MEASURE acc:from-over-derived (oracle):")
      (:wat::kernel::println bag))))
