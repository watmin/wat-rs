;; tests/wat_lang/wat_idempotent_redeclare.wat — co-located fixture for the sibling probe (.rs).
;; Covers all positive (startup-ok) tests: byte-equivalent re-registration is a no-op
;; for typealias, defn, defmacro, and the shim double-register pattern.

; typealias byte-equivalent — double registration is idempotent
(wat.core/typealias my/Amount wat.type/f64)
(wat.core/typealias my/Amount wat.type/f64)

; defn byte-equivalent — double registration is idempotent
(wat.core/defn my/add-one [a :- wat.type/i64] :- wat.type/i64 (wat.core/+ a 1))
(wat.core/defn my/add-one [a :- wat.type/i64] :- wat.type/i64 (wat.core/+ a 1))

; defmacro byte-equivalent — double registration is idempotent
(wat.core/defmacro my/ident [x :- wat.type/AST] :- wat.type/AST `~x)
(wat.core/defmacro my/ident [x :- wat.type/AST] :- wat.type/AST `~x)

; shim double-register pattern — typealias delivered via two paths
(wat.core/typealias lab.candles/Stream wat.type/i64)
(wat.core/typealias lab.candles/Stream wat.type/i64)
