;; Arc 255 Stone P5-b — TWO things, and the second one is a REPRODUCTION, not a convenience.
;;
;; 1. THE READING P5-b's RIDER COULD NOT TAKE. `:wat::kernel::spawn-thread`/`spawn-process` carry
;;    `{:restricted-to [:wat::kernel::]}`, and `check.rs`'s `walk_for_restricted_call` fires on a
;;    MENTION in ANY position — so their FQDNs cannot appear inside a `:user::` fn body, not even
;;    as `render-doc`'s argument. The rider mirrored `reflect.rs`'s rendering loop in a throwaway
;;    Rust test instead, and said so honestly. This reads both through the REAL `eval_render_doc`,
;;    and shows P5-b's whole point: TWO `@yields` on one verb, which the old singleton forbade.
;;
;; 2. ⛔ IT DOES SO BY EXPLOITING A HOLE IN THE CAPABILITY WALL, and that is why the file is kept.
;;    The restriction attributes a call to its ENCLOSING FN. A TOP-LEVEL form has NO enclosing fn,
;;    so the check is SKIPPED rather than FAILED — absence of a caller reads as "no restriction
;;    applies" instead of "no caller is authorized". Bind the restricted keyword at top level and
;;    it is laundered into any `:user::` fn:
;;
;;      (:wat::core::def :user::smuggled :wat::kernel::flood-stdout-raw)
;;      (:wat::core::defn :user::main [] -> :wat::core::nil
;;        (:wat::kernel::println (:wat::core::apply :user::smuggled ["LAUNDERED"])))
;;      ;; => RUNS. Writes to fd 1. The SAME call written directly in main is refused at check time
;;      ;;    with DefRestrictedCallerNotAllowed. Measured 2026-08-28.
;;
;;    `wat/spawn.wat:333` shows the adjacent case WAS considered — a macro-spliced call attributes
;;    to the EXPANSION SITE, named there as "capability laundering" and closed. The top-level case
;;    was not, because top level does not look like a call site.
;;
;; ★ THIS FILE IS A CANARY. It is loader-gated (`every_wat_scripts_file_loads`), so the day the hole
;;   is closed it goes RED — and whoever closes it must come here and see that CLOSING THE HOLE
;;   BREAKS DOC TOOLING OVER RESTRICTED VERBS, because the hoist above is the only route a wat
;;   program has to reflect on one. That is the argument FOR a sanctioned reflective path
;;   (option B), not against closing the hole. See
;;   NOTE-restricted-call-fires-on-mention-not-call.md.
(:wat::core::def :user::thread-doc (:wat::core::render-doc :wat::kernel::spawn-thread))
(:wat::core::def :user::process-doc (:wat::core::render-doc :wat::kernel::spawn-process))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println :user::thread-doc)
  (:wat::kernel::println :user::process-doc))
