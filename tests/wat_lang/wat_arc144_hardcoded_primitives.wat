;; tests/wat_lang/wat_arc144_hardcoded_primitives.wat — co-located fixture.
;; Arc 144 slice 3 — TypeScheme callable-fingerprints for 15 hardcoded callables.
;; Each :t:: function returns bool (Some→true / None→false) or String.

;; ─── signature-of-defn returns Some for hardcoded callables ─────────────────

(:wat::core::defn :t::sig-length [] -> :wat::core::bool
  (:wat::core::match
    (:wat::runtime::signature-of-defn :wat::core::length)
    
    ((:wat::core::Some _) true)
    (:wat::core::None    false)))

(:wat::core::defn :t::sig-empty-q [] -> :wat::core::bool
  (:wat::core::match
    (:wat::runtime::signature-of-defn :wat::core::empty?)
    
    ((:wat::core::Some _) true)
    (:wat::core::None    false)))

(:wat::core::defn :t::sig-contains-q [] -> :wat::core::bool
  (:wat::core::match
    (:wat::runtime::signature-of-defn :wat::core::contains?)
    
    ((:wat::core::Some _) true)
    (:wat::core::None    false)))

(:wat::core::defn :t::sig-get [] -> :wat::core::bool
  (:wat::core::match
    (:wat::runtime::signature-of-defn :wat::core::get)
    
    ((:wat::core::Some _) true)
    (:wat::core::None    false)))

(:wat::core::defn :t::sig-conj [] -> :wat::core::bool
  (:wat::core::match
    (:wat::runtime::signature-of-defn :wat::core::conj)
    
    ((:wat::core::Some _) true)
    (:wat::core::None    false)))

(:wat::core::defn :t::sig-assoc [] -> :wat::core::bool
  (:wat::core::match
    (:wat::runtime::signature-of-defn :wat::core::assoc)
    
    ((:wat::core::Some _) true)
    (:wat::core::None    false)))

(:wat::core::defn :t::sig-dissoc [] -> :wat::core::bool
  (:wat::core::match
    (:wat::runtime::signature-of-defn :wat::core::dissoc)
    
    ((:wat::core::Some _) true)
    (:wat::core::None    false)))

(:wat::core::defn :t::sig-keys [] -> :wat::core::bool
  (:wat::core::match
    (:wat::runtime::signature-of-defn :wat::core::keys)
    
    ((:wat::core::Some _) true)
    (:wat::core::None    false)))

(:wat::core::defn :t::sig-values [] -> :wat::core::bool
  (:wat::core::match
    (:wat::runtime::signature-of-defn :wat::core::values)
    
    ((:wat::core::Some _) true)
    (:wat::core::None    false)))

(:wat::core::defn :t::sig-vector [] -> :wat::core::bool
  (:wat::core::match
    (:wat::runtime::signature-of-defn :wat::core::Vector)
    
    ((:wat::core::Some _) true)
    (:wat::core::None    false)))

(:wat::core::defn :t::sig-tuple [] -> :wat::core::bool
  (:wat::core::match
    (:wat::runtime::signature-of-defn :wat::core::Tuple)
    
    ((:wat::core::Some _) true)
    (:wat::core::None    false)))

(:wat::core::defn :t::sig-hashmap [] -> :wat::core::bool
  (:wat::core::match
    (:wat::runtime::signature-of-defn :wat::core::HashMap)
    
    ((:wat::core::Some _) true)
    (:wat::core::None    false)))

(:wat::core::defn :t::sig-hashset [] -> :wat::core::bool
  (:wat::core::match
    (:wat::runtime::signature-of-defn :wat::core::HashSet)
    
    ((:wat::core::Some _) true)
    (:wat::core::None    false)))

(:wat::core::defn :t::sig-concat [] -> :wat::core::bool
  (:wat::core::match
    (:wat::runtime::signature-of-defn :wat::core::concat)
    
    ((:wat::core::Some _) true)
    (:wat::core::None    false)))

(:wat::core::defn :t::sig-string-concat [] -> :wat::core::bool
  (:wat::core::match
    (:wat::runtime::signature-of-defn :wat::string::concat)
    
    ((:wat::core::Some _) true)
    (:wat::core::None    false)))

;; ─── body-of returns None for hardcoded primitives ───────────────────────────

(:wat::core::defn :t::body-length-none [] -> :wat::core::bool
  (:wat::core::match
    (:wat::runtime::body-of :wat::core::length)
    
    ((:wat::core::Some _) false)
    (:wat::core::None    true)))

;; ─── lookup-define renders the synthesised primitive form ────────────────────

(:wat::core::defn :t::lookup-vector-length-render [] -> :wat::core::String
  (:wat::core::let [def-opt  (:wat::runtime::lookup-define :wat::vec::length)
                   rendered (:wat::edn::write def-opt)]
    rendered))
