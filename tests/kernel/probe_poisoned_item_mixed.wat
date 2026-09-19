;; Mixed input: three good items and one oversize poison. map-by-outcome must
;; return the good results, name the poison, and keep the fleet. Process tier
;; (FrameTooLarge is a wire cap). 3 runners so a survivor can finish the rest.
(:wat::core::defn :user::double-n
  [s <- :wat::core::String  n <- :wat::core::i64]
  -> :wat::core::String
  (:wat::core::if (:wat::i64::<= n 0)
    s
    (:user::double-n (:wat::string::concat s s) (:wat::i64::- n 1))))

(:wat::core::defn :user::work
  [x <- :wat::core::i64]
  -> :wat::core::String
  (:wat::core::if (:wat::core::= x -1)
    (:user::double-n "x" 20)
    (:wat::i64::to-string (:wat::core::+ x 1))))

(:wat::core::defn :user::piece
  [i <- :wat::core::i64
   o <- (:wat::bracket::ItemOutcome :- [:wat::core::String])]
  -> :wat::core::String
  (:wat::core::match o
    ((:wat::bracket::ItemOutcome::Ok v)
      (:wat::core::format "ok:{i}={v}" :i i :v v))
    ((:wat::bracket::ItemOutcome::Rejected c)
      (:wat::core::format "rejected:{i}={c}" :i i :c (:wat::kernel::Failure/message c)))
    ((:wat::bracket::ItemOutcome::Malformed c)
      (:wat::core::format "malformed:{i}={c}" :i i :c (:wat::kernel::Failure/message c)))
    ((:wat::bracket::ItemOutcome::Lost c)
      (:wat::core::format "lost:{i}={c}" :i i :c (:wat::kernel::Failure/message c)))
    (:wat::bracket::ItemOutcome::Closed
      (:wat::core::format "closed:{i}" :i i))
    ((:wat::bracket::ItemOutcome::GaveUp w last)
      (:wat::core::format "gaveup:{i}={w}:{last}" :i i :w w :last last))))

(:wat::core::defn :user::summarize
  [outcomes <- (:wat::core::Vector :- [(:wat::bracket::ItemOutcome :- [:wat::core::String])])]
  -> :wat::core::String
  (:wat::core::let
    [n (:wat::core::length outcomes)]
    (:wat::core::foldl
      (:wat::core::fn [acc <- :wat::core::String  i <- :wat::core::i64]
        -> :wat::core::String
        (:wat::core::let
          [p (:user::piece i (:wat::core::nth outcomes i))]
          (:wat::core::if (:wat::core::= acc "")
            p
            (:wat::core::format "{acc};{p}" :acc acc :p p))))
      ""
      (:wat::core::range 0 n))))

(:wat::core::defn :user::compute [] -> :wat::core::String
  (:wat::core::let
    [t0 (:wat::time::epoch-nanos (:wat::time::now))
     outcomes (:wat::bracket::map-by-outcome
                (:wat::spawn::process/runner-count 3)
                (:wat::core::Vector :- [:wat::core::i64] 10 20 -1 40)
                :user::work)
     elapsed (:wat::i64::/ (:wat::i64::- (:wat::time::epoch-nanos (:wat::time::now)) t0) 1000000)
     body (:user::summarize outcomes)]
    (:wat::core::format "{body};elapsed-ms={e};n={n}"
      :body body :e elapsed :n (:wat::core::length outcomes))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:user::compute)))
