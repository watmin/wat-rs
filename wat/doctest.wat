;; wat/doctest.wat — doctest types and verifier surface (arc 255.1b-iv-b2).
;;
;; Arc 255 Stone iv-b2-a — defines :wat::intrinsic::Example, the typed record
;; returned by the `:wat::intrinsic::examples` reflection seam. Records (not
;; heterogeneous tuples) so `verify-examples` (iv-b2-b) can field-access typed
;; values and pass `expr`/`expected` to `:wat::eval-ast!` without a down-cast.
;;
;; Load order: after Record.wat (uses :wat::core::Record::def), core.wat (keyword/bool),
;; and the holon/*.wat files (no additional deps beyond those). The seam that
;; RETURNS these records (:wat::intrinsic::examples) is a Rust intrinsic and does
;; not need the record type at registration time — only at call time.

(:wat::core::defrecord :wat::intrinsic::Example
  [fqdn          <- :wat::core::keyword
   expr          <- :wat::WatAST
   expected      <- (:wat::core::Option :- [:wat::WatAST])
   run           <- :wat::core::bool
   pure          <- :wat::core::bool
   deterministic <- :wat::core::bool])

;; ─── Doctest failure record ───────────────────────────────────────────

(:wat::core::defrecord :wat::doctest::Failure
  [fqdn   <- :wat::core::keyword
   reason <- :wat::core::String])

;; ─── verify-examples — the self-hosting doctest runner ───────────────
;;
;; Folds over (:wat::intrinsic::examples) — the iv-b2-a reflection seam.
;; For each Example whose run=true:
;;   1. Cross-check: intrinsic must be pure∧deterministic (the @example
;;      marker guarantees this; a mismatch is a Failure).
;;   2. Doctest: eval expr and expected via :wat::eval-ast!, compare with
;;      :wat::core::=; a mismatch is a Failure.
;; run=false examples (@example-norun) are skipped.
;; Returns (Vector :- [:wat::doctest::Failure]) — empty means all doctests passed.

(:wat::core::defn :wat::doctest::verify-examples
  []
  -> (:wat::core::Vector :- [:wat::doctest::Failure])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::doctest::Failure])
                     ex  <- :wat::intrinsic::Example]
      -> (:wat::core::Vector :- [:wat::doctest::Failure])
      ;; The Example values are Value::wat__Record (the seam builds the
      ;; :wat::core::Record::def representation), so the generated named accessors
      ;; :wat::intrinsic::Example/<field> work directly — no positional indexing.
      (:wat::core::if (:wat::intrinsic::Example/run ex)
        ;; run=true: cross-check purity, then run the doctest
        (:wat::core::let [acc1 (:wat::core::if (:wat::core::not
                                                   (:wat::core::and
                                                     (:wat::intrinsic::Example/pure ex)
                                                     (:wat::intrinsic::Example/deterministic ex)))
                                  (:wat::core::concat acc
                                    (:wat::core::Vector :- [:wat::doctest::Failure]
                                      (:wat::doctest::Failure
                                        :fqdn (:wat::intrinsic::Example/fqdn ex)
                                        :reason "doctested @example on a non-pure∧deterministic intrinsic")))
                                  acc)
                          fqdn (:wat::intrinsic::Example/fqdn ex)]
          (:wat::core::match (:wat::intrinsic::Example/expected ex)
            ((:wat::core::Some expected-ast)
              (:wat::core::match (:wat::eval-ast! (:wat::intrinsic::Example/expr ex))
                ((:wat::core::Ok got)
                  (:wat::core::match (:wat::eval-ast! expected-ast)
                    ((:wat::core::Ok want)
                      (:wat::core::if (:wat::core::not (:wat::core::= got want))
                        (:wat::core::concat acc1
                          (:wat::core::Vector :- [:wat::doctest::Failure]
                            (:wat::doctest::Failure
                              :fqdn fqdn
                              :reason "@example result did not match #=>")))
                        acc1))
                    ((:wat::core::Err err)
                      (:wat::core::concat acc1
                        (:wat::core::Vector :- [:wat::doctest::Failure]
                          (:wat::doctest::Failure
                            :fqdn fqdn
                            :reason (:wat::string::concat
                                      "expected eval failed: "
                                      (:wat::core::EvalError/message err))))))))
                ((:wat::core::Err err)
                  (:wat::core::concat acc1
                    (:wat::core::Vector :- [:wat::doctest::Failure]
                      (:wat::doctest::Failure
                        :fqdn fqdn
                        :reason (:wat::string::concat
                                  "expr eval failed: "
                                  (:wat::core::EvalError/message err))))))))
            (:wat::core::None
              (:wat::core::concat acc1
                (:wat::core::Vector :- [:wat::doctest::Failure]
                  (:wat::doctest::Failure
                    :fqdn fqdn
                    :reason "run=true example missing expected"))))))
        ;; run=false: skip
        acc))
    (:wat::core::Vector :- [:wat::doctest::Failure])
    (:wat::intrinsic::examples)))
