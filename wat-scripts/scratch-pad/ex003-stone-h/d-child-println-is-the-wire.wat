;; Excursus 003 stone H — measurement for brief step 5: is a forked child's stdout its peer channel?
;; The child `println`s (never `send`s); the parent `recv`s. If a Message arrives, println ships
;; frames the parent decodes as data. Then the same with a Box<Lru> the lenient writer prints as nil.
(:wat::core::defrecord :h::Box :- [T] [x <- :T])

(:wat::core::defn :h::render [b <- (:h::Box :- [:wat::core::i64])] -> :wat::core::String
  (:wat::core::format "Message: {w}" :w (:wat::edn::write b)))

(:wat::core::defn :h::pure [] -> :wat::core::String
  (:wat::core::let
    [svc (:wat::test::spawn-peer (:wat::spawn::process)
           (:wat::core::forms
             (:wat::core::defrecord :h::Box :- [T] [x <- :T])
             (:wat::core::defn :h::ship :- [T] [x <- :T] -> :wat::core::nil
               (:wat::kernel::println (:h::Box :x x)))
             (:wat::core::defn :user::main [] -> :wat::core::nil
               (:wat::core::let
                 [_p (:wat::program::self-peer (:h::Box :- [:wat::core::i64]) :wat::core::i64)]
                 (:h::ship 42)))))]
    (:wat::core::match (:wat::kernel::recv svc)
      [:wat::kernel::RecvOutcome.Message {:msg m} (:h::render m)]
      [:wat::kernel::RecvOutcome.Lost {:cause c} (:wat::core::format "Lost: {m}" :m (:wat::kernel::LociDiedError/message c))]
      [:wat::kernel::RecvOutcome.Stopped {} "Stopped"]
      [:wat::kernel::RecvOutcome.Closed {} "Closed"])))

(:wat::core::defn :h::handle [] -> :wat::core::String
  (:wat::core::let
    [svc (:wat::test::spawn-peer (:wat::spawn::process)
           (:wat::core::forms
             (:wat::core::defrecord :h::Box :- [T] [x <- :T])
             (:wat::core::defn :h::ship :- [T] [x <- :T] -> :wat::core::nil
               (:wat::kernel::println (:h::Box :x x)))
             (:wat::core::defn :user::main [] -> :wat::core::nil
               (:wat::core::let
                 [_p (:wat::program::self-peer (:h::Box :- [:wat::core::i64]) :wat::core::i64)]
                 (:h::ship (:wat::core::Result/expect
                             (:wat::cache::Lru/new :- [:wat::core::i64 :wat::core::i64] 2) "lru"))))))]
    (:wat::core::match (:wat::kernel::recv svc)
      [:wat::kernel::RecvOutcome.Message {:msg m} (:h::render m)]
      [:wat::kernel::RecvOutcome.Lost {:cause c} (:wat::core::format "Lost: {m}" :m (:wat::kernel::LociDiedError/message c))]
      [:wat::kernel::RecvOutcome.Stopped {} "Stopped"]
      [:wat::kernel::RecvOutcome.Closed {} "Closed"])))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [_ (:wat::kernel::println (:wat::core::format "child println pure Box:   {s}" :s (:h::pure)))
     _ (:wat::kernel::println (:wat::core::format "child println Box<Lru>:    {s}" :s (:h::handle)))]
    nil))
