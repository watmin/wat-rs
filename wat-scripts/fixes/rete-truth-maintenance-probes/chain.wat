(:wat::core::defrecord :c::A [k <- :wat::core::i64])
(:wat::core::defrecord :c::B [k <- :wat::core::i64])
(:wat::core::defrecord :c::C [k <- :wat::core::i64])

;; R1: A → B (single input match)
(:wat::rete::defrule :c::r1
  :when [(:c::A (?k <- :k))]
  :then (:wat::rete::insert (:c::B ?k)))

;; R2: B JOIN A (derived B joined with the ORIGINAL input A, same k) → C
(:wat::rete::defrule :c::r2
  :when [(:c::B (?k <- :k))
         (:c::A (?k <- :k))]
  :then (:wat::rete::insert (:c::C ?k)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [s0 (:wat::rete::compile (:wat::rete::collect-rules :c))
                    s1 (:wat::rete::insert s0 (:c::A 1))
                    s2 (:wat::rete::insert s1 (:c::A 2))
                    fired (:wat::rete::fire-rules' s2)]
    (:wat::core::do
      (:wat::kernel::println (:wat::core::string::concat "A (input, queried) = " (:wat::core::str (:wat::core::length (:wat::rete::query-by-type-string fired "c::A")))))
      (:wat::kernel::println (:wat::core::string::concat "B (derived)        = " (:wat::core::str (:wat::core::length (:wat::rete::query-by-type-string fired "c::B")))))
      (:wat::kernel::println (:wat::core::string::concat "C (B join A)       = " (:wat::core::str (:wat::core::length (:wat::rete::query-by-type-string fired "c::C"))))))))
