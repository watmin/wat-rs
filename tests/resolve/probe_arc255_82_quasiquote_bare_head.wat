;; 255.82 — a bare head inside quasiquote is data. It must check and run.
(wat.core/defn user/compute [] :- wat.type/nil
  (wat.core/do
    (wat.core/quasiquote (foozle 1))
    nil))
