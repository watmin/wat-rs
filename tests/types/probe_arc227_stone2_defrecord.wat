;; tests/types/probe_arc227_stone2_defrecord.wat
;; Co-located fixture for probe_arc227_stone2_defrecord.rs (arc 227 Stone 227.2 v3).
;; Loaded via startup_beside(file!()). Each :user::t<N> function is exercised by
;; its sibling Rust test. Negative-startup tests use separate .wat.bad fixtures.

;; ─── Type definitions ────────────────────────────────────────────────────────

(:wat::core::defrecord :test::Voltage [value <- :wat::core::f64])
(:wat::core::defrecord :test::Current [value <- :wat::core::f64])
(:wat::core::defrecord :test::Celsius [value <- :wat::core::f64])
(:wat::core::defrecord :test::Kelvin [value <- :wat::core::f64])
(:wat::core::defrecord :test::MyMap [value <- :wat::core::String])
(:wat::core::defrecord :test::Other [value <- :wat::core::String])
(:wat::core::defrecord :test::BasisPoint [value <- :wat::core::i64])
(:wat::core::defrecord :test::Count [value <- :wat::core::i64])
(:wat::core::defrecord :test::Label [text <- :wat::core::String])
(:wat::core::defrecord :appA::Voltage [value <- :wat::core::i64])
(:wat::core::defrecord :appB::Voltage [value <- :wat::core::i64])
(:wat::core::defrecord :awesome::lib::Sensor [value <- :wat::core::i64])
(:wat::core::defrecord :ns::Tag [])
(:wat::core::defrecord :ns::Done [])
(:wat::core::defrecord :ns::Pending [])
(:wat::core::defrecord :nsA::Tag [])
(:wat::core::defrecord :nsB::Tag [])
(:wat::core::defrecord :my::deep::ns::Reading [value <- :wat::core::f64])
(:wat::core::defrecord :ns::W [v <- :wat::core::i64])
(:wat::core::defrecord :ns::P [a <- :wat::core::i64  b <- :wat::core::String])
(:wat::core::defrecord :ns::T [a <- :wat::core::i64  b <- :wat::core::String  c <- :wat::core::bool])
(:wat::core::defrecord :multi::Tag [])
(:wat::core::defrecord :multi::W [v <- :wat::core::i64])
(:wat::core::defrecord :multi::P [a <- :wat::core::i64  b <- :wat::core::String])
(:wat::core::defrecord :multi::Q [a <- :wat::core::i64  b <- :wat::core::String])
(:wat::core::defrecord :multi::T [a <- :wat::core::i64  b <- :wat::core::String  c <- :wat::core::bool])
(:wat::core::defrecord :appA::Point [x <- :wat::core::i64  y <- :wat::core::i64])
(:wat::core::defrecord :appB::Point [x <- :wat::core::i64  y <- :wat::core::i64])

;; ─── t01: single FQDN positive ───────────────────────────────────────────────
(:wat::core::defn :user::t01 [] -> :wat::core::bool
  (:wat::core::let [instance (:test::Voltage 5.0)] (:test::is-Voltage? instance)))

;; ─── t02: single FQDN negative (different class) ─────────────────────────────
(:wat::core::defn :user::t02 [] -> :wat::core::bool (:test::is-Voltage? (:test::Current 1.0)))

;; ─── t03: cross-namespace appA positive ─────────────────────────────────────
(:wat::core::defn :user::t03 [] -> :wat::core::bool
  (:wat::core::let [a-instance (:appA::Voltage 42)] (:appA::is-Voltage? a-instance)))

;; ─── t04: cross-namespace discrimination ────────────────────────────────────
(:wat::core::defn :user::t04 [] -> :wat::core::bool
  (:wat::core::let [b-instance (:appB::Voltage 42)] (:appA::is-Voltage? b-instance)))

;; ─── t05: same-namespace celsius positive ────────────────────────────────────
(:wat::core::defn :user::t05 [] -> :wat::core::bool
  (:wat::core::let [c (:test::Celsius 100.0)] (:test::is-Celsius? c)))

;; ─── t06: same-namespace cross-discrimination ────────────────────────────────
(:wat::core::defn :user::t06 [] -> :wat::core::bool
  (:wat::core::let [c (:test::Celsius 100.0)] (:test::is-Kelvin? c)))

;; ─── t07: user type vs builtin positive ─────────────────────────────────────
(:wat::core::defn :user::t07 [] -> :wat::core::bool
  (:wat::core::let [instance (:test::MyMap "data")] (:test::is-MyMap? instance)))

