;; isolates further: does the SPLICED user def's OWN accessor (not the template-defined Req/Resp)
;; also fail to resolve inside a macro-emitted defsurface, or only the template-literal records?

(:wat::core::defmacro :probe::echo2-defsvc
  [def-form <- :wat::WatAST]
  -> :wat::WatAST
  `(:wat::core::do
     (:wat::core::defsurface :probe::Echo2 :nature :wat::kernel::Peer'
       :messages
       [~def-form
        (:wat::core::defrecord :probe::Echo2::Req [c <- :wat::core::i64])
        (:wat::core::defenum :probe::Echo2::Resp :wat::enum::Pure
          :Val [n <- :wat::core::i64])]
       :features
       [(echo [self <- :probe::Echo2 req <- :probe::Echo2::Req] -> :probe::Echo2::Resp)])
     (:wat::service::defservice :probe::echosvc2'
       :satisfies :probe::Echo2
       :durable []
       :impls
       [(echo [s req]
          (:wat::core::let
            [m (:probe::Marker2 :x 7)
             mx (:probe::Marker2/x m)]
            (:wat::service::Outcome::Reply s (:probe::Echo2::Resp::Val mx))))])))

(:probe::echo2-defsvc (:wat::core::defrecord :probe::Marker2 [x <- :wat::core::i64]))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [h    (:probe::echosvc2'/start :locus (:wat::spawn::thread) :record (:probe::echosvc2'::Record))
     addr (:probe::echosvc2'::Handle/addr h)
     cli  (:wat::kernel::connect' addr)
     r    (:probe::Echo2/echo cli (:probe::Echo2::Req :c 42))]
    (:wat::kernel::println (:wat::core::str "r=" r))))
