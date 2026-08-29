;; tests/value/wat_arc221_char_atomization.wat — co-located fixture for the sibling probe (.rs).
;; Slurped via startup_beside(file!()). Each function covers one sub-call from the three probes.
;; No :user::main needed — startup_beside loads defns; tests call each fn via eval_in_frozen.

;; ─── Probe 1 — char atom round-trip + distinctness ───────────────────────────

(:wat::core::defn :t::p1-same [] -> :wat::core::bool
  (:wat::core::let
    [atom-a1  (:wat::holon::to-holon \a)
     atom-a2  (:wat::holon::to-holon \a)]
    (:wat::core::= atom-a1 atom-a2)))

(:wat::core::defn :t::p1-diff [] -> :wat::core::bool
  (:wat::core::let
    [atom-a  (:wat::holon::to-holon \a)
     atom-b  (:wat::holon::to-holon \b)
     eq      (:wat::core::= atom-a atom-b)]
    (:wat::core::not eq)))

(:wat::core::defn :t::p1-not-i64 [] -> :wat::core::bool
  (:wat::core::let
    [atom-char  (:wat::holon::to-holon \a)
     atom-int   (:wat::holon::to-holon 97)
     eq         (:wat::core::= atom-char atom-int)]
    (:wat::core::not eq)))

;; ─── Probe 2 — (HashMap :- [char i64]) insert + lookup ────────────────────────────

(:wat::core::defn :t::p2-a-val [] -> :wat::core::i64
  (:wat::core::let
    [tally   (:wat::core::HashMap :- [:wat::core::char :wat::core::i64])
     tally2  (:wat::hashmap::assoc tally \a 3)
     tally3  (:wat::hashmap::assoc tally2 \b 7)]
    (:wat::core::match (:wat::hashmap::get tally3 \a) 
      ((:wat::core::Some v) v)
      (_ -1))))

(:wat::core::defn :t::p2-b-val [] -> :wat::core::i64
  (:wat::core::let
    [tally   (:wat::core::HashMap :- [:wat::core::char :wat::core::i64])
     tally2  (:wat::hashmap::assoc tally \a 3)
     tally3  (:wat::hashmap::assoc tally2 \b 7)]
    (:wat::core::match (:wat::hashmap::get tally3 \b) 
      ((:wat::core::Some v) v)
      (_ -1))))

(:wat::core::defn :t::p2-len [] -> :wat::core::i64
  (:wat::core::let
    [tally   (:wat::core::HashMap :- [:wat::core::char :wat::core::i64])
     tally2  (:wat::hashmap::assoc tally \a 3)
     tally3  (:wat::hashmap::assoc tally2 \b 7)]
    (:wat::hashmap::length tally3)))

;; ─── Probe 3 — (HashSet :- [char]) insert + contains? ──────────────────────────────

(:wat::core::defn :t::p3-has-a [] -> :wat::core::bool
  (:wat::core::let
    [vowels (:wat::core::HashSet :- [:wat::core::char] \a \e \i \o \u)]
    (:wat::core::contains? vowels \a)))

(:wat::core::defn :t::p3-has-e [] -> :wat::core::bool
  (:wat::core::let
    [vowels (:wat::core::HashSet :- [:wat::core::char] \a \e \i \o \u)]
    (:wat::core::contains? vowels \e)))

(:wat::core::defn :t::p3-no-z [] -> :wat::core::bool
  (:wat::core::let
    [vowels (:wat::core::HashSet :- [:wat::core::char] \a \e \i \o \u)
     found  (:wat::core::contains? vowels \z)]
    (:wat::core::not found)))

(:wat::core::defn :t::p3-len [] -> :wat::core::i64
  (:wat::core::let
    [vowels (:wat::core::HashSet :- [:wat::core::char] \a \e \i \o \u)]
    (:wat::hashset::length vowels)))
