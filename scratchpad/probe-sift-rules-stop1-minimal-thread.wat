;; isolates: does WRAPPING a defsurface+defservice pair inside an outer defmacro's `do` (vs
;; hand-writing them at top level) break accessor (`/field`)/enum-variant-ctor (`::Variant`)
;; resolution for records/enums declared in the SAME :messages vector — independent of rete.
;; No rules, no rete — just echo req's field back via the surface's own generated Req/Resp types.

(:wat::core::defmacro :probe::echo-defsvc
  [def-form <- :wat::WatAST]
  -> :wat::WatAST
  `(:wat::core::do
     (:wat::core::defsurface :probe::Echo :nature :wat::kernel::Peer'
       :messages
       [~def-form
        (:wat::core::defrecord :probe::Echo::Req [c <- :wat::core::i64])
        (:wat::core::defenum :probe::Echo::Resp :wat::enum::Pure
          :Val [n <- :wat::core::i64])]
       :features
       [(echo [self <- :probe::Echo req <- :probe::Echo::Req] -> :probe::Echo::Resp)])
     (:wat::service::defservice :probe::echosvc'
       :satisfies :probe::Echo
       :durable []
       :impls
       [(echo [s req]
          (:wat::core::let
            [c (:probe::Echo::Req/c req)]
            (:wat::service::Outcome::Reply s (:probe::Echo::Resp::Val c))))])))

(:probe::echo-defsvc (:wat::core::defrecord :probe::Marker [x <- :wat::core::i64]))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [h    (:probe::echosvc'/start :locus (:wat::spawn::thread) :record (:probe::echosvc'::Record))
     addr (:probe::echosvc'::Handle/addr h)
     cli  (:wat::kernel::connect' addr)
     r    (:probe::Echo/echo cli (:probe::Echo::Req :c 42))]
    (:wat::kernel::println (:wat::core::str "r=" r))))
