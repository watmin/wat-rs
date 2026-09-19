;; Happy path of the sibling: every item Ok, values match `map`. Thread tier.
(:wat::core::defn :user::work
  [x <- :wat::core::i64]
  -> :wat::core::i64
  (:wat::core::+ x 1))

(:wat::core::defn :user::ok-value
  [o <- (:wat::bracket::ItemOutcome :- [:wat::core::i64])]
  -> :wat::core::i64
  (:wat::core::match o
    ((:wat::bracket::ItemOutcome::Ok v) v)
    ((:wat::bracket::ItemOutcome::Rejected _c) -1)
    ((:wat::bracket::ItemOutcome::Malformed _c) -1)
    ((:wat::bracket::ItemOutcome::Lost _c) -1)
    (:wat::bracket::ItemOutcome::Closed -1)
    ((:wat::bracket::ItemOutcome::GaveUp _w _l) -1)))

(:wat::core::defn :user::compute [] -> :wat::core::String
  (:wat::core::let
    [outcomes (:wat::bracket::map-by-outcome
                (:wat::spawn::thread/runner-count 2)
                (:wat::core::Vector :- [:wat::core::i64] 1 2 3)
                :user::work)
     mapped (:wat::bracket::map
              (:wat::spawn::thread/runner-count 2)
              (:wat::core::Vector :- [:wat::core::i64] 1 2 3)
              :user::work)
     a (:user::ok-value (:wat::core::nth outcomes 0))
     b (:user::ok-value (:wat::core::nth outcomes 1))
     c (:user::ok-value (:wat::core::nth outcomes 2))
     ma (:wat::core::nth mapped 0)
     mb (:wat::core::nth mapped 1)
     mc (:wat::core::nth mapped 2)]
    (:wat::core::format "by={a},{b},{c};map={ma},{mb},{mc};n={n}"
      :a a :b b :c c :ma ma :mb mb :mc mc :n (:wat::core::length outcomes))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:user::compute)))
