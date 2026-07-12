;; tests/rete/probe_arc278_7exists_native_differential.wat — co-located fixture for the sibling probe (.rs),
;; slurped via startup_beside(file!()). Defines the :w::watched existential rule for exists tests.

(:wat::core::defrecord :w::Station [location <- :wat::core::String])
(:wat::core::defrecord :w::Reading [location <- :wat::core::String  value <- :wat::core::i64])
(:wat::core::defrecord :w::Watched [location <- :wat::core::String])

(:wat::rete::defrule :w::watched
  :when
  [(:w::Station :location (?loc <- :location))
   (:wat::rete::exists (:w::Reading (?loc <- :location)))]
  :then
  (:wat::rete::insert (:w::Watched :location ?loc)))

