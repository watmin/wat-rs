;; Scratch probe — arc 255 Stone "the record family gets homes — ALL SEVEN".
;;
;; BRIEF:  docs/arc/2026/06/255-builtin-registry/BRIEF-STONE-the-record-family.md
;; DESIGN: docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-the-record-family.md
;;
;; Homes all seven aggregate verbs as thin `#[wat_intrinsic]` delegates in
;; `src/intrinsic/record.rs` over their pre-existing named fns in `src/runtime.rs` (no body
;; moves, STOP-4): `Record/field-at` (already homed, arc 255 Stone A-2-ii-b-0), `to-record`,
;; `record->map`, `Record/assoc`, `Record/same-data?`, `struct-field`, `struct-new`, `variant`.
;;
;; ⛔ The struct pair (`struct-new`/`struct-field`) is IN SCOPE — an earlier DESIGN draft parked
;; it on a contradiction that does not exist (`accessor_meta`'s first guard keys on a slash;
;; `:wat::core::struct-field` has none) and a census that never applied. See
;; `255-struct-field-is-a-constant-projection.wat` for the measured evidence that a struct field
;; read is a constant projection even when the field holds a live, mutable handle.
;;
;; `@Totality` is measured PER VERB (not copied across the family): all seven land `Partial` —
;; each raises on a value inside its declared domain that the domain-level gate does not by
;; itself exclude (an unregistered record/struct/enum class, an unknown field/variant name, an
;; out-of-range index, or a write whose new value's type disagrees with the old field's). See
;; `src/intrinsic/record.rs`'s per-verb "Totality ground" for the cited `src/runtime.rs` line.
;;
;; The debt-ledger prediction (`FROZEN_CHECKER_DEBT_LEDGER`, `src/intrinsic/mod.rs`) is the
;; orthogonal axis — whether `check.rs` carries an `env.register()` TypeScheme. MIXED, both
;; directions measured: `Record/assoc`/`Record/same-data?`/`record->map` carry one
;; (`check.rs:21236/21259/21275`), so need NO ledger row; `to-record`/`variant`/`struct-new`/
;; `struct-field` carry none, so each needs one — `FROZEN_CHECKER_DEBT_LEDGER` 64 -> 68.
;; `KNOWN_UNREVIEWED` (`src/rete/purity.rs`) loses all seven names: 41 -> 34.
;;
;;   section 1 — behaviour unchanged: each of the seven, exercised on a record/struct/enum built
;;               for this probe, still returns exactly what it returned before this stone.
;;   section 2 — `metadata-of` for all seven, against the PRE-EXISTING `target/release/wat`
;;               binary (predates this stone's Rust changes, per the rider's brief): expect
;;               `:None` for all seven (this stone's registrations are not compiled into it) —
;;               contrast the collection-readers probe's `Some` for `assoc`/`conj`/`drop`/`take`,
;;               whose OWN prior stone IS already baked into this same binary. See the rider's
;;               report for what this binary actually printed.

;; `to-record` needs a receiver that structurally satisfies a REGISTERED surface — rather than
;; declare a new one (and its satisfaction edge) just for this probe, project the pre-existing
;; `:wat::core::Fault` (wat/core.wat) into the pre-existing `:wat::core::Error` surface it
;; already, load-bearingly, satisfies (`runtime.rs`'s `fault_from_runtime_error` relies on
;; exactly this edge).
(:wat::core::defrecord :probe255rf::FieldAtEx [sk <- :wat::core::i64])
(:wat::core::defrecord :probe255rf::ToMapEx [sk <- :wat::core::i64])
(:wat::core::defrecord :probe255rf::AssocEx [sk <- :wat::core::i64])
(:wat::core::defrecord :probe255rf::PtEx [sk <- :wat::core::i64])
(:wat::core::defrecord :probe255rf::CoordEx [sk <- :wat::core::i64])
(:wat::core::defstruct :probe255rf::StructFieldEx [sk <- :wat::core::i64])
(:wat::core::defstruct :probe255rf::StructNewEx [sk <- :wat::core::i64])
(:wat::core::defenum :probe255rf::VariantEx :wat::enum::Pure :V [sk <- :wat::core::i64])

;; ★ NO `:user::`/`:probe255rf::` wrapper fn for the metadata-of calls below — the
;; collection-readers probe's own note applies here too: `Record/assoc`/`Record/same-data?`/
;; `record->map` already carry a registered `env.register()` TypeScheme in THIS binary (measured
;; §5 of the brief — pre-existing, not new). A bare `:wat::core::Record/assoc` passed through an
;; ordinary `:wat::core::keyword`-typed parameter resolves against that scheme first and infers a
;; FUNCTION type, not a keyword literal, and fails to check. `:wat::runtime::metadata-of`'s own
;; hand-written inference arm accepts the bare FQDN directly in its callee argument position, so
;; every one of the seven is called through it inline, uniformly (same shape for all seven, not
;; just the three with schemes, so a reader cannot mistake the inlining for a per-verb special
;; case).

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println "── section 1 — behaviour unchanged ──")
    (:wat::kernel::println
      (:wat::string::concat "Record/field-at              => "
        (:wat::edn::write (:wat::core::Record/field-at (:probe255rf::FieldAtEx :sk 7) 0))))
    (:wat::kernel::println
      (:wat::string::concat "to-record                    => "
        (:wat::edn::write (:wat::core::Record/field-at
          (:wat::core::to-record (:wat::core::Fault/of "boom") :wat::core::Error) 0))))
    (:wat::kernel::println
      (:wat::string::concat "record->map                  => "
        (:wat::edn::write (:wat::hashmap::get (:wat::core::record->map (:probe255rf::ToMapEx :sk 3)) :sk))))
    (:wat::kernel::println
      (:wat::string::concat "Record/assoc                 => "
        (:wat::edn::write (:wat::core::Record/field-at
          (:wat::core::Record/assoc (:probe255rf::AssocEx :sk 1) :sk 9) 0))))
    (:wat::kernel::println
      (:wat::string::concat "Record/same-data?            => "
        (:wat::edn::write (:wat::core::Record/same-data? (:probe255rf::PtEx :sk 0) (:probe255rf::CoordEx :sk 0)))))
    (:wat::kernel::println
      (:wat::string::concat "struct-field                 => "
        (:wat::edn::write (:wat::core::struct-field (:probe255rf::StructFieldEx :sk 5) 0))))
    (:wat::kernel::println
      (:wat::string::concat "struct-new + struct-field     => "
        (:wat::edn::write (:wat::core::struct-field (:wat::core::struct-new :probe255rf::StructNewEx 4) 0))))
    (:wat::kernel::println
      (:wat::string::concat "variant (via = against sugar) => "
        (:wat::edn::write (:wat::core::= (:wat::core::variant :probe255rf::VariantEx :V 6)
                                          (:probe255rf::VariantEx::V 6)))))
    (:wat::kernel::println "── section 2 — metadata-of :totality (all seven, this binary) ──")
    (:wat::kernel::println
      (:wat::string::concat "Record/field-at    :totality => "
        (:wat::core::match (:wat::runtime::metadata-of :wat::core::Record/field-at)
          ((:wat::core::Some hm)
           (:wat::core::match (:wat::hashmap::get hm :totality)
             ((:wat::core::Some t) (:wat::edn::write t))
             (:None "registered, but no :totality key (unexpected)")))
          (:None "None (not registered in this binary)"))))
    (:wat::kernel::println
      (:wat::string::concat "to-record          :totality => "
        (:wat::core::match (:wat::runtime::metadata-of :wat::core::to-record)
          ((:wat::core::Some hm)
           (:wat::core::match (:wat::hashmap::get hm :totality)
             ((:wat::core::Some t) (:wat::edn::write t))
             (:None "registered, but no :totality key (unexpected)")))
          (:None "None (not registered in this binary)"))))
    (:wat::kernel::println
      (:wat::string::concat "record->map        :totality => "
        (:wat::core::match (:wat::runtime::metadata-of :wat::core::record->map)
          ((:wat::core::Some hm)
           (:wat::core::match (:wat::hashmap::get hm :totality)
             ((:wat::core::Some t) (:wat::edn::write t))
             (:None "registered, but no :totality key (unexpected)")))
          (:None "None (not registered in this binary)"))))
    (:wat::kernel::println
      (:wat::string::concat "Record/assoc       :totality => "
        (:wat::core::match (:wat::runtime::metadata-of :wat::core::Record/assoc)
          ((:wat::core::Some hm)
           (:wat::core::match (:wat::hashmap::get hm :totality)
             ((:wat::core::Some t) (:wat::edn::write t))
             (:None "registered, but no :totality key (unexpected)")))
          (:None "None (not registered in this binary)"))))
    (:wat::kernel::println
      (:wat::string::concat "Record/same-data?  :totality => "
        (:wat::core::match (:wat::runtime::metadata-of :wat::core::Record/same-data?)
          ((:wat::core::Some hm)
           (:wat::core::match (:wat::hashmap::get hm :totality)
             ((:wat::core::Some t) (:wat::edn::write t))
             (:None "registered, but no :totality key (unexpected)")))
          (:None "None (not registered in this binary)"))))
    (:wat::kernel::println
      (:wat::string::concat "struct-field       :totality => "
        (:wat::core::match (:wat::runtime::metadata-of :wat::core::struct-field)
          ((:wat::core::Some hm)
           (:wat::core::match (:wat::hashmap::get hm :totality)
             ((:wat::core::Some t) (:wat::edn::write t))
             (:None "registered, but no :totality key (unexpected)")))
          (:None "None (not registered in this binary)"))))
    (:wat::kernel::println
      (:wat::string::concat "struct-new         :totality => "
        (:wat::core::match (:wat::runtime::metadata-of :wat::core::struct-new)
          ((:wat::core::Some hm)
           (:wat::core::match (:wat::hashmap::get hm :totality)
             ((:wat::core::Some t) (:wat::edn::write t))
             (:None "registered, but no :totality key (unexpected)")))
          (:None "None (not registered in this binary)"))))
    (:wat::kernel::println
      (:wat::string::concat "variant            :totality => "
        (:wat::core::match (:wat::runtime::metadata-of :wat::core::variant)
          ((:wat::core::Some hm)
           (:wat::core::match (:wat::hashmap::get hm :totality)
             ((:wat::core::Some t) (:wat::edn::write t))
             (:None "registered, but no :totality key (unexpected)")))
          (:None "None (not registered in this binary)"))))
    nil))
