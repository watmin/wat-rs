;; Scratch probe — arc 255 Stone O-iv-b, acceptance row 0.
;;
;; THE CLAIM UNDER TEST: after migrating the 32 collection verbs (map · hashmap · vec ·
;; linkedlist · hashset) to ALGEBRA, every one of them reaches through
;; `:wat::core::apply`, not just direct calls. Before the strike, the 8 `:wat::map::` rows
;; report the O-iv-a "registered-but-unreachable" diagnostic (no value door yet) while the
;; other 24 already work through apply (Stone N hand-written twins). After the strike, all
;; 32 succeed through apply.
;;
;; Run against the pre-migration tree and the post-migration tree; paste both transcripts.

(:wat::core::defn :probe::outcome [r <- (:wat::core::Result :- [wat.type/Value :wat::core::EvalError])]
  -> wat.type/String
  (:wat::core::match r
    [:wat::core::Result.Ok {:value v}  (:wat::string::concat "ok:" (:wat::edn::write v))]
    [:wat::core::Result.Err {:error e} (:wat::string::concat "err:kind=" (:wat::core::EvalError/kind e) " msg=" (:wat::core::EvalError/message e))]))

(:wat::core::defn :probe::row [name <- wat.type/String thru <- wat.type/AST] -> wat.type/nil
  (:wat::kernel::println
    (:wat::string::concat name "  APPLY=" (:probe::outcome (:wat::eval-ast! thru)))))

