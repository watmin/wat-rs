;; tests/types/probe_arc294c2a_aggregate_new.wat — co-located fixture (arc 294.c.2a)
;;
;; aggregate-new is the ONE nature-dispatched constructor (294 DESIGN:128). struct /
;; base record / holon record ALL construct via (:wat::core::aggregate-new :T field…);
;; the holon record's hologram is DERIVED in Rust (build_holon_hologram), no precomputed
;; arg. struct-new / Record::of / holon::Record::of die into it (c.2b).
;;
;; RED at HEAD: :wat::core::aggregate-new is unknown → startup/eval fails.
;; GREEN after 294.c.2a: all three natures construct + the holon's hologram measures.

(:wat::core::defstruct  :test::an::ST [a <- :wat::core::i64  b <- :wat::core::i64])
(:wat::core::defrecord  :test::an::BR [a <- :wat::core::i64  b <- :wat::core::i64])
(:wat::holon::defrecord :test::an::HR [a <- :wat::core::i64  b <- :wat::core::i64])

;; struct constructed via aggregate-new → field reads back (= 7)
(:wat::core::defn :user::an-struct-a [] -> :wat::core::i64
  (:test::an::ST/a (:wat::core::aggregate-new :test::an::ST 7 8)))

;; base record constructed via aggregate-new → field reads back (= 8)
(:wat::core::defn :user::an-record-b [] -> :wat::core::i64
  (:test::an::BR/b (:wat::core::aggregate-new :test::an::BR 7 8)))

;; holon record constructed via aggregate-new → field reads back (= 7), hologram derived
(:wat::core::defn :user::an-holon-a [] -> :wat::core::i64
  (:test::an::HR/a (:wat::core::aggregate-new :test::an::HR 7 8)))

;; the DERIVED hologram is correct: cosine of a holon record with itself = 1.0
(:wat::core::defn :user::an-holon-self-cos [] -> :wat::core::f64
  (:wat::core::let [h (:wat::core::aggregate-new :test::an::HR 7 8)]
    (:wat::holon::cosine h h)))

;; the DERIVED hologram is DATA-DEPENDENT (not a constant/empty bundle): two holon
;; records differing only in field b measure < 1.0. Self-cosine alone is trivially
;; 1.0 for any value; this proves the hologram actually encodes the fields.
(:wat::core::defn :user::an-holon-diff-cos [] -> :wat::core::f64
  (:wat::holon::cosine
    (:wat::core::aggregate-new :test::an::HR 7 8)
    (:wat::core::aggregate-new :test::an::HR 7 9)))
