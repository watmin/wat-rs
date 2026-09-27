;; Excursus 003 step 4 (D4) — G3, privilege, not prefix.
;;
;; The driver loads THIS fixture as the entry program under the label "wat/foo.wat" —
;; a path string that STARTS WITH "wat/", same as every real stdlib file, but was never
;; baked into the binary. Privilege must come from provenance (this file was genuinely
;; read as the entry program), never from the shared prefix.
(:wat::core::defn :user::grow [n <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::+ n 9223372036854775807))
