;; Excursus 003 stone H — measurement (c): the writer renders a HandlePool (a String body), a
;; wat::holon::Vector (a {:dim} body) and a forced Stream (its head) as something OTHER than a
;; nil-bodied tag, so the strict wire encoder does not refuse them. Do they round-trip? A child ships
;; each inside a generic Box over its process self-peer; the parent says what arrived.
(:wat::core::defrecord :h::Box :- [T] [x <- :T])


(:wat::core::defn :h::vec-recv [] -> :wat::core::String
  (:wat::core::let
    [svc (:wat::test::spawn-peer (:wat::spawn::process)
           (:wat::core::forms
             (:wat::core::defrecord :h::Box :- [T] [x <- :T])
             (:wat::core::defn :h::ship :- [T] [x <- :T] -> :wat::core::nil
               (:wat::core::match
                 (:wat::kernel::send (:wat::program::self-peer (:h::Box :- [:T]) :wat::core::i64) (:h::Box :x x))
                 [:wat::kernel::SendOutcome.Sent {} nil]
                 [:wat::kernel::SendOutcome.Closed {} nil]
                 [:wat::kernel::SendOutcome.Lost {:cause _c} nil]
                 [:wat::kernel::SendOutcome.Stopped {} nil]))
             (:wat::core::defn :user::main [] -> :wat::core::nil
               (:h::ship (:wat::holon::encode (:wat::holon::to-holon "x"))))))]
    (:wat::core::match (:wat::kernel::recv svc)
      [:wat::kernel::RecvOutcome.Message {:msg m} (:h::vec-arrived m)]
      [:wat::kernel::RecvOutcome.Lost {:cause c} (:wat::core::format "Lost: {m}" :m (:wat::kernel::LociDiedError/message c))]
      [:wat::kernel::RecvOutcome.Stopped {} "Stopped"]
      [:wat::kernel::RecvOutcome.Closed {} "Closed"])))

(:wat::core::defn :h::vec-arrived [b <- (:h::Box :- [:wat::holon::Vector])] -> :wat::core::String
  (:wat::core::format "Message: {w} ; equal to the sender's vector: {e}"
    :w (:wat::edn::write b)
    :e (:wat::core::= (:h::Box/x b) (:wat::holon::encode (:wat::holon::to-holon "x")))))

(:wat::core::defn :h::pool-recv [] -> :wat::core::String
  (:wat::core::let
    [svc (:wat::test::spawn-peer (:wat::spawn::process)
           (:wat::core::forms
             (:wat::core::defrecord :h::Box :- [T] [x <- :T])
             (:wat::core::defn :h::ship :- [T] [x <- :T] -> :wat::core::nil
               (:wat::core::match
                 (:wat::kernel::send (:wat::program::self-peer (:h::Box :- [:T]) :wat::core::i64) (:h::Box :x x))
                 [:wat::kernel::SendOutcome.Sent {} nil]
                 [:wat::kernel::SendOutcome.Closed {} nil]
                 [:wat::kernel::SendOutcome.Lost {:cause _c} nil]
                 [:wat::kernel::SendOutcome.Stopped {} nil]))
             (:wat::core::defn :user::main [] -> :wat::core::nil
               (:h::ship (:wat::kernel::HandlePool/new "p" (:wat::core::Vector :- [:wat::core::i64] 1 2))))))]
    (:wat::core::match (:wat::kernel::recv svc)
      [:wat::kernel::RecvOutcome.Message {:msg m} (:h::pool-arrived m)]
      [:wat::kernel::RecvOutcome.Lost {:cause c} (:wat::core::format "Lost: {m}" :m (:wat::kernel::LociDiedError/message c))]
      [:wat::kernel::RecvOutcome.Stopped {} "Stopped"]
      [:wat::kernel::RecvOutcome.Closed {} "Closed"])))

(:wat::core::defn :h::pool-arrived
  [b <- (:h::Box :- [(:wat::kernel::HandlePool :- [:wat::core::i64])])] -> :wat::core::String
  (:wat::core::format "Message: {w}" :w (:wat::edn::write b)))

