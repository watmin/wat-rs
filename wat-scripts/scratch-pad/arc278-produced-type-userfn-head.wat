;; arc 278 — DOES `produced` DIVERGE WHEN THE `:then` HEAD IS A USER FN?
;;
;; `src/rete/kernel/stratify.rs`'s `produced_type` resolves the head through the SymbolTable:
;; if it names a fn whose return type is a non-`wat::core::` Path, it returns THAT RETURN TYPE.
;; The oracle's `rule-produces` (`wat/rete/oracle/stratify.wat:46-67`) takes the first child of
;; the RHS form, reads its name, strips a leading colon — no symbol table, no resolution.
;;
;; A user-fn head in `:then` is a SHIPPED form: `tests/rete/probe_arc278_then_user_forms_userfn.wat`
;; drives `:then [(:tf::first-rate ?rates)]` where `:tf::first-rate` returns `:tf::Rate`.
;;
;; `produced` is where the sweep ASSIGNS a stratum. If the oracle assigns to the FN NAME instead
;; of the fact type, the fact type is never raised, and a consumer of it computes `req-pos` from
;; 0 — it can be stratified BELOW its own input.
;;
;; ⛔ ORACLE HALF ONLY. `:wat::rete::rule-produces` has one definition — the oracle's. The native
;; extractor is `pub(crate)` Rust. Do not read agreement into this file.
;;
;; ANCHOR: `plain` has an ordinary fact-type head and MUST come back as the type name on both
;; sides. If the anchor is wrong the extractor is broken, not divergent.
;;
;; Run (NOT the installed binary — wat/ is include_str!'d):
;;   cargo run --release --bin wat -- wat-scripts/scratch-pad/arc278-produced-type-userfn-head.wat

(:wat::core::defrecord :pt::Anchor [x <- :wat::core::i64])
(:wat::core::defrecord :pt::Rate   [count <- :wat::core::i64])

(:wat::rete::core::defn :pt::first-rate
  [rs <- (:wat::core::PersistentVector :- [:pt::Rate])]
  -> :pt::Rate
  (:wat::rete::core::PersistentVector/first rs :undefined (:pt::Rate :count 0)))

;; THE MEASUREMENT — the `:then` head is a user fn returning :pt::Rate.
(:wat::rete::defrule :pt::via-userfn
  :when [(:pt::Anchor (?x <- :x))
         (?rates <- (:wat::rete::acc::all) :from (:pt::Rate (?c <- :count)))]
  :then [(:pt::first-rate ?rates)])

;; THE ANCHOR — an ordinary fact-type head. Both sides must say "pt::Rate".
(:wat::rete::defrule :pt::plain
  :when [(:pt::Anchor (?x <- :x))]
  :then [(:pt::Rate :count ?x)])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println "ANCHOR  plain fact-type head — oracle rule-produces:")
    (:wat::kernel::println (:wat::rete::rule-produces (:pt::plain)))
    (:wat::kernel::println "MEASURE user-fn head (native resolves to pt::Rate) — oracle rule-produces:")
    (:wat::kernel::println (:wat::rete::rule-produces (:pt::via-userfn)))))
