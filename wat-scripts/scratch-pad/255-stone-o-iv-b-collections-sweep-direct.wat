;; Scratch probe — arc 255 Stone O-iv-b, acceptance row 1.
;;
;; THE CLAIM UNDER TEST: migrating the 32 `:wat::{map,hashmap,vec,linkedlist,hashset}::*`
;; verbs from BINDING (hand-written AST shell, 24 of them also a hand-written value twin)
;; to ALGEBRA (one declaration, macro-generated doors) changes NOTHING about the direct
;; call's observable behaviour — value AND error text, byte-identical before and after.
;;
;; Each verb gets a success-path direct call. Each of the five files also gets at least one
;; type-mismatch error path (wrong-typed argument, forced through `:wat::eval-ast!` +
;; `:wat::core::quote` so the static checker cannot refuse it first) and one arity-mismatch
;; error path, matching the shape of the O-iii sibling probe.
;;
;; Run against the pre-migration tree and the post-migration tree; diff the two transcripts.

(:wat::core::defn :probe::outcome [r <- (:wat::core::Result :wat::core::Value :wat::core::EvalError)]
  -> :wat::core::String
  (:wat::core::match r
    ((:wat::core::Ok v)  (:wat::string::concat "ok:" (:wat::edn::write v)))
    ((:wat::core::Err e) (:wat::string::concat "err:kind=" (:wat::core::EvalError/kind e) " msg=" (:wat::core::EvalError/message e)))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [;; ── map (8) success paths ───────────────────────────────────────────
     _m0 (:wat::core::PersistentMap)
     _m1 (:wat::map::assoc _m0 "a" 1)
     _01 (:wat::kernel::println (:wat::string::concat "map::length        " (:wat::edn::write (:wat::map::length _m1))))
     _02 (:wat::kernel::println (:wat::string::concat "map::empty? true   " (:wat::edn::write (:wat::map::empty? _m0))))
     _03 (:wat::kernel::println (:wat::string::concat "map::empty? false  " (:wat::edn::write (:wat::map::empty? _m1))))
     _04 (:wat::kernel::println (:wat::string::concat "map::contains-key? true  " (:wat::edn::write (:wat::map::contains-key? _m1 "a"))))
     _05 (:wat::kernel::println (:wat::string::concat "map::contains-key? false " (:wat::edn::write (:wat::map::contains-key? _m1 "z"))))
     _06 (:wat::kernel::println (:wat::string::concat "map::get hit        " (:wat::edn::write (:wat::map::get _m1 "a"))))
     _07 (:wat::kernel::println (:wat::string::concat "map::get miss       " (:wat::edn::write (:wat::map::get _m1 "z"))))
     _08 (:wat::kernel::println (:wat::string::concat "map::assoc          " (:wat::edn::write (:wat::map::assoc _m0 "b" 2))))
     _09 (:wat::kernel::println (:wat::string::concat "map::dissoc         " (:wat::edn::write (:wat::map::dissoc _m1 "a"))))
     _10 (:wat::kernel::println (:wat::string::concat "map::keys           " (:wat::edn::write (:wat::vec::length (:wat::map::keys _m1)))))
     _11 (:wat::kernel::println (:wat::string::concat "map::values         " (:wat::edn::write (:wat::vec::length (:wat::map::values (:wat::core::PersistentMap))))))
     ;; error paths — one type-mismatch, one arity-mismatch (via eval-ast! to bypass the checker).
     _12 (:wat::kernel::println (:wat::string::concat "map::length type-mismatch: " (:probe::outcome (:wat::eval-ast! (:wat::core::quote (:wat::map::length 5))))))
     _13 (:wat::kernel::println (:wat::string::concat "map::length arity-mismatch: " (:probe::outcome (:wat::eval-ast! (:wat::core::quote (:wat::map::length _m1 _m1))))))

     ;; ── hashmap (8) success paths ────────────────────────────────────────
     _h0 (:wat::core::HashMap :wat::core::String :wat::core::i64)
     _h1 (:wat::hashmap::assoc _h0 "a" 1)
     _14 (:wat::kernel::println (:wat::string::concat "hashmap::length        " (:wat::edn::write (:wat::hashmap::length _h1))))
     _15 (:wat::kernel::println (:wat::string::concat "hashmap::empty? true   " (:wat::edn::write (:wat::hashmap::empty? _h0))))
     _16 (:wat::kernel::println (:wat::string::concat "hashmap::empty? false  " (:wat::edn::write (:wat::hashmap::empty? _h1))))
     _17 (:wat::kernel::println (:wat::string::concat "hashmap::contains-key? true  " (:wat::edn::write (:wat::hashmap::contains-key? _h1 "a"))))
     _18 (:wat::kernel::println (:wat::string::concat "hashmap::contains-key? false " (:wat::edn::write (:wat::hashmap::contains-key? _h1 "z"))))
     _19 (:wat::kernel::println (:wat::string::concat "hashmap::get hit        " (:wat::edn::write (:wat::hashmap::get _h1 "a"))))
     _20 (:wat::kernel::println (:wat::string::concat "hashmap::get miss       " (:wat::edn::write (:wat::hashmap::get _h1 "z"))))
     _21 (:wat::kernel::println (:wat::string::concat "hashmap::assoc          " (:wat::edn::write (:wat::hashmap::assoc _h0 "b" 2))))
     _22 (:wat::kernel::println (:wat::string::concat "hashmap::dissoc         " (:wat::edn::write (:wat::hashmap::dissoc _h1 "a"))))
     _23 (:wat::kernel::println (:wat::string::concat "hashmap::keys           " (:wat::edn::write (:wat::hashmap::keys _h1))))
     _24 (:wat::kernel::println (:wat::string::concat "hashmap::values         " (:wat::edn::write (:wat::hashmap::values _h1))))
     _25 (:wat::kernel::println (:wat::string::concat "hashmap::length type-mismatch: " (:probe::outcome (:wat::eval-ast! (:wat::core::quote (:wat::hashmap::length 5))))))
     _26 (:wat::kernel::println (:wat::string::concat "hashmap::length arity-mismatch: " (:probe::outcome (:wat::eval-ast! (:wat::core::quote (:wat::hashmap::length _h1 _h1))))))

     ;; ── vec (7) success paths ────────────────────────────────────────────
     _v0 (:wat::core::Vector :wat::core::i64)
     _v1 (:wat::core::Vector :wat::core::i64 1 2 3)
     _27 (:wat::kernel::println (:wat::string::concat "vec::length      " (:wat::edn::write (:wat::vec::length _v1))))
     _28 (:wat::kernel::println (:wat::string::concat "vec::empty? true " (:wat::edn::write (:wat::vec::empty? _v0))))
     _29 (:wat::kernel::println (:wat::string::concat "vec::empty? false" (:wat::edn::write (:wat::vec::empty? _v1))))
     _30 (:wat::kernel::println (:wat::string::concat "vec::contains? true " (:wat::edn::write (:wat::vec::contains? _v1 2))))
     _31 (:wat::kernel::println (:wat::string::concat "vec::contains? false" (:wat::edn::write (:wat::vec::contains? _v1 9))))
     _32 (:wat::kernel::println (:wat::string::concat "vec::get in-range" (:wat::edn::write (:wat::vec::get _v1 0))))
     _33 (:wat::kernel::println (:wat::string::concat "vec::get oob     " (:wat::edn::write (:wat::vec::get _v1 9))))
     _34 (:wat::kernel::println (:wat::string::concat "vec::conj        " (:wat::edn::write (:wat::vec::conj _v0 1))))
     _35 (:wat::kernel::println (:wat::string::concat "vec::concat      " (:wat::edn::write (:wat::vec::concat (:wat::core::Vector :wat::core::i64 1) (:wat::core::Vector :wat::core::i64 2)))))
     _36 (:wat::kernel::println (:wat::string::concat "vec::extend      " (:wat::edn::write (:wat::vec::extend (:wat::core::Vector :wat::core::i64 1) (:wat::core::Vector :wat::core::i64 2 3)))))
     _37 (:wat::kernel::println (:wat::string::concat "vec::length type-mismatch: " (:probe::outcome (:wat::eval-ast! (:wat::core::quote (:wat::vec::length 5))))))
     _38 (:wat::kernel::println (:wat::string::concat "vec::length arity-mismatch: " (:probe::outcome (:wat::eval-ast! (:wat::core::quote (:wat::vec::length _v1 _v1))))))

     ;; ── linkedlist (5) success paths ─────────────────────────────────────
     _l0 (:wat::core::List)
     _l1 (:wat::core::List 1 2 3)
     _39 (:wat::kernel::println (:wat::string::concat "linkedlist::length      " (:wat::edn::write (:wat::linkedlist::length _l1))))
     _40 (:wat::kernel::println (:wat::string::concat "linkedlist::empty? true " (:wat::edn::write (:wat::linkedlist::empty? _l0))))
     _41 (:wat::kernel::println (:wat::string::concat "linkedlist::empty? false" (:wat::edn::write (:wat::linkedlist::empty? _l1))))
     _42 (:wat::kernel::println (:wat::string::concat "linkedlist::contains? true " (:wat::edn::write (:wat::linkedlist::contains? _l1 2))))
     _43 (:wat::kernel::println (:wat::string::concat "linkedlist::contains? false" (:wat::edn::write (:wat::linkedlist::contains? _l1 9))))
     _44 (:wat::kernel::println (:wat::string::concat "linkedlist::get in-range" (:wat::edn::write (:wat::linkedlist::get _l1 0))))
     _45 (:wat::kernel::println (:wat::string::concat "linkedlist::get oob     " (:wat::edn::write (:wat::linkedlist::get _l1 9))))
     _46 (:wat::kernel::println (:wat::string::concat "linkedlist::conj        " (:wat::edn::write (:wat::linkedlist::conj _l1 0))))
     _47 (:wat::kernel::println (:wat::string::concat "linkedlist::length type-mismatch: " (:probe::outcome (:wat::eval-ast! (:wat::core::quote (:wat::linkedlist::length 5))))))
     _48 (:wat::kernel::println (:wat::string::concat "linkedlist::length arity-mismatch: " (:probe::outcome (:wat::eval-ast! (:wat::core::quote (:wat::linkedlist::length _l1 _l1))))))

     ;; ── hashset (4) success paths ────────────────────────────────────────
     _s0 (:wat::core::HashSet :wat::core::i64)
     _s1 (:wat::core::HashSet :wat::core::i64 1 2 3)
     _49 (:wat::kernel::println (:wat::string::concat "hashset::length      " (:wat::edn::write (:wat::hashset::length _s1))))
     _50 (:wat::kernel::println (:wat::string::concat "hashset::empty? true " (:wat::edn::write (:wat::hashset::empty? _s0))))
     _51 (:wat::kernel::println (:wat::string::concat "hashset::empty? false" (:wat::edn::write (:wat::hashset::empty? _s1))))
     _52 (:wat::kernel::println (:wat::string::concat "hashset::contains? true " (:wat::edn::write (:wat::hashset::contains? _s1 2))))
     _53 (:wat::kernel::println (:wat::string::concat "hashset::contains? false" (:wat::edn::write (:wat::hashset::contains? _s1 9))))
     _54 (:wat::kernel::println (:wat::string::concat "hashset::conj        " (:wat::edn::write (:wat::hashset::length (:wat::hashset::conj _s1 9)))))
     _55 (:wat::kernel::println (:wat::string::concat "hashset::length type-mismatch: " (:probe::outcome (:wat::eval-ast! (:wat::core::quote (:wat::hashset::length 5))))))
     _56 (:wat::kernel::println (:wat::string::concat "hashset::length arity-mismatch: " (:probe::outcome (:wat::eval-ast! (:wat::core::quote (:wat::hashset::length _s1 _s1))))))]
    nil))
