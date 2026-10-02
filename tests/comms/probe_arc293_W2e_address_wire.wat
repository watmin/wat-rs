;; FM 2-bis / 293.W.2e — address-wire? probes an Address: false = shared memory, true = wire.
;;
;; RED at HEAD: :wat::kernel::address-wire? is UnknownFunction.
;; GREEN after 2e: [false true].

(:wat::core::defn :probe::compute [] -> (wat.type/Vector :- [wat.type/bool])
  (:wat::core::let
    [tb (:wat::kernel::listener (:wat::spawn::thread) wat.type/i64 wat.type/i64)
     pb (:wat::kernel::listener (:wat::spawn::process) wat.type/i64 wat.type/i64)
     ta (:wat::spawn::Bound/address tb)
     pa (:wat::spawn::Bound/address pb)]
    (wat.type/Vector :- [wat.type/bool]
      (:wat::kernel::address-wire? ta)
      (:wat::kernel::address-wire? pa))))
