;; tests/value/wat_arc221_keyword_nil_tag_atomization.wat — co-located fixture for the sibling probe.
;; Slurped via startup_beside(file!()). Each function covers one sub-call from the six probes.
;; No :user::main needed — startup_beside loads defns; tests call each fn via eval_in_frozen.

;; ─── Probe 1 — keyword atom round-trip + distinctness from String ─────────────

(:wat::core::defn :t::p1-same [] -> :wat::core::bool
  (:wat::core::let
    [atom-foo1  (:wat::holon::to-holon :foo)
     atom-foo2  (:wat::holon::to-holon :foo)]
    (:wat::core::= atom-foo1 atom-foo2)))

(:wat::core::defn :t::p1-diff [] -> :wat::core::bool
  (:wat::core::let
    [atom-foo  (:wat::holon::to-holon :foo)
     atom-bar  (:wat::holon::to-holon :bar)
     eq        (:wat::core::= atom-foo atom-bar)]
    (:wat::core::not eq)))

(:wat::core::defn :t::p1-not-string [] -> :wat::core::bool
  (:wat::core::let
    [atom-kw  (:wat::holon::to-holon :foo)
     atom-str (:wat::holon::to-holon "foo")
     eq       (:wat::core::= atom-kw atom-str)]
    (:wat::core::not eq)))

;; ─── Probe 2 — nil atom round-trip + distinct from keyword :nil ──────────────

(:wat::core::defn :t::p2-same [] -> :wat::core::bool
  (:wat::core::let
    [atom-nil1  (:wat::holon::to-holon nil)
     atom-nil2  (:wat::holon::to-holon nil)]
    (:wat::core::= atom-nil1 atom-nil2)))

(:wat::core::defn :t::p2-diff [] -> :wat::core::bool
  (:wat::core::let
    [atom-nil  (:wat::holon::to-holon nil)
     atom-knil (:wat::holon::to-holon :nil)
     eq        (:wat::core::= atom-nil atom-knil)]
    (:wat::core::not eq)))

;; ─── Probe 3 — Uuid atom round-trip — closes arc 207 ─────────────────────────

(:wat::core::defn :t::p3-same [] -> :wat::core::bool
  (:wat::core::let
    [ns    (:wat::core::Uuid/nil)
     u1    (:wat::core::Uuid/v5 ns "hello")
     u2    (:wat::core::Uuid/v5 ns "hello")
     a1    (:wat::holon::to-holon u1)
     a2    (:wat::holon::to-holon u2)]
    (:wat::core::= a1 a2)))

(:wat::core::defn :t::p3-diff [] -> :wat::core::bool
  (:wat::core::let
    [ns    (:wat::core::Uuid/nil)
     u1    (:wat::core::Uuid/v5 ns "hello")
     u2    (:wat::core::Uuid/v5 ns "world")
     a1    (:wat::holon::to-holon u1)
     a2    (:wat::holon::to-holon u2)
     eq    (:wat::core::= a1 a2)]
    (:wat::core::not eq)))

;; ─── Probe 4 — (HashMap :- [keyword i64]) insert + lookup ─────────────────────────

(:wat::core::defn :t::p4-a-val [] -> :wat::core::i64
  (:wat::core::let
    [m   (:wat::core::HashMap :wat::core::keyword :wat::core::i64)
     m2  (:wat::core::HashMap/assoc m :tag-a 10)
     m3  (:wat::core::HashMap/assoc m2 :tag-b 20)]
    (:wat::core::match (:wat::core::HashMap/get m3 :tag-a) 
      ((:wat::core::Some v) v)
      (_ -1))))

(:wat::core::defn :t::p4-b-val [] -> :wat::core::i64
  (:wat::core::let
    [m   (:wat::core::HashMap :wat::core::keyword :wat::core::i64)
     m2  (:wat::core::HashMap/assoc m :tag-a 10)
     m3  (:wat::core::HashMap/assoc m2 :tag-b 20)]
    (:wat::core::match (:wat::core::HashMap/get m3 :tag-b) 
      ((:wat::core::Some v) v)
      (_ -1))))

(:wat::core::defn :t::p4-len [] -> :wat::core::i64
  (:wat::core::let
    [m   (:wat::core::HashMap :wat::core::keyword :wat::core::i64)
     m2  (:wat::core::HashMap/assoc m :tag-a 10)
     m3  (:wat::core::HashMap/assoc m2 :tag-b 20)]
    (:wat::core::HashMap/length m3)))

;; ─── Probe 5 — (HashSet :- [keyword]) insert + contains? ───────────────────────────

(:wat::core::defn :t::p5-has-foo [] -> :wat::core::bool
  (:wat::core::let
    [tags (:wat::core::HashSet :wat::core::keyword :foo :bar :baz)]
    (:wat::core::contains? tags :foo)))

(:wat::core::defn :t::p5-has-bar [] -> :wat::core::bool
  (:wat::core::let
    [tags (:wat::core::HashSet :wat::core::keyword :foo :bar :baz)]
    (:wat::core::contains? tags :bar)))

(:wat::core::defn :t::p5-no-unknown [] -> :wat::core::bool
  (:wat::core::let
    [tags  (:wat::core::HashSet :wat::core::keyword :foo :bar :baz)
     found (:wat::core::contains? tags :unknown)]
    (:wat::core::not found)))

(:wat::core::defn :t::p5-len [] -> :wat::core::i64
  (:wat::core::let
    [tags (:wat::core::HashSet :wat::core::keyword :foo :bar :baz)]
    (:wat::core::HashSet/length tags)))

;; ─── Probe 6 — (HashMap :- [Uuid String]) insert + lookup — closes arc 207 ────────

(:wat::core::defn :t::p6-retrieved [] -> :wat::core::String
  (:wat::core::let
    [ns   (:wat::core::Uuid/nil)
     u1   (:wat::core::Uuid/v5 ns "hello")
     m    (:wat::core::HashMap :wat::core::Uuid :wat::core::String)
     m2   (:wat::core::HashMap/assoc m u1 "world-entry")]
    (:wat::core::match (:wat::core::HashMap/get m2 u1) 
      ((:wat::core::Some v) v)
      (_ "NOT-FOUND"))))

(:wat::core::defn :t::p6-not-found [] -> :wat::core::String
  (:wat::core::let
    [ns   (:wat::core::Uuid/nil)
     u1   (:wat::core::Uuid/v5 ns "hello")
     u2   (:wat::core::Uuid/v5 ns "world")
     m    (:wat::core::HashMap :wat::core::Uuid :wat::core::String)
     m2   (:wat::core::HashMap/assoc m u1 "hello-entry")]
    (:wat::core::match (:wat::core::HashMap/get m2 u2) 
      ((:wat::core::Some v) v)
      (_ "NOT-FOUND"))))
