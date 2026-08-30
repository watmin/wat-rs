;; Gate 2 — Surface ships. A user peer surface is registered in the parent; when a
;; process bracket forks, freeze ships the surface's RETAINED source form verbatim
;; (Arc 170 fix) instead of reconstructing it via `type_def_to_ast` (which emits the
;; obsolete `(:defsurface :Name [members])` grammar and cannot recover `:messages` →
;; the child crashes at "expected `:nature :<kw>`"). EXPECT: "[2 4 6]", no crash.

(:wat::core::defsurface :probe::Foo :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :probe::Foo::FRequest  [x <- :wat::core::i64])
   (:wat::core::defenum :probe::Foo::FResponse :wat::enum::Pure :Ok [y <- :wat::core::i64] :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
                                                                                      :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(f [self <- :probe::Foo  req <- :probe::Foo::FRequest] -> :probe::Foo::FResponse :max-request-bytes 524288)])

(:wat::core::defn :probe::double [n <- :wat::core::i64] -> :wat::core::i64
  (:wat::i64::* n 2))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [nums (:wat::core::Vector :- [:wat::core::i64] 1 2 3)
     pr   (:wat::bracket::map (:wat::spawn::process) nums :probe::double)
     _    (:wat::kernel::println (:wat::edn::write pr))
     expected (:wat::core::Vector :- [:wat::core::i64] 2 4 6)
     _    (:wat::core::if (:wat::core::= pr expected)
             nil
            (:wat::kernel::assertion-failed! "surface-ships result mismatch" :wat::core::None :wat::core::None))]
    (:wat::kernel::println "surface-ships-ok")))
