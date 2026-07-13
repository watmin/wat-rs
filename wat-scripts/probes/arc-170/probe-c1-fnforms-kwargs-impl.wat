;; C1 probe — does fn-forms, given a COMPUTED keyword ":probe::work$impl", ship the
;; ::Kwargs struct dep too? And what does field-names-of/field-types-of on
;; :probe::work::Kwargs return (canonical wat.type/ decomposable forms)?
(:wat::core::defsurface :probe::Echo :nature :wat::kernel::Peer'
  :messages [(:wat::core::defrecord :probe::Echo::EchoRequest  [msg   <- :wat::core::String])
             (:wat::core::defrecord :probe::Echo::EchoResponse [reply <- :wat::core::String])]
  :features [(echo [self <- :probe::Echo  req <- :probe::Echo::EchoRequest] -> :probe::Echo::EchoResponse)])

(:wat::core::defn :probe::work
  [item <- :wat::core::String
   & [echo <- :wat::kernel::Peer'<probe::Echo::Op,probe::Echo::Reply>]]
  -> :wat::core::String
  (:probe::Echo::EchoResponse/reply
    (:probe::Echo/echo echo (:probe::Echo::EchoRequest :msg item))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [impl-kw (:wat::core::keyword/from-string "probe::work$impl")
     forms   (:wat::kernel::fn-forms impl-kw :user::bracket::work-fn)
     n       (:wat::core::length forms)
     _       (:wat::kernel::println n)
     _       (:wat::kernel::println forms)
     names   (:wat::runtime::field-names-of :probe::work::Kwargs)
     types   (:wat::runtime::field-types-of :probe::work::Kwargs)
     _       (:wat::kernel::println names)
     _       (:wat::kernel::println types)]
    (:wat::kernel::println "c1-fnforms-kwargs-impl: ok")))
