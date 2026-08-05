;; What does the rete-NAMED `cond` actually EXPAND INTO?
;;
;; BRIEF-rete-cond-is-its-own-macro.md — `:wat::rete::core::cond` is now its OWN `defmacro`
;; (`wat/rete.wat`, right after `query`), not a clone of core's registered `MacroDef` under a
;; different name. STOP-3's acceptance evidence: the rete arm's expansion must contain
;; `:wat.rete.core/if` and `:wat.rete.core/cond`, and ZERO `:wat.core/if` / `:wat.core/cond`.
;; (An earlier, now-deleted `freeze/env.rs` alias loop cloned core's cond MacroDef under the
;; rete name — a clone carries core's TEMPLATE, so it emitted `:wat::core::if` regardless of
;; which name invoked it, a second door laundering back through core's spelling. This probe
;; originally caught exactly that; it now confirms the replacement is clean.)
;;
;; READ THE EXPANDED FORM FIRST (CLAUDE.md item 4). This prints it.

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [rete-expanded (:wat::core::macroexpand
                     (:wat::core::quote
                       (:wat::rete::core::cond
                         ((:wat::core::keyword::= :silver :gold)   0.5)
                         ((:wat::core::keyword::= :silver :silver) 0.7)
                         (:else                                    0.9))))
     core-expanded (:wat::core::macroexpand
                     (:wat::core::quote
                       (:wat::core::cond
                         ((:wat::core::keyword::= :silver :gold)   0.5)
                         ((:wat::core::keyword::= :silver :silver) 0.7)
                         (:else                                    0.9))))]
    (:wat::core::do
      (:wat::kernel::println "--- rete-spelled cond expands to: ---")
      (:wat::kernel::println rete-expanded)
      (:wat::kernel::println "--- core-spelled cond expands to: ---")
      (:wat::kernel::println core-expanded))))
