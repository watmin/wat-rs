;; Excursus 003 stone I — `:wat::edn::validate` renders its value with the EDN writer and decodes
;; that against the declared type. Does it consume a Stream's (or HandlePool's) body? Run before and
;; after the writer change; the answer says whether the removed bodies had a behavioural consumer here.
(:wat::core::defn :i::none [] -> (:wat::stream::Stream :- [:wat::core::i64])
  (:wat::stream::empty))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [_ (:wat::kernel::println (:wat::edn::validate (:i::none) (:wat::stream::Stream :- [:wat::core::i64])))
     _ (:wat::kernel::println (:wat::edn::validate (:wat::stream::cons 1 (:wat::stream::lazy (:i::none)))
                                                   (:wat::stream::Stream :- [:wat::core::i64])))
     _ (:wat::kernel::println (:wat::edn::validate (:wat::kernel::HandlePool/new "p" (:wat::core::Vector :- [:wat::core::i64] 1))
                                                   (:wat::kernel::HandlePool :- [:wat::core::i64])))]
    nil))
