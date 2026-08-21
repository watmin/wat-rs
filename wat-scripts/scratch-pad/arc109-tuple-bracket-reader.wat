;; wat-scripts/scratch-pad/arc109-tuple-bracket-reader.wat — arc 109 Stone ②-iii's disconfirming probe.
;;
;; ②-iii brackets the renderer's `TypeExpr::Tuple` arm: `(wat.type/Tuple a b)` → `(wat.type/Tuple [a b])`.
;; A renderer that emits a form the READER cannot take back is a broken round-trip, so the
;; load-bearing question is asked BEFORE the strike: does the type-form reader already accept a
;; BRACKETED Tuple head?
;;
;; It should. `parse_type_node` (src/types.rs:4528) unwraps a lone `WatAST::Vector` args-tail for
;; ANY head — arc 109 ① — and the `Tuple` branch (src/types.rs:4540) reads `args` AFTER that unwrap.
;; So the bracket rule reaches Tuple for free. This probe proves it rather than reading it.
;;
;; This file type-checking AT ALL is the whole assertion — every type annotation below is a
;; bracketed Tuple form in a position the checker must resolve.

;; 2-ary bracketed tuple as a PARAM type
(:wat::core::defn :user::takes-pair [p <- (wat.type/Tuple [wat.type/i64 wat.type/String])] -> :wat::core::i64
  1)

;; bracketed tuple as a RETURN type, with a nested parametric inside it
(:wat::core::defn :user::nested [] -> (wat.type/Tuple [(wat.type/Vector [wat.type/i64]) wat.type/String])
  (:wat::core::Tuple (:wat::core::Vector [:wat::core::i64] 1 2) "s"))

;; EMPTY bracketed tuple — `(wat.type/Tuple [])`. Today `:wat::core::nil` canonicalizes to the
;; 0-tuple, so this return type must unify with a nil-returning body.
(:wat::core::defn :user::empty-tuple [] -> (wat.type/Tuple [])
  (:wat::kernel::println "e"))

;; the FLAT form still reads (the c09 contract) — the reader accepts both; only the WRITER changes
(:wat::core::defn :user::flat-still-reads [p <- (wat.type/Tuple wat.type/i64 wat.type/String)] -> :wat::core::i64
  2)

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:wat::core::string::interpolate "pair={a} flat={b}"
    :a (:user::takes-pair (:wat::core::Tuple 1 "x"))
    :b (:user::flat-still-reads (:wat::core::Tuple 2 "y")))))
