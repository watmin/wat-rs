;; `edn::write` of a holon it cannot tag USED TO PANIC (`src/edn_shim.rs`, "cannot encode HolonAST
;; to the wire"). It is a data-dependent failure — the value comes from the user's program — so it
;; must be a located diagnostic, not a process abort.
;;
;; The two doors differ only in how the SAME data was lifted, which is a separate DEFERRED defect
;; (~/work/NOTE-holon-classifier-contract-is-unenforced-and-the-holon-tag-breaks-it.md). This
;; fixture does not care which door is right — only that the encoder REPORTS instead of dying.
(:wat::core::defn :user::good [] -> :wat::core::String
  (:wat::edn::write (:wat::holon::to-holon (:wat::core::Vector :wat::core::i64 1 2 3))))

(:wat::core::defn :user::compute [] -> :wat::core::String
  (:wat::edn::write #holon [1 2 3]))