;; ─── t08: user type vs other user type (cross-pred) ─────────────────────────
(:wat::core::defn :user::t08 [] -> :wat::core::bool
  (:wat::core::let [instance (:test::MyMap "data")] (:test::is-Other? instance)))

;; ─── t09: polymorphic is? positive ──────────────────────────────────────────
(:wat::core::defn :user::t09 [] -> :wat::core::bool
  (:wat::core::let [instance (:test::Voltage 5.0)] (:test::is-Voltage? instance)))

;; ─── t10: polymorphic is? cross-class negative ──────────────────────────────
(:wat::core::defn :user::t10 [] -> :wat::core::bool
  (:wat::core::let [instance (:test::Current 2.0)] (:test::is-Voltage? instance)))

;; ─── t11: multi-segment namespace positive ───────────────────────────────────
(:wat::core::defn :user::t11 [] -> :wat::core::bool
  (:wat::core::let [instance (:awesome::lib::Sensor 42)] (:awesome::lib::is-Sensor? instance)))

;; ─── t12: multi-segment polymorphic is? ─────────────────────────────────────
(:wat::core::defn :user::t12 [] -> :wat::core::bool
  (:wat::core::let [instance (:awesome::lib::Sensor 42)] (:awesome::lib::is-Sensor? instance)))

;; ─── t13: predicate name shape ───────────────────────────────────────────────
(:wat::core::defn :user::t13 [] -> :wat::core::bool
  (:wat::core::let [instance (:test::BasisPoint 25)] (:test::is-BasisPoint? instance)))

;; ─── t14: i64 payload ────────────────────────────────────────────────────────
(:wat::core::defn :user::t14 [] -> :wat::core::bool
  (:wat::core::let [instance (:test::Count 99)] (:test::is-Count? instance)))

;; ─── t15: cross-type discrimination kelvin positive ─────────────────────────
(:wat::core::defn :user::t15 [] -> :wat::core::bool
  (:wat::core::let [k (:test::Kelvin 373.15)] (:test::is-Kelvin? k)))

;; ─── t16: no user-namespace insertion ────────────────────────────────────────
(:wat::core::defn :user::t16 [] -> :wat::core::bool
  (:wat::core::let [c (:test::Celsius 273.15)] (:test::is-Celsius? c)))

;; ─── t17: appB cross-namespace predicate ─────────────────────────────────────
(:wat::core::defn :user::t17 [] -> :wat::core::bool
  (:wat::core::let [b-instance (:appB::Voltage 99)] (:appB::is-Voltage? b-instance)))

;; ─── t18: empty field-list zero-arg constructor ──────────────────────────────
(:wat::core::defn :user::t18 [] -> :wat::core::bool
  (:wat::core::let [instance (:ns::Tag)] (:ns::is-Tag? instance)))

;; ─── t19: tagged unit predicate true ─────────────────────────────────────────
(:wat::core::defn :user::t19 [] -> :wat::core::bool (:ns::is-Done? (:ns::Done)))

;; ─── t20: tagged unit predicate false for non-instance ───────────────────────
(:wat::core::defn :user::t20 [] -> :wat::core::bool (:ns::is-Done? (:ns::Pending)))

;; ─── t21: single-field String constructor ────────────────────────────────────
(:wat::core::defn :user::t21 [] -> :wat::core::bool
  (:wat::core::let [instance (:test::Label "hello")] (:test::is-Label? instance)))

;; ─── t22: cross-namespace tags distinct ─────────────────────────────────────
(:wat::core::defn :user::t22 [] -> :wat::core::bool
  (:wat::core::let [a-tag (:nsA::Tag)] (:nsA::is-Tag? a-tag)))

;; ─── t23: multi-segment namespace with field ─────────────────────────────────
(:wat::core::defn :user::t23 [] -> :wat::core::bool
  (:wat::core::let [instance (:my::deep::ns::Reading 3.14)] (:my::deep::ns::is-Reading? instance)))

;; ─── t25: zero-field instance uses empty Bundle (v3) ────────────────────────
;; part-a: predicate works
(:wat::core::defn :user::t25a [] -> :wat::core::bool (:ns::is-Tag? (:ns::Tag)))
;; part-b: separately-constructed empty Bundle has statement-length 0
(:wat::core::defn :user::t25b [] -> :wat::core::i64
  (:wat::holon::statement-length
    (:wat::core::Result/expect
      (:wat::holon::Bundle [])
      "empty bundle should not overflow")))

