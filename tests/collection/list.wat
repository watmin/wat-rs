;; tests/collection/list.wat — co-located fixture for the sibling probe (.rs),
;; slurped via startup_beside(file!()). Named defns for each WAT-backed test.

(:wat::core::defn :list::length-of-3 [] -> :wat::core::Int
  (:wat::linkedlist::length (:wat::core::List 1 2 3)))

(:wat::core::defn :list::length-of-2 [] -> :wat::core::Int
  (:wat::linkedlist::length (:wat::core::List 1 2)))

(:wat::core::defn :list::empty-q-of-empty [] -> :wat::core::bool
  (:wat::linkedlist::empty? (:wat::core::List)))

(:wat::core::defn :list::length-3 [] -> :wat::core::Int
  (:wat::linkedlist::length (:wat::core::List 10 20 30)))

(:wat::core::defn :list::length-0 [] -> :wat::core::Int
  (:wat::linkedlist::length (:wat::core::List)))

(:wat::core::defn :list::empty-q-true [] -> :wat::core::bool
  (:wat::linkedlist::empty? (:wat::core::List)))

(:wat::core::defn :list::empty-q-false [] -> :wat::core::bool
  (:wat::linkedlist::empty? (:wat::core::List 1)))

(:wat::core::defn :list::first-some [] -> :wat::core::bool
  (:wat::core::= (:wat::core::first (:wat::core::List 10 20 30)) 10))

(:wat::core::defn :list::rest-tail-len [] -> :wat::core::Int
  (:wat::linkedlist::length (:wat::core::rest (:wat::core::List 1 2 3))))

(:wat::core::defn :list::conj-prepends [] -> :wat::core::bool
  (:wat::core::= (:wat::core::first (:wat::linkedlist::conj (:wat::core::List 2 3) 1)) 1))

(:wat::core::defn :list::vec-conj-appends [] -> :wat::core::bool
  (:wat::core::= (:wat::core::first (:wat::vec::conj [2 3] 1)) 2))

(:wat::core::defn :list::contains-found [] -> :wat::core::bool
  (:wat::linkedlist::contains? (:wat::core::List 1 2 3) 2))

(:wat::core::defn :list::contains-not-found [] -> :wat::core::bool
  (:wat::linkedlist::contains? (:wat::core::List 1 2 3) 99))

(:wat::core::defn :list::get-found [] -> :wat::core::bool
  (:wat::core::match (:wat::linkedlist::get (:wat::core::List 10 20 30) 1)
    
    ((:wat::core::Some x) (:wat::core::= x 20))
    (:None false)))

(:wat::core::defn :list::get-oob [] -> :wat::core::bool
  (:wat::core::match (:wat::linkedlist::get (:wat::core::List 10 20 30) 99)
    
    ((:wat::core::Some _) false)
    (:None true)))
