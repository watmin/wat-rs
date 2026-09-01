;; strike-field-span — the BIND-CLAUSE path to `UnknownField`.
;;
;; `(?b <- :nofield)` classifies as `ReteClauseShape::Bind`. The classifier held the `:nofield`
;; KEYWORD NODE all along and dropped it (`keyword_payload(&items[2])` keeps only the text), so
;; the wall could only locate this at the whole `(?b <- :nofield)` clause. `Bind` now carries
;; `field_kw` and the caret lands on `:nofield`.

(:wat::core::defrecord :fsb::Src [k <- :wat::core::i64])
(:wat::core::defrecord :fsb::Hit [k <- :wat::core::i64])

(:wat::rete::defrule :fsb::r
  :when [(:fsb::Src (?k <- :k) (?b <- :nofield))]
  :then [(:fsb::Hit :k ?k)])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println "the wall refuses before main runs"))
