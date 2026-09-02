;; Arc 255 Stone 1a-β-ii — the orchestrator's INDEPENDENT weigh of the kill.
;;
;; `freeze::is_liftable_declaration_head` is DELETED. Its nine-name hand-list is now a
;; registry query: does this row name a `SpecialFormRole::Declare` implementation?
;;
;; The rider confirmed MISSING was empty by reading the source, not by running the meter
;; — and then deleted the meter, so the number cannot be re-run. This reads the same fact
;; back out of the LIVE registry instead: every one of the eight names the predicate held
;; must render a `;; role: declare` block, and a control that never had one must not.
(:wat::core::def :scratch::declares?
  (:wat::core::fn [n <- :wat::core::keyword] -> :wat::core::bool
    (:wat::string::contains? (:wat::core::show-source n) "role: declare")))

(:wat::core::def :user::main
  (:wat::core::fn [] -> :wat::core::nil
    (:wat::kernel::println (:scratch::declares? :wat::core::def))
    (:wat::kernel::println (:scratch::declares? :wat::core::defalias))
    (:wat::kernel::println (:scratch::declares? :wat::core::defmacro))
    (:wat::kernel::println (:scratch::declares? :wat::core::defenum))
    (:wat::kernel::println (:scratch::declares? :wat::core::newtype))
    (:wat::kernel::println (:scratch::declares? :wat::core::structtype))
    (:wat::kernel::println (:scratch::declares? :wat::core::typealias))
    (:wat::kernel::println (:scratch::declares? :wat::core::defsurface))
    ;; ⛔ the controls — neither ever belonged to the predicate's domain
    (:wat::kernel::println (:scratch::declares? :wat::core::if))
    (:wat::kernel::println (:scratch::declares? :wat::string::declare-acronyms))))
