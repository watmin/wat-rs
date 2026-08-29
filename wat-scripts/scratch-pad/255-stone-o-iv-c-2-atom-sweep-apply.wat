;; wat-scripts/scratch-pad/255-stone-o-iv-c-2-atom-sweep-apply.wat — arc 255
;; Stone O-iv-c-2, acceptance rows 0/1/2. `atom.rs` has 60 handlers; this rider's own
;; disposition table (independent of the brief's 16/25/19 candidate list) found:
;;
;;   MIGRATABLE                       15  Thermometer, Blend, the nine is-*?/is-Nil?
;;                                         predicates, vector-bytes, vector-bind,
;;                                         vector-blend, statement-length
;;   ARG-SPAN (incl. from-holon)      25  reads an argument's own WatAST span
;;   BINDING (incl. 5 dual-flagged)   19  needs `sym` (require_encoding_ctx or a
;;                                         sym-taking helper), directly or via a
;;                                         delegate one level down
;;   UNEVALUATED-ARGS (`literal`)      1  needs the raw, un-evaluated WatAST form
;;                                         itself (quote semantics) — `apply` has
;;                                         already evaluated every argument to a
;;                                         Value by the time a handler sees it, so
;;                                         this is impossible for a reason none of
;;                                         the three named disqualifiers states
;;   ------------------------------------
;;   total                            60
;;
;; `literal` is this rider's one disagreement with the brief's candidate table: the
;; brief's pattern likely counted it MIGRATABLE (16, not 15) because it reads no
;; arg span and touches no `sym`/`env` beyond nothing at all (`eval_quote(args, span)`
;; takes no env/sym whatsoever) — but its whole point is to capture `args` UNEVALUATED,
;; and by the time `apply` calls any handler every argument is already a `Value`.
;; Porting it to ALGEBRA would silently make it behave like `to-holon` on a
;; pre-evaluated value instead of quoting the form — a behaviour change, forbidden by
;; STOP-3. Left as SHELL, named here.
;;
;; BEFORE this stone: every row below reports the O-iv-a diagnostic ("… is registered,
;; but no handler taking EVALUATED arguments is registered under …", kind =
;; "runtime-error"). AFTER: the 15 MIGRATED rows answer with a real value; the ARG-SPAN /
;; BINDING / UNEVALUATED-ARGS control rows still report the same diagnostic, unchanged.

(:wat::core::defn :probe::show
  [tag <- :wat::core::String r <- (:wat::core::Result :- [:wat::core::Value :wat::core::EvalError])]
  -> :wat::core::nil
  (:wat::kernel::println (:wat::string::concat tag ": " (:wat::edn::write r))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [mapv    (:wat::holon::Map (:wat::core::Vector :- [:wat::holon::HolonAST]
               (:wat::holon::Bind (:wat::holon::leaf "k") (:wat::holon::leaf "v"))))
     setv    (:wat::holon::Set (:wat::core::Vector :- [:wat::holon::HolonAST] (:wat::holon::leaf "role")))
     vecv    (:wat::holon::Vector (:wat::core::Vector :- [:wat::holon::HolonAST] (:wat::holon::leaf "role")))
     listv   (:wat::holon::List (:wat::core::Vector :- [:wat::holon::HolonAST] (:wat::holon::leaf "role")))
     tuplev  (:wat::holon::Tuple (:wat::core::Vector :- [:wat::holon::HolonAST] (:wat::holon::leaf "role")))
     symv    (:wat::holon::from-wat (:wat::core::quote x))
     kwv     (:wat::holon::from-wat (:wat::core::quote :k))
     leafv   (:wat::holon::leaf "role")
     nilv    (:wat::holon::to-holon nil)
     rolev   (:wat::holon::leaf "role")
     fillerv (:wat::holon::leaf "filler")
     enc1    (:wat::holon::encode (:wat::holon::leaf "role"))
     enc2    (:wat::holon::encode (:wat::holon::leaf "filler"))
     bindv   (:wat::holon::Bind (:wat::holon::leaf "role") (:wat::holon::leaf "filler"))]
    (:wat::core::do
      ;; ── MIGRATABLE, 15 — must answer through apply ──
      (:probe::show "is-Map? (true)"
        (:wat::eval-ast! (:wat::core::quote
          (:wat::core::apply :wat::holon::is-Map? (:wat::core::Vector :- [:wat::core::Any] mapv)))))
      (:probe::show "is-Map? (false)"
        (:wat::eval-ast! (:wat::core::quote
          (:wat::core::apply :wat::holon::is-Map? (:wat::core::Vector :- [:wat::core::Any] setv)))))
      (:probe::show "is-Set? (true)"
        (:wat::eval-ast! (:wat::core::quote
          (:wat::core::apply :wat::holon::is-Set? (:wat::core::Vector :- [:wat::core::Any] setv)))))
      (:probe::show "is-Vector? (true)"
        (:wat::eval-ast! (:wat::core::quote
          (:wat::core::apply :wat::holon::is-Vector? (:wat::core::Vector :- [:wat::core::Any] vecv)))))
      (:probe::show "is-List? (true)"
        (:wat::eval-ast! (:wat::core::quote
          (:wat::core::apply :wat::holon::is-List? (:wat::core::Vector :- [:wat::core::Any] listv)))))
      (:probe::show "is-Tuple? (true)"
        (:wat::eval-ast! (:wat::core::quote
          (:wat::core::apply :wat::holon::is-Tuple? (:wat::core::Vector :- [:wat::core::Any] tuplev)))))
      (:probe::show "is-Symbol? (true)"
        (:wat::eval-ast! (:wat::core::quote
          (:wat::core::apply :wat::holon::is-Symbol? (:wat::core::Vector :- [:wat::core::Any] symv)))))
      (:probe::show "is-Keyword? (true)"
        (:wat::eval-ast! (:wat::core::quote
          (:wat::core::apply :wat::holon::is-Keyword? (:wat::core::Vector :- [:wat::core::Any] kwv)))))
      (:probe::show "is-Tag? (false)"
        (:wat::eval-ast! (:wat::core::quote
          (:wat::core::apply :wat::holon::is-Tag? (:wat::core::Vector :- [:wat::core::Any] leafv)))))
      (:probe::show "is-Nil? (true)"
        (:wat::eval-ast! (:wat::core::quote
          (:wat::core::apply :wat::holon::is-Nil? (:wat::core::Vector :- [:wat::core::Any] nilv)))))
      (:probe::show "is-Nil? (false)"
        (:wat::eval-ast! (:wat::core::quote
          (:wat::core::apply :wat::holon::is-Nil? (:wat::core::Vector :- [:wat::core::Any] leafv)))))
      (:probe::show "Thermometer"
        (:wat::eval-ast! (:wat::core::quote
          (:wat::core::apply :wat::holon::Thermometer (:wat::core::Vector :- [:wat::core::Any] 5.0 0.0 10.0)))))
      ;; Blend's Ok value is a raw composite HolonAST, which `:wat::edn::write`
      ;; refuses to cross the wire (DESIGN-STONE-294.j — only DATA and the two
      ;; directives, Thermometer/SlotMarker, do). Prove dispatch by chaining a
      ;; direct `statement-length` (2 for a Blend node) onto the unwrapped value.
      (:probe::show "Blend (statement-length 2 proves the Ok value is a real Blend)"
        (:wat::core::match
          (:wat::eval-ast! (:wat::core::quote
            (:wat::core::apply :wat::holon::Blend (:wat::core::Vector :- [:wat::core::Any] rolev fillerv 0.7 0.3))))
          ((:wat::core::Ok v) (:wat::core::Ok (:wat::holon::statement-length v)))
          ((:wat::core::Err e) (:wat::core::Err e))))
      (:probe::show "vector-bytes"
        (:wat::eval-ast! (:wat::core::quote
          (:wat::core::apply :wat::holon::vector-bytes (:wat::core::Vector :- [:wat::core::Any] enc1)))))
      (:probe::show "vector-bind"
        (:wat::eval-ast! (:wat::core::quote
          (:wat::core::apply :wat::holon::vector-bind (:wat::core::Vector :- [:wat::core::Any] enc1 enc2)))))
      (:probe::show "vector-blend"
        (:wat::eval-ast! (:wat::core::quote
          (:wat::core::apply :wat::holon::vector-blend (:wat::core::Vector :- [:wat::core::Any] enc1 enc2 0.6 0.4)))))
      (:probe::show "statement-length"
        (:wat::eval-ast! (:wat::core::quote
          (:wat::core::apply :wat::holon::statement-length (:wat::core::Vector :- [:wat::core::Any] bindv)))))

      ;; ── controls: refused verbs still report the O-iv-a diagnostic, unchanged ──
      (:probe::show "STILL-ARG-SPAN Atom"
        (:wat::eval-ast! (:wat::core::quote
          (:wat::core::apply :wat::holon::Atom (:wat::core::Vector :- [:wat::core::Any] leafv)))))
      (:probe::show "STILL-ARG-SPAN leaf"
        (:wat::eval-ast! (:wat::core::quote
          (:wat::core::apply :wat::holon::leaf (:wat::core::Vector :- [:wat::core::Any] "role")))))
      (:probe::show "STILL-ARG-SPAN Bind"
        (:wat::eval-ast! (:wat::core::quote
          (:wat::core::apply :wat::holon::Bind (:wat::core::Vector :- [:wat::core::Any] rolev fillerv)))))
      (:probe::show "STILL-ARG-SPAN is?"
        (:wat::eval-ast! (:wat::core::quote
          (:wat::core::apply :wat::holon::is? (:wat::core::Vector :- [:wat::core::Any] leafv "Vector")))))
      (:probe::show "STILL-ARG-SPAN vector-permute"
        (:wat::eval-ast! (:wat::core::quote
          (:wat::core::apply :wat::holon::vector-permute (:wat::core::Vector :- [:wat::core::Any] enc1 1)))))
      (:probe::show "STILL-ARG-SPAN vector-bundle"
        (:wat::eval-ast! (:wat::core::quote
          (:wat::core::apply :wat::holon::vector-bundle (:wat::core::Vector :- [:wat::core::Any] (:wat::core::Vector :- [:wat::holon::Vector] enc1 enc2))))))
      (:probe::show "STILL-BINDING cosine"
        (:wat::eval-ast! (:wat::core::quote
          (:wat::core::apply :wat::holon::cosine (:wat::core::Vector :- [:wat::core::Any] rolev rolev)))))
      (:probe::show "STILL-BINDING encode"
        (:wat::eval-ast! (:wat::core::quote
          (:wat::core::apply :wat::holon::encode (:wat::core::Vector :- [:wat::core::Any] rolev)))))
      (:probe::show "STILL-BINDING Bundle"
        (:wat::eval-ast! (:wat::core::quote
          (:wat::core::apply :wat::holon::Bundle (:wat::core::Vector :- [:wat::core::Any] (:wat::core::Vector :- [:wat::holon::HolonAST] rolev fillerv))))))
      (:probe::show "STILL-BINDING presence-floor"
        (:wat::eval-ast! (:wat::core::quote
          (:wat::core::apply :wat::holon::presence-floor (:wat::core::Vector :- [:wat::core::Any] 4096)))))
      (:probe::show "STILL-BINDING eval-coincident?"
        (:wat::eval-ast! (:wat::core::quote
          (:wat::core::apply :wat::holon::eval-coincident? (:wat::core::Vector :- [:wat::core::Any] 1 2)))))
      (:probe::show "STILL-BINDING+ARG-SPAN term::matches?"
        (:wat::eval-ast! (:wat::core::quote
          (:wat::core::apply :wat::holon::term::matches? (:wat::core::Vector :- [:wat::core::Any] rolev fillerv)))))
      (:probe::show "STILL-UNEVALUATED-ARGS literal"
        (:wat::eval-ast! (:wat::core::quote
          (:wat::core::apply :wat::holon::literal (:wat::core::Vector :- [:wat::core::Any] 1)))))
      (:probe::show "STILL-OUT-OF-SCOPE from-holon (STOP-1)"
        (:wat::eval-ast! (:wat::core::quote
          (:wat::core::apply :wat::holon::from-holon (:wat::core::Vector :- [:wat::core::Any] bindv))))))))
