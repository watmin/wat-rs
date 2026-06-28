;; probe_arc278_value_universal_top_widen.wat — Surface B widen (positive). RED at HEAD.
;; A record field typed :wat::core::Value accepts both i64 and String (widening is free).

(:wat::core::defrecord :my::Box [slot <- :wat::core::Value])
(:wat::core::defn :my::box-int [] -> :my::Box (:my::Box 7))
(:wat::core::defn :my::box-str [] -> :my::Box (:my::Box "hi"))
