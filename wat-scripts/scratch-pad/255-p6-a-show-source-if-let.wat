;; arc 255 Stone P6-a — show-source on the two currently registered special forms
;; (:wat::core::if and :wat::core::let). Row 1 acceptance evidence: before the
;; #[wat_special_form_impl] annotations land, this prints P2's honest
;; "no source available in this context" line for both; after, it prints all
;; three labelled implementations (check / eval / tail).
;;
;; Scratch, per holon/CLAUDE.md's `.wat` scratch convention (not the ephemeral session tmp).

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println (:wat::core::show-source :wat::core::if))
    (:wat::kernel::println "---")
    (:wat::kernel::println (:wat::core::show-source :wat::core::let))))
