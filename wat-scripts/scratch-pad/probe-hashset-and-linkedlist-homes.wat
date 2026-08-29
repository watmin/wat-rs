;; probe-hashset-and-linkedlist-homes.wat — arc 255 Stone E-iii acceptance row 1.
;;
;; Asserts a result for each of the 9 moved verbs under their NEW spellings:
;;   :wat::hashset::{length, empty?, contains?, conj}          (4)
;;   :wat::linkedlist::{conj, contains?, empty?, get, length}  (5)
;;
;; Usage: ./target/release/wat wat-scripts/scratch-pad/probe-hashset-and-linkedlist-homes.wat

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    ;; ── HashSet ──────────────────────────────────────────────────────────
    (:wat::test::assert-eq (:wat::hashset::length (:wat::core::HashSet :- [:wat::core::i64])) 0)
    (:wat::test::assert-eq (:wat::hashset::length (:wat::core::HashSet :- [:wat::core::i64] 1 2 3)) 3)
    (:wat::test::assert-eq (:wat::hashset::empty? (:wat::core::HashSet :- [:wat::core::i64])) true)
    (:wat::test::assert-eq (:wat::hashset::empty? (:wat::core::HashSet :- [:wat::core::i64] 1)) false)
    (:wat::test::assert-eq (:wat::hashset::contains? (:wat::core::HashSet :- [:wat::core::i64] 1 2 3) 2) true)
    (:wat::test::assert-eq (:wat::hashset::contains? (:wat::core::HashSet :- [:wat::core::i64] 1 2 3) 9) false)
    (:wat::test::assert-eq (:wat::hashset::length (:wat::hashset::conj (:wat::core::HashSet :- [:wat::core::i64]) 1)) 1)
    (:wat::test::assert-eq (:wat::hashset::contains? (:wat::hashset::conj (:wat::core::HashSet :- [:wat::core::i64]) 7) 7) true)

    ;; ── List (LinkedList) ────────────────────────────────────────────────
    (:wat::test::assert-eq (:wat::linkedlist::length (:wat::core::List)) 0)
    (:wat::test::assert-eq (:wat::linkedlist::length (:wat::core::List 1 2 3)) 3)
    (:wat::test::assert-eq (:wat::linkedlist::empty? (:wat::core::List)) true)
    (:wat::test::assert-eq (:wat::linkedlist::empty? (:wat::core::List 1)) false)
    (:wat::test::assert-eq (:wat::linkedlist::contains? (:wat::core::List 1 2 3) 2) true)
    (:wat::test::assert-eq (:wat::linkedlist::contains? (:wat::core::List 1 2 3) 9) false)
    (:wat::test::assert-eq (:wat::linkedlist::get (:wat::core::List 10 20 30) 0) (:wat::core::Some 10))
    (:wat::test::assert-eq (:wat::linkedlist::get (:wat::core::List 10 20 30) 9) :wat::core::None)
    (:wat::test::assert-eq (:wat::linkedlist::length (:wat::linkedlist::conj (:wat::core::List) 1)) 1)
    (:wat::test::assert-eq (:wat::linkedlist::get (:wat::linkedlist::conj (:wat::core::List 2 3) 1) 0) (:wat::core::Some 1))

    (:wat::kernel::println "OK: all 9 verbs (4 hashset + 5 linkedlist) run under their new spellings")))
