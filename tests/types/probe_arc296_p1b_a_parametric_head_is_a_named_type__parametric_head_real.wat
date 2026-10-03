;; ⛔ THE WIDEST CONTROL — a real parametric head. If this breaks, every generic in the
;; corpus breaks: Option, Result, Vector, HashMap are all parametric annotations.
(wat.core/defn user/f [x :- (wat.core/Option :- [wat.type/i64])] :- wat.type/nil
  (wat.kernel/println "ok"))
(wat.core/defn user/main [] :- wat.type/nil (wat.kernel/println "ok"))
