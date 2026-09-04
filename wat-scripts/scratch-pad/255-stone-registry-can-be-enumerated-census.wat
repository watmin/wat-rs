;; Scratch probe — arc 255 STONE "the registry can be enumerated".
;;
;; Acceptance censuses (BRIEF's four rows), run from wat against the freshly-added
;; `(:wat::intrinsic::rows)` seam — the set-level sibling of `(:wat::intrinsic::examples)`.
;; Every number below is printed, never asserted-and-swallowed, so the rider's report can
;; quote it and cross-check row 1/2 against `probe_can_doc_types_reconstruct_the_checker_scheme`'s
;; `total registry rows` (same `all_entries()`, read through a completely different door).
;;
;; ⚠ This probe's own row IS a registered entry (`:wat::intrinsic::rows` itself), so its own
;; census total counts itself — the rider's report says explicitly which total (BEFORE/AFTER
;; this stone) it is reporting, per the EXPECTATIONS doc's ledger note.

(:wat::core::defn :user::is-special-form? [r <- :wat::intrinsic::Row] -> :wat::core::bool
  (:wat::core::= (:wat::intrinsic::Row/kind r) :wat::runtime::Kind::SpecialForm))

(:wat::core::defn :user::is-intrinsic? [r <- :wat::intrinsic::Row] -> :wat::core::bool
  (:wat::core::= (:wat::intrinsic::Row/kind r) :wat::runtime::Kind::Intrinsic))

(:wat::core::defn :user::is-macro? [r <- :wat::intrinsic::Row] -> :wat::core::bool
  (:wat::core::= (:wat::intrinsic::Row/kind r) :wat::runtime::Kind::Macro))

(:wat::core::defn :user::empty-syntax? [r <- :wat::intrinsic::Row] -> :wat::core::bool
  (:wat::core::= (:wat::intrinsic::Row/syntax r) ""))

(:wat::core::defn :user::totality-total? [r <- :wat::intrinsic::Row] -> :wat::core::bool
  (:wat::core::= (:wat::intrinsic::Row/totality r) :wat::runtime::Totality::Total))

(:wat::core::defn :user::totality-partial? [r <- :wat::intrinsic::Row] -> :wat::core::bool
  (:wat::core::= (:wat::intrinsic::Row/totality r) :wat::runtime::Totality::Partial))

(:wat::core::defn :user::totality-preserving? [r <- :wat::intrinsic::Row] -> :wat::core::bool
  (:wat::core::= (:wat::intrinsic::Row/totality r) :wat::runtime::Totality::Preserving))

(:wat::core::defn :user::totality-unreviewed? [r <- :wat::intrinsic::Row] -> :wat::core::bool
  (:wat::core::= (:wat::intrinsic::Row/totality r) :wat::runtime::Totality::Unreviewed))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [rows (:wat::intrinsic::rows)
                     total (:wat::core::count rows)
                     n-special-form (:wat::core::count (:wat::core::filterv :user::is-special-form? rows))
                     n-intrinsic (:wat::core::count (:wat::core::filterv :user::is-intrinsic? rows))
                     n-macro (:wat::core::count (:wat::core::filterv :user::is-macro? rows))
                     n-empty-syntax (:wat::core::count (:wat::core::filterv :user::empty-syntax? rows))
                     n-total (:wat::core::count (:wat::core::filterv :user::totality-total? rows))
                     n-partial (:wat::core::count (:wat::core::filterv :user::totality-partial? rows))
                     n-preserving (:wat::core::count (:wat::core::filterv :user::totality-preserving? rows))
                     n-unreviewed (:wat::core::count (:wat::core::filterv :user::totality-unreviewed? rows))]
    (:wat::core::do
      (:wat::kernel::println "=== census 1: total rows ===")
      (:wat::kernel::println (:wat::string::concat "total rows: " (:wat::edn::write total)))

      (:wat::kernel::println "=== census 2: rows by kind ===")
      (:wat::kernel::println (:wat::string::concat "  SpecialForm: " (:wat::edn::write n-special-form)))
      (:wat::kernel::println (:wat::string::concat "  Intrinsic:   " (:wat::edn::write n-intrinsic)))
      (:wat::kernel::println (:wat::string::concat "  Macro:       " (:wat::edn::write n-macro) "  (expected 0 — no registered row is Kind::Macro)"))
      (:wat::kernel::println (:wat::string::concat "  SpecialForm + Intrinsic = " (:wat::edn::write (:wat::core::+ n-special-form n-intrinsic))
                                                     "  (must equal total " (:wat::edn::write total) ")"))

      (:wat::kernel::println "=== census 3: rows with an empty :syntax ===")
      (:wat::kernel::println (:wat::string::concat "empty-syntax rows: " (:wat::edn::write n-empty-syntax)))

      (:wat::kernel::println "=== census 4: rows by :totality (THE WORK LIST is :Partial) ===")
      (:wat::kernel::println (:wat::string::concat "  Total:       " (:wat::edn::write n-total)))
      (:wat::kernel::println (:wat::string::concat "  Partial:     " (:wat::edn::write n-partial) "  <- runtime-meta.wat:241's WORK LIST"))
      (:wat::kernel::println (:wat::string::concat "  Preserving:  " (:wat::edn::write n-preserving)))
      (:wat::kernel::println (:wat::string::concat "  Unreviewed:  " (:wat::edn::write n-unreviewed)))
      (:wat::kernel::println (:wat::string::concat "  sum = " (:wat::edn::write (:wat::core::+ n-total (:wat::core::+ n-partial (:wat::core::+ n-preserving n-unreviewed))))
                                                     "  (must equal total " (:wat::edn::write total) ")")))))
