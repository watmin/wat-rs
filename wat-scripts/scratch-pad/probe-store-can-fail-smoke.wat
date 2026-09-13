;; Isolate: proxy-only put, no queue.
(:wat::config::set-redef! true)
(:wat::load-file! "../query/faulting-store.wat")

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [sh (:wat::query::mem-store/start :locus (:wat::spawn::thread)
          :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     ph (:query::faulting-store/start :locus (:wat::spawn::thread)
          :record (:query::faulting-store::Record
                    :real-addr (:wat::query::mem-store::Handle/addr sh)
                    :drop-reply-bp 0 :die-bp 0 :seed 1 :drops-fired 0 :dies-fired 0))
     p (:wat::core::match (:wat::kernel::connect (:query::faulting-store::Handle/addr ph))
         ((:wat::kernel::ConnectOutcome::Connected c) c)
         (_ (:wat::kernel::assertion-failed! "smoke: proxy dial failed" :wat::core::None :wat::core::None)))
     es (:wat::query::Store/ensure-schema p
           (:wat::query::Store::EnsureSchemaRequest
             :table (:wat::query::TableSchema :pk "pk" :sk "sk")
             :indexes (:wat::core::Vector :- [:wat::query::IndexSchema]
                        (:wat::query::IndexSchema
                          :name "by-visible-at" :pk "pk" :sk "sk" :ipk "ipk" :isk "isk"))))]
    (:wat::core::match es
      ((:wat::kernel::RecvOutcome::Message r)
        (:wat::kernel::println (:wat::core::format "ensure={r}" :r r)))
      ((:wat::kernel::RecvOutcome::Lost c)
        (:wat::kernel::println
          (:wat::core::format "ensure-Lost={m}" :m (:wat::kernel::LociDiedError/message c))))
      (:wat::kernel::RecvOutcome::Closed (:wat::kernel::println "ensure-Closed"))
      (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::println "ensure-TimedOut"))
      (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::println "ensure-Stopped"))
      ((:wat::kernel::RecvOutcome::Malformed _) (:wat::kernel::println "ensure-Malformed")))))
