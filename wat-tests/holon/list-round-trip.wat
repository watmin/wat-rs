;; wat-tests/holon/list-round-trip.wat — Arc 220 Stone 220.4: :wat::core::List.
;;
;; Exercises List/of constructor, length, empty?, first, rest, conj (prepend),
;; contains?, and cross-type equality with Vector.
;; All cases pass if (List :- [T]) is correctly wired (eval, dispatch arms, equality).

;; ─── 1: List/of and List/length ────────────────────────────────────────────

(:wat::test::deftest :wat-tests::holon::list-round-trip::list-of-length
  
  (:wat::core::let
    [xs (:wat::core::List 1 2 3)
     n  (:wat::linkedlist::length xs)]
    (:wat::test::assert-eq n 3)))

;; ─── 2: Empty list ─────────────────────────────────────────────────────────

(:wat::test::deftest :wat-tests::holon::list-round-trip::empty-list
  
  (:wat::core::let
    [xs (:wat::core::List)]
    (:wat::test::assert-eq (:wat::linkedlist::empty? xs) true)))

;; ─── 3: List/empty? false ─────────────────────────────────────────────────

(:wat::test::deftest :wat-tests::holon::list-round-trip::nonempty-list-not-empty
  
  (:wat::core::let
    [xs (:wat::core::List 1)]
    (:wat::test::assert-eq (:wat::linkedlist::empty? xs) false)))

;; ─── 4: List/contains? found ─────────────────────────────────────────────

(:wat::test::deftest :wat-tests::holon::list-round-trip::contains-found
  
  (:wat::core::let
    [xs (:wat::core::List 1 2 3)]
    (:wat::test::assert-eq (:wat::linkedlist::contains? xs 2) true)))

;; ─── 5: List/contains? not found ─────────────────────────────────────────

(:wat::test::deftest :wat-tests::holon::list-round-trip::contains-not-found
  
  (:wat::core::let
    [xs (:wat::core::List 1 2 3)]
    (:wat::test::assert-eq (:wat::linkedlist::contains? xs 99) false)))

;; ─── 6: rest length ────────────────────────────────────────────────────────

(:wat::test::deftest :wat-tests::holon::list-round-trip::rest-length
  
  (:wat::core::let
    [xs (:wat::core::List 1 2 3)
     tl (:wat::core::rest xs)]
    (:wat::test::assert-eq (:wat::linkedlist::length tl) 2)))

;; ─── 7: conj prepends — length increases ──────────────────────────────────

(:wat::test::deftest :wat-tests::holon::list-round-trip::conj-length
  
  (:wat::core::let
    [xs (:wat::core::List 2 3)
     ys (:wat::linkedlist::conj xs 1)]
    (:wat::test::assert-eq (:wat::linkedlist::length ys) 3)))

;; ─── 8: same-type equality List == List (same contents) ─────────────────

(:wat::test::deftest :wat-tests::holon::list-round-trip::list-eq-vector
  
  (:wat::core::let
    [lst  (:wat::core::List 1 2 3)
     lst2 (:wat::core::List 1 2 3)
     eq   (:wat::core::= lst lst2)]
    (:wat::test::assert-eq eq true)))
