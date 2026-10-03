(wat.core/defrecord probe/Rec [alpha :- wat.type/i64])

(wat.core/defenum probe/Box wat.enum/Pure
  :Full [payload :- wat.type/i64]
  :Empty [])

(wat.core/typealias probe/Count wat.type/i64)

(wat.core/defn user/main [] :- wat.type/nil
  (wat.kernel/pprintln (wat.runtime/type-of probe/Box)))
