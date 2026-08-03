;; tests/rete/probe_construction_headline_red.wat — BRIEF-construction-inside-a-fn.md's "both
;; directions" requirement: classifying `aggregate-new`/`kwargs-construct` pure must NOT open a
;; hole. `:cr::make-rate-bad`'s body BOTH constructs a record (`(:cr::Rate2 :count n)`, the newly-
;; admitted act) AND touches a genuinely impure op (`:wat::io::IOReader/open-file`) — the compile
;; fence must still refuse it, naming the impure head, exactly as
;; `probe_arc278_then_user_forms_impure.wat` proves for a fn with NO construction in its body.

(:wat::core::defrecord :cr::In    [n <- :wat::core::i64])
(:wat::core::defrecord :cr::Rate2 [count <- :wat::core::i64])

(:wat::core::defn :cr::make-rate-bad
  [n <- :wat::core::i64]
  -> :cr::Rate2
  (:wat::core::if (:wat::core::record? (:wat::io::IOReader/open-file "x"))
    (:cr::Rate2 :count n)
    (:cr::Rate2 :count n)))

(:wat::rete::defrule :cr::compute-bad
  :when [(:cr::In (?n <- :n))]
  :then [(:cr::make-rate-bad ?n)])

;; Compiling ALONE must panic (freeze-time-only, mirroring probe_arc278_then_user_forms_impure.wat).
(:wat::core::defn :user::run-compile [] -> :wat::core::i64
  (:wat::core::let
    [rules   (:wat::rete::collect-rules :cr)
     session (:wat::rete::compile rules)]
    (:wat::core::length (:wat::rete::Session/facts session))))
