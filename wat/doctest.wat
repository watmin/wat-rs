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

;; ─── Row — the enumerable registry census record ──────────────────────
;;
;; Arc 255 STONE "the registry can be enumerated" — the typed record returned by
;; the `:wat::intrinsic::rows` reflection seam. `metadata-of` answers per-name;
;; `(:wat::intrinsic::rows)` answers per-SET, one Row per registered entry, so a
;; wat program can run a census (`filter`/`count` over kind/totality/etc) — the
;; same four-site pattern `:wat::intrinsic::Example` (immediately above) already
;; shipped once: a wat-side defrecord, a checker scheme, a load-order position,
;; and a `#[wat_intrinsic]` walking `all_entries()`.
;;
;; Field list is DESIGN's, exactly — the exclusions are load-bearing, not an
;; oversight: no `doc`/`prose`/`ret`(description)/`source`/`examples` (552 rows
;; of prose in one value is why `metadata-of` and `:wat::intrinsic::examples`
;; serve those per-name/via their own seam instead). `arity` uses `-1` for
;; `Variadic`, `metadata-of`'s existing sentinel — not a second convention.
;; `syntax` is `""` when absent (matches `IntrinsicEntry.syntax`'s own
;; `&'static str` shape), not wrapped in an `Option`.
;;
;; The five closed-domain axis fields (`kind`/`purity`/`determinism`/
;; `totality`/`expand-time`/`category`) reference the enums `wat/runtime-meta.wat`
;; declares — this file loads well after that one (`src/load/stdlib.rs`), so
;; every axis type is already registered. Row itself lives HERE, not in
;; runtime-meta.wat: `defrecord`'s expansion calls `:wat::core::Record::def` at
;; EVAL time, which requires `wat/Record.wat` to have already loaded —
;; runtime-meta.wat loads BEFORE Record.wat (its own header: "no eval-deps
;; beyond :wat::core::defenum"), so a `defrecord` placed there would break that
;; ordering. `doctest.wat` already loads after Record.wat (see this file's own
;; header, immediately above) and after runtime-meta.wat, so it is the correct
;; home for the composition, mirroring `wat/program.wat`'s `Env` record (a
;; `defenum` + a `defrecord` referencing it, same-file precedent for this exact
;; shape).
;;
;; Load order: after Record.wat + runtime-meta.wat (both required — see above).
;; The seam that RETURNS these records (:wat::intrinsic::rows) is a Rust
;; intrinsic and does not need the record type at registration time — only at
;; call time, same as Example/examples.

(:wat::core::defrecord :wat::intrinsic::Row
  [name          <- :wat::core::keyword
   kind          <- :wat::runtime::Kind
   arity         <- :wat::core::i64
   purity        <- :wat::runtime::Purity
   determinism   <- :wat::runtime::Determinism
   totality      <- :wat::runtime::Totality
   expand-time   <- :wat::runtime::ExpandTime
   category      <- :wat::runtime::Category
   syntax        <- :wat::core::String
   ret-type      <- :wat::core::String
   alias-of      <- (:wat::core::Option :- [:wat::core::String])
   has-handler   <- :wat::core::bool])

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
