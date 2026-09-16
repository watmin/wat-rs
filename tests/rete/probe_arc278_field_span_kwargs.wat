;; strike-field-span ROW 3 — the KWARGS-FACT path to `UnknownField`.
;;
;; `validate_then_form`'s kwargs branch reported every unknown field name against `fact_span` —
;; the whole `(:fsk::Hit :nope ?k)` form — because it built `kv_pairs` (field TEXT + value node)
;; and only then checked the names, by which point the key keyword's span was gone. The caret
;; must land on `:nope`. `RhsMissingFields` rides along (the fact under-supplies `k`) and keeps
;; the fact form's span, which is correct for it: no single field is the mistake there.

(:wat::core::defrecord :fsk::Src [k <- :wat::core::i64])
(:wat::core::defrecord :fsk::Hit [k <- :wat::core::i64])

(:wat::rete::defrule :fsk::r
  :when [(:fsk::Src (?k <- :k))]
  :then [(:fsk::Hit :nope ?k)])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println "the wall refuses before main runs"))
