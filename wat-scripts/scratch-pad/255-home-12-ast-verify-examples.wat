;; wat-scripts/scratch-pad/255-home-12-ast-verify-examples.wat — arc 255 Stone HOME-12
;; rider verification: run ONLY the ten AST-surface intrinsics' @example doctests through the
;; same eval-ast!/`=` machinery :wat::doctest::verify-examples uses, WITHOUT touching the
;; pre-existing unrelated ForeignVariant `=`-comparison raise the full verify-examples run hits
;; (see the Rust probe `probe_arc255_ivb2b_verify_examples`, `#[ignore]`d, "5 failures / 1
;; cause" — a DIFFERENT, already-known defect, not this stone's). Scratch, per holon/CLAUDE.md's
;; `.wat` scratch convention.

;; Compared as STRINGS (via keyword::to-string), not bare `:wat::core::…` fqdn keywords —
;; a bare fqdn keyword that ALSO names a registered intrinsic resolves to a first-class
;; function-value reference at check time (its `[:- :->]` type), not a plain keyword literal,
;; which broke an earlier draft of this probe with 9 unrelated `:wat::core::vec` TypeMismatch
;; errors. Comparing plain strings sidesteps that resolution entirely.
(:wat::core::defn :user::mine? [ex <- :wat::intrinsic::Example] -> :wat::core::bool
  (:wat::core::let [name (:wat::keyword::to-string (:wat::intrinsic::Example/fqdn ex))]
    (:wat::core::or
      (:wat::string::starts-with? name "wat::core::ast")
      (:wat::core::or
        (:wat::core::= name "wat::core::read-string")
        (:wat::core::or
          (:wat::core::= name "wat::core::symbol-node")
          (:wat::core::or
            (:wat::core::= name "wat::core::keyword-node")
            (:wat::core::= name "wat::core::fresh-symbol")))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [all-examples (:wat::intrinsic::examples)
                     mine (:wat::core::into [] (:wat::core::filter :user::mine? all-examples))]
    (:wat::core::do
      (:wat::kernel::println (:wat::string::interpolate "MINE COUNT: {n}" :n (:wat::i64::to-string (:wat::core::length mine))))
      (:wat::core::foldl
        (:wat::core::fn [acc <- :wat::core::i64 ex <- :wat::intrinsic::Example] -> :wat::core::i64
          (:wat::core::do
            (:wat::kernel::println
              (:wat::string::interpolate "fqdn={fqdn} run={run} pure={pure} det={det}"
                :fqdn (:wat::keyword::to-string (:wat::intrinsic::Example/fqdn ex))
                :run (:wat::edn::write (:wat::intrinsic::Example/run ex))
                :pure (:wat::edn::write (:wat::intrinsic::Example/pure ex))
                :det (:wat::edn::write (:wat::intrinsic::Example/deterministic ex))))
            (:wat::core::if (:wat::intrinsic::Example/run ex)
              (:wat::core::match (:wat::intrinsic::Example/expected ex)
                ((:wat::core::Some expected-ast)
                  (:wat::core::match (:wat::eval-ast! (:wat::intrinsic::Example/expr ex))
                    ((:wat::core::Ok got)
                      (:wat::core::match (:wat::eval-ast! expected-ast)
                        ((:wat::core::Ok want)
                          (:wat::core::do
                            (:wat::kernel::println
                              (:wat::string::interpolate "  got={got} want={want} eq={eq}"
                                :got (:wat::edn::write got)
                                :want (:wat::edn::write want)
                                :eq (:wat::edn::write (:wat::core::= got want))))
                            acc))
                        ((:wat::core::Err err)
                          (:wat::core::do
                            (:wat::kernel::println (:wat::string::concat "  EXPECTED EVAL FAILED: " (:wat::core::EvalError/message err)))
                            (:wat::i64::+ acc 1)))))
                    ((:wat::core::Err err)
                      (:wat::core::do
                        (:wat::kernel::println (:wat::string::concat "  EXPR EVAL FAILED: " (:wat::core::EvalError/message err)))
                        (:wat::i64::+ acc 1)))))
                (:wat::core::None
                  (:wat::core::do (:wat::kernel::println "  (norun, no expected)") acc)))
              (:wat::core::do (:wat::kernel::println "  (norun)") acc))))
        0
        mine)
      nil)))
