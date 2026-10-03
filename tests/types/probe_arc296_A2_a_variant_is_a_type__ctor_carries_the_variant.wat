;; SUBJECT — the posterity example. The CTOR must carry the variant type.
(wat.core/defenum usr/Box :- [T] wat.enum/Pure
  :Full  [inside :- T]
  :Empty [])
(wat.core/defn user/takes-full [b :- (usr/Box.Full :- [wat.type/i64])] :- wat.type/i64 7)
(wat.core/defn user/main [] :- wat.type/nil
  (wat.core/let [full-box (usr/Box.Full {:inside 42})]
    (wat.kernel/println (user/takes-full full-box))))