;; ─── t26: N=2 constructor ────────────────────────────────────────────────────
(:wat::core::defn :user::t26 [] -> :wat::core::bool
  (:wat::core::let [instance (:ns::P 5 "hi")] (:ns::is-P? instance)))

;; ─── t27: N=1 instance uses Bundle(Bind) ────────────────────────────────────
;; part-a: predicate works
(:wat::core::defn :user::t27a [] -> :wat::core::bool (:ns::is-W? (:ns::W 42)))
;; part-b: Bundle([one-item]) has statement-length 1
(:wat::core::defn :user::t27b [] -> :wat::core::i64
  (:wat::core::let
    [field-bind (:wat::holon::Bind
                  (:wat::holon::Atom (:wat::holon::to-holon "v"))
                  (:wat::holon::Atom (:wat::holon::to-holon 42)))]
    (:wat::holon::statement-length
      (:wat::core::Result/expect
        (:wat::holon::Bundle [field-bind])
        "single-item bundle should not overflow"))))

;; ─── t28: N=2 inner Bundle has 2 children ───────────────────────────────────
;; part-a: predicate works
(:wat::core::defn :user::t28a [] -> :wat::core::bool (:ns::is-P? (:ns::P 99 "test")))
;; part-b: Bundle([fa, fb]) has statement-length 2
(:wat::core::defn :user::t28b [] -> :wat::core::i64
  (:wat::core::let
    [fa (:wat::holon::Bind
          (:wat::holon::Atom (:wat::holon::to-holon "a"))
          (:wat::holon::Atom (:wat::holon::to-holon 5)))
     fb (:wat::holon::Bind
          (:wat::holon::Atom (:wat::holon::to-holon "b"))
          (:wat::holon::Atom (:wat::holon::to-holon "hi")))]
    (:wat::holon::statement-length
      (:wat::core::Result/expect
        (:wat::holon::Bundle [fa fb])
        "two-item bundle should not overflow"))))

;; ─── t29: N=3 constructor ────────────────────────────────────────────────────
(:wat::core::defn :user::t29 [] -> :wat::core::bool
  (:wat::core::let [instance (:ns::T 7 "world" true)] (:ns::is-T? instance)))

;; ─── t30: N=3 inner Bundle has 3 children ───────────────────────────────────
;; part-a: predicate works
(:wat::core::defn :user::t30a [] -> :wat::core::bool (:ns::is-T? (:ns::T 1 "x" false)))
;; part-b: Bundle([fa, fb, fc]) has statement-length 3
(:wat::core::defn :user::t30b [] -> :wat::core::i64
  (:wat::core::let
    [fa (:wat::holon::Bind
          (:wat::holon::Atom (:wat::holon::to-holon "a"))
          (:wat::holon::Atom (:wat::holon::to-holon 7)))
     fb (:wat::holon::Bind
          (:wat::holon::Atom (:wat::holon::to-holon "b"))
          (:wat::holon::Atom (:wat::holon::to-holon "world")))
     fc (:wat::holon::Bind
          (:wat::holon::Atom (:wat::holon::to-holon "c"))
          (:wat::holon::Atom (:wat::holon::to-holon true)))]
    (:wat::holon::statement-length
      (:wat::core::Result/expect
        (:wat::holon::Bundle [fa fb fc])
        "three-item bundle should not overflow"))))

;; ─── t31: predicate works for N=0,1,2,3 ─────────────────────────────────────
(:wat::core::defn :user::t31-n0 [] -> :wat::core::bool (:multi::is-Tag? (:multi::Tag)))
(:wat::core::defn :user::t31-n1 [] -> :wat::core::bool (:multi::is-W? (:multi::W 42)))
(:wat::core::defn :user::t31-n2 [] -> :wat::core::bool (:multi::is-P? (:multi::P 5 "hi")))
(:wat::core::defn :user::t31-n3 [] -> :wat::core::bool (:multi::is-T? (:multi::T 1 "x" false)))
(:wat::core::defn :user::t31-neg [] -> :wat::core::bool (:multi::is-P? (:multi::Q 1 "y")))

;; ─── t32: cross-namespace distinct classifiers N=2 ──────────────────────────
(:wat::core::defn :user::t32a [] -> :wat::core::bool (:appA::is-Point? (:appA::Point 1 2)))
(:wat::core::defn :user::t32neg [] -> :wat::core::bool (:appA::is-Point? (:appB::Point 1 2)))
