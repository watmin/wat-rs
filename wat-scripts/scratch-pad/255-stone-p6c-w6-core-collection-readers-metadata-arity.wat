;; wat-scripts/scratch-pad/255-stone-p6c-w6-core-collection-readers-metadata-arity.wat — arc 255
;; Stone P6-c-W6. Rider recon ONLY — no homing performed this wave (STOP-1 fired on
;; `:wat::core::find-last-index`, see the rider's report). Dumps `(metadata-of <fqdn>)` for the
;; eight collection readers named in
;; docs/arc/2026/06/255-builtin-registry/BRIEF-STONE-P6-c-W6-core-collection-readers.md, and a
;; direct call per verb on a NON-trivial collection, so the "before" numbers are measured (not
;; asserted) even though no `src/` edit happened. Scratch, per holon/CLAUDE.md's `.wat` scratch
;; convention. `--check` clean; loadable by `every_wat_scripts_file_loads`.

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println "── :wat::core::length ──")
    (:wat::kernel::pprintln (:wat::runtime::metadata-of :wat::core::length))
    (:wat::kernel::println "── :wat::core::empty? ──")
    (:wat::kernel::pprintln (:wat::runtime::metadata-of :wat::core::empty?))
    (:wat::kernel::println "── :wat::core::last ──")
    (:wat::kernel::pprintln (:wat::runtime::metadata-of :wat::core::last))
    (:wat::kernel::println "── :wat::core::rest ──")
    (:wat::kernel::pprintln (:wat::runtime::metadata-of :wat::core::rest))
    (:wat::kernel::println "── :wat::core::nth ──")
    (:wat::kernel::pprintln (:wat::runtime::metadata-of :wat::core::nth))
    (:wat::kernel::println "── :wat::core::reverse ──")
    (:wat::kernel::pprintln (:wat::runtime::metadata-of :wat::core::reverse))
    (:wat::kernel::println "── :wat::core::find-last-index ──")
    (:wat::kernel::pprintln (:wat::runtime::metadata-of :wat::core::find-last-index))
    (:wat::kernel::println "── :wat::core::range ──")
    (:wat::kernel::pprintln (:wat::runtime::metadata-of :wat::core::range))

    (:wat::kernel::println "── direct calls, non-trivial collection ──")
    (:wat::kernel::pprintln (:wat::core::length (:wat::core::Vector :- [:wat::core::i64] 1 2 3 4 5)))
    (:wat::kernel::pprintln (:wat::core::empty? (:wat::core::Vector :- [:wat::core::i64] 1 2 3 4 5)))
    (:wat::kernel::pprintln (:wat::core::last (:wat::core::Vector :- [:wat::core::i64] 1 2 3 4 5)))
    (:wat::kernel::pprintln (:wat::core::rest (:wat::core::Vector :- [:wat::core::i64] 1 2 3 4 5)))
    (:wat::kernel::pprintln (:wat::core::nth (:wat::core::Vector :- [:wat::core::i64] 1 2 3 4 5) 2))
    (:wat::kernel::pprintln (:wat::core::reverse (:wat::core::Vector :- [:wat::core::i64] 1 2 3 4 5)))
    (:wat::kernel::pprintln
      (:wat::core::find-last-index (:wat::core::Vector :- [:wat::core::i64] 1 2 3 4 5)
        (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::bool (:wat::i64::< x 4))))
    (:wat::kernel::pprintln (:wat::core::range 2 7))))
