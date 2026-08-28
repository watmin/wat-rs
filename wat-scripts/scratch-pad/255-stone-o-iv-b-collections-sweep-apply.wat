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

(:wat::core::defn :probe::outcome [r <- (:wat::core::Result :wat::core::Value :wat::core::EvalError)]
  -> :wat::core::String
  (:wat::core::match r
    ((:wat::core::Ok v)  (:wat::string::concat "ok:" (:wat::edn::write v)))
    ((:wat::core::Err e) (:wat::string::concat "err:kind=" (:wat::core::EvalError/kind e) " msg=" (:wat::core::EvalError/message e)))))

(:wat::core::defn :probe::row [name <- :wat::core::String thru <- :wat::WatAST] -> :wat::core::nil
  (:wat::kernel::println
    (:wat::string::concat name "  APPLY=" (:probe::outcome (:wat::eval-ast! thru)))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [;; ── map (8) ──────────────────────────────────────────────────────────
     _01 (:probe::row ":wat::map::length      "
           (:wat::core::quote (:wat::core::apply :wat::map::length
             (:wat::core::Vector (:wat::core::PersistentMap :- [:wat::core::String :wat::core::i64])
               (:wat::map::assoc (:wat::core::PersistentMap) "a" 1)))))
     _02 (:probe::row ":wat::map::empty?      "
           (:wat::core::quote (:wat::core::apply :wat::map::empty?
             (:wat::core::Vector (:wat::core::PersistentMap :- [:wat::core::String :wat::core::i64])
               (:wat::core::PersistentMap)))))
     _03 (:probe::row ":wat::map::contains-key?"
           (:wat::core::quote (:wat::core::apply :wat::map::contains-key?
             (:wat::map::assoc (:wat::core::PersistentMap) "a" 1)
             (:wat::core::Vector :wat::core::String "a"))))
     _04 (:probe::row ":wat::map::get         "
           (:wat::core::quote (:wat::core::apply :wat::map::get
             (:wat::map::assoc (:wat::core::PersistentMap) "a" 1)
             (:wat::core::Vector :wat::core::String "a"))))
     _05 (:probe::row ":wat::map::assoc       "
           (:wat::core::quote (:wat::core::apply :wat::map::assoc
             (:wat::core::PersistentMap) "a"
             (:wat::core::Vector :wat::core::i64 1))))
     _06 (:probe::row ":wat::map::dissoc      "
           (:wat::core::quote (:wat::core::apply :wat::map::dissoc
             (:wat::map::assoc (:wat::core::PersistentMap) "a" 1)
             (:wat::core::Vector :wat::core::String "a"))))
     _07 (:probe::row ":wat::map::keys        "
           (:wat::core::quote (:wat::core::apply :wat::map::keys
             (:wat::core::Vector (:wat::core::PersistentMap :- [:wat::core::String :wat::core::i64])
               (:wat::map::assoc (:wat::core::PersistentMap) "a" 1)))))
     _08 (:probe::row ":wat::map::values      "
           (:wat::core::quote (:wat::core::apply :wat::map::values
             (:wat::core::Vector (:wat::core::PersistentMap :- [:wat::core::String :wat::core::i64])
               (:wat::map::assoc (:wat::core::PersistentMap) "a" 1)))))

     ;; ── hashmap (8) ──────────────────────────────────────────────────────
     _09 (:probe::row ":wat::hashmap::length      "
           (:wat::core::quote (:wat::core::apply :wat::hashmap::length
             (:wat::core::Vector (:wat::core::HashMap :- [:wat::core::String :wat::core::i64])
               (:wat::hashmap::assoc (:wat::core::HashMap :wat::core::String :wat::core::i64) "a" 1)))))
     _10 (:probe::row ":wat::hashmap::empty?      "
           (:wat::core::quote (:wat::core::apply :wat::hashmap::empty?
             (:wat::core::Vector (:wat::core::HashMap :- [:wat::core::String :wat::core::i64])
               (:wat::core::HashMap :wat::core::String :wat::core::i64)))))
     _11 (:probe::row ":wat::hashmap::contains-key?"
           (:wat::core::quote (:wat::core::apply :wat::hashmap::contains-key?
             (:wat::hashmap::assoc (:wat::core::HashMap :wat::core::String :wat::core::i64) "a" 1)
             (:wat::core::Vector :wat::core::String "a"))))
     _12 (:probe::row ":wat::hashmap::get         "
           (:wat::core::quote (:wat::core::apply :wat::hashmap::get
             (:wat::hashmap::assoc (:wat::core::HashMap :wat::core::String :wat::core::i64) "a" 1)
             (:wat::core::Vector :wat::core::String "a"))))
     _13 (:probe::row ":wat::hashmap::assoc       "
           (:wat::core::quote (:wat::core::apply :wat::hashmap::assoc
             (:wat::core::HashMap :wat::core::String :wat::core::i64) "a"
             (:wat::core::Vector :wat::core::i64 1))))
     _14 (:probe::row ":wat::hashmap::dissoc      "
           (:wat::core::quote (:wat::core::apply :wat::hashmap::dissoc
             (:wat::hashmap::assoc (:wat::core::HashMap :wat::core::String :wat::core::i64) "a" 1)
             (:wat::core::Vector :wat::core::String "a"))))
     _15 (:probe::row ":wat::hashmap::keys        "
           (:wat::core::quote (:wat::core::apply :wat::hashmap::keys
             (:wat::core::Vector (:wat::core::HashMap :- [:wat::core::String :wat::core::i64])
               (:wat::hashmap::assoc (:wat::core::HashMap :wat::core::String :wat::core::i64) "a" 1)))))
     _16 (:probe::row ":wat::hashmap::values      "
           (:wat::core::quote (:wat::core::apply :wat::hashmap::values
             (:wat::core::Vector (:wat::core::HashMap :- [:wat::core::String :wat::core::i64])
               (:wat::hashmap::assoc (:wat::core::HashMap :wat::core::String :wat::core::i64) "a" 1)))))

     ;; ── vec (7) ──────────────────────────────────────────────────────────
     _17 (:probe::row ":wat::vec::length  "
           (:wat::core::quote (:wat::core::apply :wat::vec::length
             (:wat::core::Vector (:wat::core::Vector :- [:wat::core::i64])
               (:wat::core::Vector :wat::core::i64 1 2 3)))))
     _18 (:probe::row ":wat::vec::empty?  "
           (:wat::core::quote (:wat::core::apply :wat::vec::empty?
             (:wat::core::Vector (:wat::core::Vector :- [:wat::core::i64])
               (:wat::core::Vector :wat::core::i64)))))
     _19 (:probe::row ":wat::vec::contains?"
           (:wat::core::quote (:wat::core::apply :wat::vec::contains?
             (:wat::core::Vector :wat::core::i64 1 2 3)
             (:wat::core::Vector :wat::core::i64 2))))
     _20 (:probe::row ":wat::vec::get      "
           (:wat::core::quote (:wat::core::apply :wat::vec::get
             (:wat::core::Vector :wat::core::i64 1 2 3)
             (:wat::core::Vector :wat::core::i64 0))))
     _21 (:probe::row ":wat::vec::conj     "
           (:wat::core::quote (:wat::core::apply :wat::vec::conj
             (:wat::core::Vector :wat::core::i64)
             (:wat::core::Vector :wat::core::i64 1))))
     _22 (:probe::row ":wat::vec::concat   "
           (:wat::core::quote (:wat::core::apply :wat::vec::concat
             (:wat::core::Vector :wat::core::i64 1)
             (:wat::core::Vector (:wat::core::Vector :- [:wat::core::i64])
               (:wat::core::Vector :wat::core::i64 2)))))
     _23 (:probe::row ":wat::vec::extend   "
           (:wat::core::quote (:wat::core::apply :wat::vec::extend
             (:wat::core::Vector :wat::core::i64 1)
             (:wat::core::Vector (:wat::core::Vector :- [:wat::core::i64])
               (:wat::core::Vector :wat::core::i64 2 3)))))

     ;; ── linkedlist (5) ───────────────────────────────────────────────────
     _24 (:probe::row ":wat::linkedlist::length  "
           (:wat::core::quote (:wat::core::apply :wat::linkedlist::length
             (:wat::core::Vector (:wat::core::List :- [:wat::core::i64])
               (:wat::core::List 1 2 3)))))
     _25 (:probe::row ":wat::linkedlist::empty?  "
           (:wat::core::quote (:wat::core::apply :wat::linkedlist::empty?
             (:wat::core::Vector (:wat::core::List :- [:wat::core::i64])
               (:wat::core::List)))))
     _26 (:probe::row ":wat::linkedlist::contains?"
           (:wat::core::quote (:wat::core::apply :wat::linkedlist::contains?
             (:wat::core::List 1 2 3)
             (:wat::core::Vector :wat::core::i64 2))))
     _27 (:probe::row ":wat::linkedlist::get      "
           (:wat::core::quote (:wat::core::apply :wat::linkedlist::get
             (:wat::core::List 1 2 3)
             (:wat::core::Vector :wat::core::i64 0))))
     _28 (:probe::row ":wat::linkedlist::conj     "
           (:wat::core::quote (:wat::core::apply :wat::linkedlist::conj
             (:wat::core::List 1 2 3)
             (:wat::core::Vector :wat::core::i64 0))))

     ;; ── hashset (4) ──────────────────────────────────────────────────────
     _29 (:probe::row ":wat::hashset::length  "
           (:wat::core::quote (:wat::core::apply :wat::hashset::length
             (:wat::core::Vector (:wat::core::HashSet :- [:wat::core::i64])
               (:wat::core::HashSet :wat::core::i64 1 2 3)))))
     _30 (:probe::row ":wat::hashset::empty?  "
           (:wat::core::quote (:wat::core::apply :wat::hashset::empty?
             (:wat::core::Vector (:wat::core::HashSet :- [:wat::core::i64])
               (:wat::core::HashSet :wat::core::i64)))))
     _31 (:probe::row ":wat::hashset::contains?"
           (:wat::core::quote (:wat::core::apply :wat::hashset::contains?
             (:wat::core::HashSet :wat::core::i64 1 2 3)
             (:wat::core::Vector :wat::core::i64 2))))
     _32 (:probe::row ":wat::hashset::conj     "
           (:wat::core::quote (:wat::core::apply :wat::hashset::conj
             (:wat::core::HashSet :wat::core::i64 1 2 3)
             (:wat::core::Vector :wat::core::i64 9))))]
    nil))
