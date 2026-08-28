;; Scratch probe (circumspicere recon, read-only) — `show-source` on a
;; registered special form. `IntrinsicEntry::source` is the empty-string
;; sentinel for Kind::SpecialForm (src/intrinsic/mod.rs:419
;; `source: "",`); `eval_show_source` (src/intrinsic/reflect.rs:238-240)
;; checks the registry FIRST and returns `entry.source` verbatim on any
;; hit — does a registered special form silently get an empty string
;; instead of the "no source available" message its own fallback path
;; (reflect.rs ~264-268) would otherwise give a Binding::SpecialForm?
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println
    (:wat::string::concat "show-source(:wat::core::if) = <<"
      (:wat::core::show-source :wat::core::if) ">>")))
