;; Arc 278 proof-probe (2026-07-05): can a defenum variant hold a defrecord as its typed payload?
;; (the shape behind :wat::sqlite'::Error — errors-as-record inside an enum). Run: `cargo wat <this>`.
;; PROVES: enum variants are tagged MAPS; a variant field can be a record; round-trips as EDN.
;; Output:
;;   #probe/Error.Err1 {:err #probe/Err1 {:msg "boom"}}    ;; 1-field variant -> map holding the record
;;   #probe/Error.Pair {:code 42 :msg "boom"}              ;; 2-field variant -> named map

(:wat::core::defrecord :probe::Err1 [msg <- :wat::core::String])
(:wat::core::defrecord :probe::Err2 [code <- :wat::core::i64])

(:wat::core::defenum :probe::Error :wat::enum::Pure
  :Err1 [err <- :probe::Err1]                              ;; variant holds a RECORD
  :Err2 [err <- :probe::Err2]
  :Pair [code <- :wat::core::i64  msg <- :wat::core::String])   ;; a TWO-field variant (positional tuple)

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:probe::Error::Err1 (:probe::Err1 "boom")))    ;; 1 field -> [record]
  (:wat::kernel::println (:probe::Error::Pair 42 "boom")))               ;; 2 fields -> [42 "boom"]
