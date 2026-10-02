;; wat-scripts/scratch-pad/255-stone-o-iv-d-remainder-apply.wat — arc 255 Stone O-iv-d,
;; acceptance rows 1-3. Drives the 12 verbs this rider migrated to ALGEBRA through
;; `:wat::core::apply` (the value door), via `:wat::eval-ast!` on a `quote`d form so the
;; static type checker never runs on the heterogeneous args-vector — same pattern
;; `255-stone-o-iv-c-1-holon-sweep-apply.wat` uses.
;;
;; BEFORE this stone: every row below reports the O-iv-a diagnostic
;; ("… is registered, but no handler taking EVALUATED arguments is registered under …").
;; AFTER: all 12 answer with a real value.
;;
;; `:wat::string::declare-acronyms` and `:wat::intrinsic::variadic-args-measurement` are
;; DELIBERATELY EXCLUDED — both refused (UNEVALUATED-ARGS; see
;; `255-stone-o-iv-d-unevaluated-args-check.wat` and
;; `255-stone-o-iv-d-declare-acronyms-unevaluated-check.wat` for the empirical proof).

(:wat::core::defn :probe::show
  [tag <- wat.type/String r <- (:wat::core::Result :- [wat.type/Value :wat::core::EvalError])]
  -> wat.type/nil
  (:wat::kernel::println (:wat::string::concat tag ": " (:wat::edn::write r))))

(:wat::core::defn :user::main [] -> wat.type/nil
  (:wat::core::do
    ;; row 2 — the variadic splat, prove 3 elements not just dispatch
    (:probe::show "core::List [1 2 3]"
      (:wat::eval-ast! (:wat::core::quote
        (:wat::core::apply wat.type/List (wat.type/Vector :- [:wat::core::Any] 1 2 3)))))
    (:probe::show "core::List []"
      (:wat::eval-ast! (:wat::core::quote
        (:wat::core::apply wat.type/List (wat.type/Vector :- [:wat::core::Any])))))

    ;; row 3 — the 0-arg door
    (:probe::show "math::pi"
      (:wat::eval-ast! (:wat::core::quote
        (:wat::core::apply :wat::math::pi (wat.type/Vector :- [:wat::core::Any])))))
    (:probe::show "uuid::v4"
      (:wat::eval-ast! (:wat::core::quote
        (:wat::core::apply :wat::uuid::v4 (wat.type/Vector :- [:wat::core::Any])))))
    (:probe::show "uuid::nil"
      (:wat::eval-ast! (:wat::core::quote
        (:wat::core::apply :wat::uuid::nil (wat.type/Vector :- [:wat::core::Any])))))
    (:probe::show "time::now"
      (:wat::eval-ast! (:wat::core::quote
        (:wat::core::apply :wat::time::now (wat.type/Vector :- [:wat::core::Any])))))
    (:probe::show "kernel::stopped?"
      (:wat::eval-ast! (:wat::core::quote
        (:wat::core::apply :wat::kernel::stopped? (wat.type/Vector :- [:wat::core::Any])))))
    (:probe::show "kernel::sigusr1?"
      (:wat::eval-ast! (:wat::core::quote
        (:wat::core::apply :wat::kernel::sigusr1? (wat.type/Vector :- [:wat::core::Any])))))
    (:probe::show "kernel::sigusr2?"
      (:wat::eval-ast! (:wat::core::quote
        (:wat::core::apply :wat::kernel::sigusr2? (wat.type/Vector :- [:wat::core::Any])))))
    (:probe::show "kernel::sighup?"
      (:wat::eval-ast! (:wat::core::quote
        (:wat::core::apply :wat::kernel::sighup? (wat.type/Vector :- [:wat::core::Any])))))
    ;; effectful 0-arg verbs (mutate global signal flags) — dispatched LAST so the readers
    ;; above see the pre-reset state
    (:probe::show "kernel::reset-sigusr1!"
      (:wat::eval-ast! (:wat::core::quote
        (:wat::core::apply :wat::kernel::reset-sigusr1! (wat.type/Vector :- [:wat::core::Any])))))
    (:probe::show "kernel::reset-sigusr2!"
      (:wat::eval-ast! (:wat::core::quote
        (:wat::core::apply :wat::kernel::reset-sigusr2! (wat.type/Vector :- [:wat::core::Any])))))
    (:probe::show "kernel::reset-sighup!"
      (:wat::eval-ast! (:wat::core::quote
        (:wat::core::apply :wat::kernel::reset-sighup! (wat.type/Vector :- [:wat::core::Any])))))))
