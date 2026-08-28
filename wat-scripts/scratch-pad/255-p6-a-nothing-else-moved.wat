;; arc 255 Stone P6-a — Row 3 acceptance evidence: NOTHING ELSE MOVED.
;; metadata-of and render-doc for :wat::core::if and :wat::core::let must be
;; byte-identical before/after (P2 set :arity 3 / -1 and the "Syntax:" line;
;; both must survive). An INTRINSIC's show-source (:wat::i64::+) must also be
;; byte-identical — this stone touches special forms only, never intrinsics.
;;
;; Scratch, per holon/CLAUDE.md's `.wat` scratch convention (not the ephemeral session tmp).

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println "== metadata-of if ==")
    (:wat::kernel::pprintln (:wat::runtime::metadata-of :wat::core::if))
    (:wat::kernel::println "== metadata-of let ==")
    (:wat::kernel::pprintln (:wat::runtime::metadata-of :wat::core::let))
    (:wat::kernel::println "== render-doc if ==")
    (:wat::kernel::println (:wat::core::render-doc :wat::core::if))
    (:wat::kernel::println "== render-doc let ==")
    (:wat::kernel::println (:wat::core::render-doc :wat::core::let))
    (:wat::kernel::println "== show-source i64::+ (intrinsic, unaffected) ==")
    (:wat::kernel::println (:wat::core::show-source :wat::i64::+))))
