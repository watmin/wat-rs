;; wat-scripts/scratch-pad/probe-stone-e-iv-keyword-new-home.wat — arc 255 Stone E-iv.
;; Acceptance row 1: all 5 `:wat::keyword::*` verbs RUN (not merely check) under the new
;; spelling — a scratch-pad probe asserting a result for each.

(:wat::core::defn :probe::check [ok <- :wat::core::bool msg <- :wat::core::String] -> :wat::core::nil
  (:wat::core::if ok
    nil
    (:wat::kernel::assertion-failed! msg :wat::core::None :wat::core::None)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:probe::check (:wat::core::= (:wat::keyword::to-string :foo) "foo") "to-string")
    (:probe::check (:wat::core::= (:wat::keyword::from-string "foo") :foo) "from-string")
    (:probe::check
      (:wat::core::= (:wat::core::ast-kind (:wat::keyword::to-symbol (:wat::core::keyword-node ":wat::core::Bytes::to-hex")))
                      "symbol")
      "to-symbol")
    (:probe::check
      (:wat::core::= (:wat::core::ast-kind (:wat::keyword::to-type-form (:wat::core::keyword-node ":wat::core::i64")))
                      "symbol")
      "to-type-form")
    (:probe::check
      (:wat::core::= (:wat::core::ast-kind (:wat::keyword::to-type-form-colon (:wat::core::keyword-node ":wat::core::i64")))
                      "keyword")
      "to-type-form-colon")
    (:wat::kernel::println "PROBE-E-IV-NEW-HOME-OK")))
