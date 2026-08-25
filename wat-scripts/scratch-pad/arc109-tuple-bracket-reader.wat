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

;; 1-ary bracketed tuple — the rung the keyword surface can only spell with a trailing comma
;; (`:(wat::core::i64,)`; bare `:(A)` is Rust GROUPING and collapses to `A`). The form surface has
;; no such ambiguity. Measured distinct from a scalar: passing a bare 7 here is a TypeMismatch.
(:wat::core::defn :user::one-ary [p <- (wat.type/Tuple [wat.type/i64])] -> :wat::core::i64
  0)

;; 2-ary bracketed tuple as a PARAM type
(:wat::core::defn :user::takes-pair [p <- (wat.type/Tuple [wat.type/i64 wat.type/String])] -> :wat::core::i64
  1)

;; bracketed tuple as a RETURN type, with a nested parametric inside it
(:wat::core::defn :user::nested [] -> (wat.type/Tuple [(wat.type/Vector [wat.type/i64]) wat.type/String])
  (:wat::core::Tuple (:wat::core::Vector [:wat::core::i64] 1 2) "s"))

;; EMPTY bracketed tuple — `(wat.type/Tuple [])`. This is LEGAL, WRITABLE source: the form surface
;; can spell the empty tuple even though the keyword surface `:()` is retired. Today it is still the
;; SAME TYPE as `nil` (measured: a `nil` argument satisfies both a `(wat.type/Tuple [])` param and a
;; `:wat::core::nil` param), which is exactly the identity the builder's `nil != ()` ruling splits.
(:wat::core::defn :user::empty-tuple [] -> (wat.type/Tuple [])
  (:wat::kernel::println "e"))

;; the FLAT form still reads (the c09 contract) — the reader accepts both; only the WRITER changes
(:wat::core::defn :user::flat-still-reads [p <- (wat.type/Tuple wat.type/i64 wat.type/String)] -> :wat::core::i64
  2)

;; Arc 109 Stone ②-i-b — the `:-`-marked spelling of the same rungs above. `:-` declares "the
;; thing on the left is parameterized by the thing on the right"; this is that production
;; landing at Tuple's args-tail (`parse_type_form`'s new `[Keyword(":-"), Vector(inner), …]`
;; arm, src/types.rs). Same rungs, `:-` this time — dual-read alongside the unmarked forms above.

;; 1-ary bracketed tuple, `:-`-marked
(:wat::core::defn :user::one-ary-colon [p <- (wat.type/Tuple :- [wat.type/i64])] -> :wat::core::i64
  0)

;; 2-ary bracketed tuple as a PARAM type, `:-`-marked
(:wat::core::defn :user::takes-pair-colon [p <- (wat.type/Tuple :- [wat.type/i64 wat.type/String])] -> :wat::core::i64
  1)

;; bracketed tuple as a RETURN type, with a nested `:-`-marked parametric inside it
(:wat::core::defn :user::nested-colon [] -> (wat.type/Tuple :- [(wat.type/Vector :- [wat.type/i64]) wat.type/String])
  (:wat::core::Tuple (:wat::core::Vector [:wat::core::i64] 1 2) "s"))

;; EMPTY `:-`-marked bracketed tuple — `(wat.type/Tuple :- [])`. The empty rung is a first-class
;; member of the arity ladder, not a defensive branch — and, unlike the unmarked
;; `(wat.type/Tuple [])` above, it is unambiguously a type declaration: `:-` never sniffs.
(:wat::core::defn :user::empty-tuple-colon [] -> (wat.type/Tuple :- [])
  (:wat::kernel::println "e"))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:wat::string::interpolate "pair={a} flat={b}"
    :a (:user::takes-pair (:wat::core::Tuple 1 "x"))
    :b (:user::flat-still-reads (:wat::core::Tuple 2 "y")))))
