;; probe-the-sets-get-a-persistent-variant.wat
;;
;; Drives `:wat::set::{conj, disj, contains?, empty?, length}` and asserts an
;; EDN round-trip. The original set is unchanged after conj/disj.
;;
;; Usage: ./target/release/wat wat-scripts/scratch-pad/probe-the-sets-get-a-persistent-variant.wat

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [empty (:wat::core::PersistentSet :- [:wat::core::i64])
     s1 (:wat::set::conj empty 1)
     s12 (:wat::set::conj s1 2)
     s12d (:wat::set::conj s12 2)
     written (:wat::edn::write s12)
     back (:wat::edn::read written)]
    (:wat::core::do
      (:wat::test::assert-eq (:wat::set::empty? empty) true)
      (:wat::test::assert-eq (:wat::set::length empty) 0)
      (:wat::test::assert-eq (:wat::set::length s1) 1)
      (:wat::test::assert-eq (:wat::set::length s12) 2)
      (:wat::test::assert-eq (:wat::set::length s12d) 2)
      (:wat::test::assert-eq (:wat::set::contains? s12 1) true)
      (:wat::test::assert-eq (:wat::set::contains? s12 2) true)
      (:wat::test::assert-eq (:wat::set::contains? s12 9) false)
      (:wat::test::assert-eq (:wat::set::empty? (:wat::set::disj s1 1)) true)
      (:wat::test::assert-eq (:wat::set::length (:wat::set::disj s12 9)) 2)
      (:wat::test::assert-eq (:wat::set::length s1) 1)
      (:wat::test::assert-eq (:wat::set::length empty) 0)
      (:wat::test::assert-eq (:wat::set::length (:wat::core::PersistentSet :- [:wat::core::i64] 1 1 2)) 2)
      (:wat::test::assert-eq back s12)
      (:wat::kernel::println "OK: :wat::set:: five verbs + EDN round-trip"))))