(:wat::core::defn :user::main [] -> wat.type/nil
  (:wat::core::let
    [;; ── map (8) ──────────────────────────────────────────────────────────
     _01 (:probe::row ":wat::map::length      "
           (:wat::core::quote (:wat::core::apply :wat::core::length
             (wat.type/Vector :- [(wat.type/PersistentMap :- [wat.type/String wat.type/i64])]
               (:wat::core::assoc (wat.type/PersistentMap :- [wat.type/String wat.type/i64]) "a" 1)))))
     _02 (:probe::row ":wat::map::empty?      "
           (:wat::core::quote (:wat::core::apply :wat::core::empty?
             (wat.type/Vector :- [(wat.type/PersistentMap :- [wat.type/String wat.type/i64])]
               (wat.type/PersistentMap :- [wat.type/String wat.type/i64])))))
     _03 (:probe::row ":wat::map::contains-key?"
           (:wat::core::quote (:wat::core::apply :wat::core::contains?
             (:wat::core::assoc (wat.type/PersistentMap :- [wat.type/String wat.type/i64]) "a" 1)
             (wat.type/Vector :- [wat.type/String] "a"))))
     _04 (:probe::row ":wat::map::get         "
           (:wat::core::quote (:wat::core::apply :wat::core::get
             (:wat::core::assoc (wat.type/PersistentMap :- [wat.type/String wat.type/i64]) "a" 1)
             (wat.type/Vector :- [wat.type/String] "a"))))
     _05 (:probe::row ":wat::map::assoc       "
           (:wat::core::quote (:wat::core::apply :wat::core::assoc
             (wat.type/PersistentMap :- [wat.type/String wat.type/i64]) "a"
             (wat.type/Vector :- [wat.type/i64] 1))))
     _06 (:probe::row ":wat::map::dissoc      "
           (:wat::core::quote (:wat::core::apply :wat::core::dissoc
             (:wat::core::assoc (wat.type/PersistentMap :- [wat.type/String wat.type/i64]) "a" 1)
             (wat.type/Vector :- [wat.type/String] "a"))))
     _07 (:probe::row ":wat::map::keys        "
           (:wat::core::quote (:wat::core::apply :wat::core::keys
             (wat.type/Vector :- [(wat.type/PersistentMap :- [wat.type/String wat.type/i64])]
               (:wat::core::assoc (wat.type/PersistentMap :- [wat.type/String wat.type/i64]) "a" 1)))))
     _08 (:probe::row ":wat::map::values      "
           (:wat::core::quote (:wat::core::apply :wat::core::values
             (wat.type/Vector :- [(wat.type/PersistentMap :- [wat.type/String wat.type/i64])]
               (:wat::core::assoc (wat.type/PersistentMap :- [wat.type/String wat.type/i64]) "a" 1)))))

     ;; ── hashmap (8) ──────────────────────────────────────────────────────
     _09 (:probe::row ":wat::hashmap::length      "
           (:wat::core::quote (:wat::core::apply :wat::core::length
             (wat.type/Vector :- [(wat.type/HashMap :- [wat.type/String wat.type/i64])]
               (:wat::core::assoc (wat.type/HashMap :- [wat.type/String wat.type/i64]) "a" 1)))))
     _10 (:probe::row ":wat::hashmap::empty?      "
           (:wat::core::quote (:wat::core::apply :wat::core::empty?
             (wat.type/Vector :- [(wat.type/HashMap :- [wat.type/String wat.type/i64])]
               (wat.type/HashMap :- [wat.type/String wat.type/i64])))))
     _11 (:probe::row ":wat::hashmap::contains-key?"
           (:wat::core::quote (:wat::core::apply :wat::core::contains?
             (:wat::core::assoc (wat.type/HashMap :- [wat.type/String wat.type/i64]) "a" 1)
             (wat.type/Vector :- [wat.type/String] "a"))))
     _12 (:probe::row ":wat::hashmap::get         "
           (:wat::core::quote (:wat::core::apply :wat::core::get
             (:wat::core::assoc (wat.type/HashMap :- [wat.type/String wat.type/i64]) "a" 1)
             (wat.type/Vector :- [wat.type/String] "a"))))
     _13 (:probe::row ":wat::hashmap::assoc       "
           (:wat::core::quote (:wat::core::apply :wat::core::assoc
             (wat.type/HashMap :- [wat.type/String wat.type/i64]) "a"
             (wat.type/Vector :- [wat.type/i64] 1))))
     _14 (:probe::row ":wat::hashmap::dissoc      "
           (:wat::core::quote (:wat::core::apply :wat::core::dissoc
             (:wat::core::assoc (wat.type/HashMap :- [wat.type/String wat.type/i64]) "a" 1)
             (wat.type/Vector :- [wat.type/String] "a"))))
     _15 (:probe::row ":wat::hashmap::keys        "
           (:wat::core::quote (:wat::core::apply :wat::core::keys
             (wat.type/Vector :- [(wat.type/HashMap :- [wat.type/String wat.type/i64])]
               (:wat::core::assoc (wat.type/HashMap :- [wat.type/String wat.type/i64]) "a" 1)))))
     _16 (:probe::row ":wat::hashmap::values      "
           (:wat::core::quote (:wat::core::apply :wat::core::values
             (wat.type/Vector :- [(wat.type/HashMap :- [wat.type/String wat.type/i64])]
               (:wat::core::assoc (wat.type/HashMap :- [wat.type/String wat.type/i64]) "a" 1)))))

     ;; ── vec (7) ──────────────────────────────────────────────────────────
     _17 (:probe::row ":wat::vec::length  "
           (:wat::core::quote (:wat::core::apply :wat::core::length
             (wat.type/Vector :- [(wat.type/Vector :- [wat.type/i64])]
               (wat.type/Vector :- [wat.type/i64] 1 2 3)))))
     _18 (:probe::row ":wat::vec::empty?  "
           (:wat::core::quote (:wat::core::apply :wat::core::empty?
             (wat.type/Vector :- [(wat.type/Vector :- [wat.type/i64])]
               (wat.type/Vector :- [wat.type/i64])))))
     _19 (:probe::row ":wat::vec::contains?"
           (:wat::core::quote (:wat::core::apply :wat::core::contains?
             (wat.type/Vector :- [wat.type/i64] 1 2 3)
             (wat.type/Vector :- [wat.type/i64] 2))))
     _20 (:probe::row ":wat::vec::get      "
           (:wat::core::quote (:wat::core::apply :wat::core::get
             (wat.type/Vector :- [wat.type/i64] 1 2 3)
             (wat.type/Vector :- [wat.type/i64] 0))))
     _21 (:probe::row ":wat::vec::conj     "
           (:wat::core::quote (:wat::core::apply :wat::core::conj
             (wat.type/Vector :- [wat.type/i64])
             (wat.type/Vector :- [wat.type/i64] 1))))
     _22 (:probe::row ":wat::vec::concat   "
           (:wat::core::quote (:wat::core::apply :wat::core::concat
             (wat.type/Vector :- [wat.type/i64] 1)
             (wat.type/Vector :- [(wat.type/Vector :- [wat.type/i64])]
               (wat.type/Vector :- [wat.type/i64] 2)))))
     _23 (:probe::row ":wat::vec::extend   "
           (:wat::core::quote (:wat::core::apply :wat::core::into
             (wat.type/Vector :- [wat.type/i64] 1)
             (wat.type/Vector :- [(wat.type/Vector :- [wat.type/i64])]
               (wat.type/Vector :- [wat.type/i64] 2 3)))))

     ;; ── linkedlist (5) ───────────────────────────────────────────────────
     _24 (:probe::row ":wat::linkedlist::length  "
           (:wat::core::quote (:wat::core::apply :wat::core::length
             (wat.type/Vector :- [(wat.type/List :- [wat.type/i64])]
               (wat.type/List :- [wat.type/i64] 1 2 3)))))
     _25 (:probe::row ":wat::linkedlist::empty?  "
           (:wat::core::quote (:wat::core::apply :wat::core::empty?
             (wat.type/Vector :- [(wat.type/List :- [wat.type/i64])]
               (wat.type/List :- [wat.type/i64])))))
     _26 (:probe::row ":wat::linkedlist::contains?"
           (:wat::core::quote (:wat::core::apply :wat::core::contains?
             (wat.type/List :- [wat.type/i64] 1 2 3)
             (wat.type/Vector :- [wat.type/i64] 2))))
     _27 (:probe::row ":wat::linkedlist::get      "
           (:wat::core::quote (:wat::core::apply :wat::core::get
             (wat.type/List :- [wat.type/i64] 1 2 3)
             (wat.type/Vector :- [wat.type/i64] 0))))
     _28 (:probe::row ":wat::linkedlist::conj     "
           (:wat::core::quote (:wat::core::apply :wat::core::conj
             (wat.type/List :- [wat.type/i64] 1 2 3)
             (wat.type/Vector :- [wat.type/i64] 0))))

     ;; ── hashset (4) ──────────────────────────────────────────────────────
     _29 (:probe::row ":wat::hashset::length  "
           (:wat::core::quote (:wat::core::apply :wat::core::length
             (wat.type/Vector :- [(wat.type/HashSet :- [wat.type/i64])]
               (wat.type/HashSet :- [wat.type/i64] 1 2 3)))))
     _30 (:probe::row ":wat::hashset::empty?  "
           (:wat::core::quote (:wat::core::apply :wat::core::empty?
             (wat.type/Vector :- [(wat.type/HashSet :- [wat.type/i64])]
               (wat.type/HashSet :- [wat.type/i64])))))
     _31 (:probe::row ":wat::hashset::contains?"
           (:wat::core::quote (:wat::core::apply :wat::core::contains?
             (wat.type/HashSet :- [wat.type/i64] 1 2 3)
             (wat.type/Vector :- [wat.type/i64] 2))))
     ;; ⚠ ORDER-INDEPENDENT ON PURPOSE (fixed 2026-08-28, Stone Q). This row used to print the
     ;; conj'd SET itself, and `HashSet`'s iteration order varies run to run — so the probe was
     ;; NON-DETERMINISTIC while being used as a byte-identical diff instrument. It would have
     ;; handed a future stone a false RED, or taught someone to ignore a real one. Asking
     ;; `contains?` proves the conj reached the verb through apply AND renders the same every run.
     ;; `[[feedback_a_probe_that_recalibrates_under_load_measures_nothing]]`
     _32 (:probe::row ":wat::hashset::conj     "
           (:wat::core::quote (:wat::core::contains?
             (:wat::core::apply :wat::core::conj
               (wat.type/HashSet :- [wat.type/i64] 1 2 3)
               (wat.type/Vector :- [wat.type/i64] 9))
             9)))]
    nil))
