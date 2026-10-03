(wat.core/defn fix/kwargs-pos
  [f :- [wat.type/keyword
         wat.type/keyword
         wat.type/keyword
         wat.type/keyword
         (wat.type/Vector :- [wat.type/i64])
         wat.type/keyword
         (wat.type/Vector :- [wat.type/i64])
         wat.type/keyword
         (wat.type/Vector :- [wat.type/i64])
         :-> wat.type/nil]]
  :- wat.type/nil
  (f wat-tests/recorder :satisfies wat-tests/Recorder :durable [] :ephemeral [] :impls []))
