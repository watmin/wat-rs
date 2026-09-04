;; probe-instrument-reports.wat — Stone D rows 4 and 5.
;;
;; Row 4: a bounded wait forced to expire names what it last saw.
;; Row 5: a dead peer is distinguishable — q-depth is (-1,-1), not (1,1).

(:wat::config::set-redef! true)
(:wat::load-file! "../topic/sns-fanout.wat")

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [msh (:wat::query::mem-store/start :locus (:wat::spawn::thread)
           :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     qh  (:queue::queue/start :locus (:wat::spawn::thread)
           :record (:queue::queue::Record :cap 8 :store-addr (:wat::query::mem-store::Handle/addr msh)))
     q   (:demo::dial-queue (:queue::queue::Handle/addr qh))
     expired (:demo::poll-until-unacked q 3)
     live (:demo::q-depth q)
     _ (:queue::queue/stop qh)
     dead (:demo::q-depth q)
     unread (:demo::poll-until-unacked q 3)]
    (:wat::core::let
      [_ (:wat::kernel::println
           (:wat::core::format "expired={e};live={v}/{u}"
             :e expired :v (:wat::core::first live) :u (:wat::core::second live)))
       _ (:wat::kernel::println
           (:wat::core::format "dead={v}/{u};unread={r}"
             :v (:wat::core::first dead) :u (:wat::core::second dead) :r unread))]
      nil)))