(:wat::core::defn :h::empty-stream-recv [] -> :wat::core::String
  (:wat::core::let
    [svc (:wat::test::spawn-peer (:wat::spawn::process)
           (:wat::core::forms
             (:wat::core::defrecord :h::Box :- [T] [x <- :T])
             (:wat::core::defn :h::ship :- [T] [x <- :T] -> :wat::core::nil
               (:wat::core::match
                 (:wat::kernel::send (:wat::program::self-peer (:h::Box :- [:T]) :wat::core::i64) (:h::Box :x x))
                 [:wat::kernel::SendOutcome.Sent {} nil]
                 [:wat::kernel::SendOutcome.Closed {} nil]
                 [:wat::kernel::SendOutcome.Lost {:cause _c} nil]
                 [:wat::kernel::SendOutcome.Stopped {} nil]))
             (:wat::core::defn :h::none [] -> (:wat::stream::Stream :- [:wat::core::i64])
               (:wat::stream::empty))
             (:wat::core::defn :user::main [] -> :wat::core::nil
               (:h::ship (:h::none)))))]
    (:wat::core::match (:wat::kernel::recv svc)
      [:wat::kernel::RecvOutcome.Message {:msg m} (:h::empty-stream-arrived m)]
      [:wat::kernel::RecvOutcome.Lost {:cause c} (:wat::core::format "Lost: {m}" :m (:wat::kernel::LociDiedError/message c))]
      [:wat::kernel::RecvOutcome.Stopped {} "Stopped"]
      [:wat::kernel::RecvOutcome.Closed {} "Closed"])))

(:wat::core::defn :h::empty-stream-arrived
  [b <- (:h::Box :- [(:wat::stream::Stream :- [:wat::core::i64])])] -> :wat::core::String
  (:wat::core::format "Message: {w} ; next of it: {n}" :w (:wat::edn::write b)
    :n (:wat::edn::write (:wat::stream::next (:h::Box/x b)))))

(:wat::core::defn :h::stream-recv [] -> :wat::core::String
  (:wat::core::let
    [svc (:wat::test::spawn-peer (:wat::spawn::process)
           (:wat::core::forms
             (:wat::core::defrecord :h::Box :- [T] [x <- :T])
             (:wat::core::defn :h::ship :- [T] [x <- :T] -> :wat::core::nil
               (:wat::core::match
                 (:wat::kernel::send (:wat::program::self-peer (:h::Box :- [:T]) :wat::core::i64) (:h::Box :x x))
                 [:wat::kernel::SendOutcome.Sent {} nil]
                 [:wat::kernel::SendOutcome.Closed {} nil]
                 [:wat::kernel::SendOutcome.Lost {:cause _c} nil]
                 [:wat::kernel::SendOutcome.Stopped {} nil]))
             (:wat::core::defn :user::main [] -> :wat::core::nil
               (:h::ship (:wat::stream::cons 1 (:wat::stream::lazy (:wat::stream::cons 2 (:wat::stream::lazy (:wat::stream::empty)))))))))]
    (:wat::core::match (:wat::kernel::recv svc)
      [:wat::kernel::RecvOutcome.Message {:msg m} (:h::stream-arrived m)]
      [:wat::kernel::RecvOutcome.Lost {:cause c} (:wat::core::format "Lost: {m}" :m (:wat::kernel::LociDiedError/message c))]
      [:wat::kernel::RecvOutcome.Stopped {} "Stopped"]
      [:wat::kernel::RecvOutcome.Closed {} "Closed"])))

(:wat::core::defn :h::stream-arrived
  [b <- (:h::Box :- [(:wat::stream::Stream :- [:wat::core::i64])])] -> :wat::core::String
  (:wat::core::format "Message: {w}" :w (:wat::edn::write b)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [_ (:wat::kernel::println (:wat::core::format "wat::holon::Vector: {s}" :s (:h::vec-recv)))
     _ (:wat::kernel::println (:wat::core::format "HandlePool:         {s}" :s (:h::pool-recv)))
     _ (:wat::kernel::println (:wat::core::format "Stream (forced):    {s}" :s (:h::stream-recv)))
     _ (:wat::kernel::println (:wat::core::format "Stream (empty):     {s}" :s (:h::empty-stream-recv)))]
    nil))
