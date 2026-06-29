;; tests/types/probe_arc234_stone2a_record_primitives.wat
;; Co-located fixture for probe_arc234_stone2a_record_primitives.rs (arc 234 Stone 234.2a).
;; Each :user::probe-N function corresponds to its Rust probe contract.

;; ─── Probe 1: construction returns :wat::core::Record ──────────────────────────────
(:wat::core::defn :user::probe-1 [] -> :wat::holon::Record
  (:wat::holon::Record::of
    :myapp::Voltage
    [5.0]
    (:wat::holon::Bind
      (:wat::holon::Atom (:wat::holon::to-holon "myapp::Voltage"))
      (:wat::core::Result/expect
        (:wat::holon::Bundle
          [(:wat::holon::Bind
             (:wat::holon::Atom (:wat::holon::to-holon "magnitude"))
             (:wat::holon::Atom (:wat::holon::to-holon 5.0)))])
        "Bundle failed in Probe 1"))))

;; ─── Probe 2: :wat::core::type returns class_fqdn ────────────────────────────
(:wat::core::defn :user::probe-2 [] -> :wat::core::String
  (:wat::core::let
    [v (:wat::holon::Record::of
         :myapp::Voltage
         [5.0]
         (:wat::holon::Bind
           (:wat::holon::Atom (:wat::holon::to-holon "myapp::Voltage"))
           (:wat::core::Result/expect
             (:wat::holon::Bundle
               [(:wat::holon::Bind
                  (:wat::holon::Atom (:wat::holon::to-holon "magnitude"))
                  (:wat::holon::Atom (:wat::holon::to-holon 5.0)))])
             "Bundle failed in Probe 2")))]
    (:wat::core::type v)))

;; ─── Probe 3: single-field struct_form ───────────────────────────────────────
(:wat::core::defn :user::probe-3 [] -> :wat::holon::Record
  (:wat::holon::Record::of
    :myapp::Voltage
    [42.0]
    (:wat::holon::Bind
      (:wat::holon::Atom (:wat::holon::to-holon "myapp::Voltage"))
      (:wat::core::Result/expect
        (:wat::holon::Bundle
          [(:wat::holon::Bind
             (:wat::holon::Atom (:wat::holon::to-holon "magnitude"))
             (:wat::holon::Atom (:wat::holon::to-holon 42.0)))])
        "Bundle failed in Probe 3"))))

;; ─── Probe 4: multi-field construction ───────────────────────────────────────
(:wat::core::defn :user::probe-4 [] -> :wat::holon::Record
  (:wat::holon::Record::of
    :myapp::Point
    [3 4]
    (:wat::holon::Bind
      (:wat::holon::Atom (:wat::holon::to-holon "myapp::Point"))
      (:wat::core::Result/expect
        (:wat::holon::Bundle
          [(:wat::holon::Bind
             (:wat::holon::Atom (:wat::holon::to-holon "x"))
             (:wat::holon::Atom (:wat::holon::to-holon 3)))
           (:wat::holon::Bind
             (:wat::holon::Atom (:wat::holon::to-holon "y"))
             (:wat::holon::Atom (:wat::holon::to-holon 4)))])
        "Bundle failed in Probe 4"))))

;; ─── Probe 5: Record/field-at positional access ──────────────────────────────
(:wat::core::defn :user::probe-5 [] -> :wat::core::i64
  (:wat::core::let
    [v (:wat::holon::Record::of
         :myapp::Point
         [3 4]
         (:wat::holon::Bind
           (:wat::holon::Atom (:wat::holon::to-holon "myapp::Point"))
           (:wat::core::Result/expect
             (:wat::holon::Bundle
               [(:wat::holon::Bind
                  (:wat::holon::Atom (:wat::holon::to-holon "x"))
                  (:wat::holon::Atom (:wat::holon::to-holon 3)))
                (:wat::holon::Bind
                  (:wat::holon::Atom (:wat::holon::to-holon "y"))
                  (:wat::holon::Atom (:wat::holon::to-holon 4)))])
             "Bundle failed in Probe 5")))]
    (:wat::core::Record/field-at v 1)))

;; ─── Probe 7: equality via holon_form ────────────────────────────────────────
(:wat::core::defn :user::probe-7 [] -> :wat::holon::Record
  (:wat::holon::Record::of
    :myapp::Voltage
    [5.0]
    (:wat::holon::Bind
      (:wat::holon::Atom (:wat::holon::to-holon "myapp::Voltage"))
      (:wat::core::Result/expect
        (:wat::holon::Bundle
          [(:wat::holon::Bind
             (:wat::holon::Atom (:wat::holon::to-holon "magnitude"))
             (:wat::holon::Atom (:wat::holon::to-holon 5.0)))])
        "Bundle failed in Probe 7"))))
