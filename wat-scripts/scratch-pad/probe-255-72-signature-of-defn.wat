;; probe-255-72-signature-of-defn.wat — STONE 255.72 site-2 measurement.
;;
;; Can the oracle read a custom accumulator fold's declared parameter type at run time?
;; Uses the SAME real fold the oracle's own differential test uses
;; (tests/rete/probe_arc278_8custom_native_differential.rs: :w::sum-of-squares,
;; declared `[xs <- (:wat::core::PersistentVector :- [:wat::core::i64])] -> :wat::core::i64`).
;; Reads it via `:wat::runtime::signature-of-defn` + `:wat::runtime::extract-arg-types`,
;; printing the resulting type AST's source text.

(wat.core/defn w/sum-of-squares
  [xs :- (wat.type/PersistentVector :- [wat.type/i64])] :- wat.type/i64
  (wat.core/foldl
    (wat.core/fn [acc :- wat.type/i64 x :- wat.type/i64] :- wat.type/i64
      (wat.i64/+ acc (wat.i64/* x x)))
    0 xs))

(wat.core/defn user/main [] :- wat.type/nil
  (wat.core/let
    [sig      (wat.core.Option/expect
                (wat.runtime/signature-of-defn w/sum-of-squares)
                "signature-of-defn: :w::sum-of-squares not found")
     arg-tys  (wat.runtime/extract-arg-types sig)
     n        (wat.core/length arg-tys)
     first-ty (wat.core.Option/expect (wat.core/get arg-tys 0) "no arg-ty 0")
     src      (wat.core/ast->source first-ty)
     ty-kind  (wat.core/ast-kind first-ty)
     ty-ch    (wat.core/ast->children first-ty)
     ty-n     (wat.core/length ty-ch)
     ch-srcs  (wat.core/foldl
                (wat.core/fn [acc :- wat.type/String i :- wat.type/i64] :- wat.type/String
                  (wat.core/let
                    [c (wat.core.Option/expect (wat.core/get ty-ch i) "ch")
                     ck (wat.core/ast-kind c)
                     cs (wat.core/ast->source c)]
                    (wat.string/concat acc (wat.string/concat " |" (wat.string/concat ck (wat.string/concat ":" cs))))))
                "" (wat.core/range 0 ty-n))
     bracket  (wat.core.Option/expect (wat.core/get ty-ch 2) "no bracket child")
     bracket-ch (wat.core/ast->children bracket)
     elem-ty  (wat.core.Option/expect (wat.core/get bracket-ch 0) "no elem ty")
     elem-src (wat.core/ast->source elem-ty)]
    (wat.core/do
      (wat.kernel/println
        (wat.string/concat "elem-ty-src=" elem-src))
      (wat.kernel/println
        (wat.string/concat "n-args=" (wat.string/concat (wat.i64/to-string n)
          (wat.string/concat " first-arg-type-src=" (wat.string/concat src
            (wat.string/concat " ty-kind=" (wat.string/concat ty-kind
              (wat.string/concat " ty-n-children=" (wat.string/concat (wat.i64/to-string ty-n)
                (wat.string/concat " children=" ch-srcs)))))))))))))
