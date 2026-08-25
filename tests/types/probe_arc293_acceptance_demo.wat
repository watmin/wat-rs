;; tests/types/probe_arc293_acceptance_demo.wat — co-located fixture
;;
;; Arc 293.0 — THE ACCEPTANCE DEMO (the arc's final GREEN gate / R1 FORMA SOLA SUFFICIT fulfillment).
;; RED at HEAD: defsurface method members / dispatcher / extend-type adapter unbuilt.
;; This fixture is #[ignore]'d until arc-293.4 lands.

;; ── THE SURFACE — a set-of-accessor (fields AND methods, uniformly) ──
;; All members go inside the single member-vector: field triples mixed with method-sig Lists.
(:wat::core::defsurface :geo::Shape
  :nature :wat::core::Struct
  :features [color <- :wat::core::String                       ; FIELD-style accessor  → :T/color -> :String
   (area  [self <- :geo::Shape] -> :wat::core::f64)   ; METHOD accessor       → :T/area  [self] -> :f64
   (label [self <- :geo::Shape] -> :wat::core::String)]); METHOD accessor       → :T/label [self] -> :String

;; ── OWN TYPE #1 — Circle (core record). :geo::Circle/color is generated FREE by the field. ──
(:wat::core::defrecord :geo::Circle [color <- :wat::core::String  radius <- :wat::core::f64])
(:wat::core::defn :geo::Circle/area [self <- :geo::Circle] -> :wat::core::f64
  (:wat::core::f64::* 3.14159 (:wat::core::f64::* (:geo::Circle/radius self) (:geo::Circle/radius self))))
(:wat::core::defn :geo::Circle/label [self <- :geo::Circle] -> :wat::core::String
  (:wat::string::concat "circle(r=" (:wat::core::str (:geo::Circle/radius self)) ")"))
;;  ⇒ Circle exposes color+area+label ⇒ STRUCTURALLY satisfies :geo::Shape. No declaration.

;; ── OWN TYPE #2 — Square. Same surface, different fields. ──
(:wat::core::defrecord :geo::Square [color <- :wat::core::String  side <- :wat::core::f64])
(:wat::core::defn :geo::Square/area [self <- :geo::Square] -> :wat::core::f64
  (:wat::core::f64::* (:geo::Square/side self) (:geo::Square/side self)))
(:wat::core::defn :geo::Square/label [self <- :geo::Square] -> :wat::core::String
  (:wat::string::concat "square(s=" (:wat::core::str (:geo::Square/side self)) ")"))

;; ── THE MONKEYPATCH — teach a FOREIGN built-in (core Vector) to be a Shape (you don't own it) ──
;; NOTE: (:wat::core::Vector ...) constructs a Value::Vec whose type_name is "wat::core::Vector",
;; NOT ":wat::holon::Vector" (a different Value variant). Extend target must match the runtime type.
(:wat::core::extend-type :wat::core::Vector :geo::Shape
  (color [self] -> :wat::core::String "grey")
  (area  [self] -> :wat::core::f64 (:wat::core::i64::to-f64 (:wat::core::length self)))
  (label [self] -> :wat::core::String
    (:wat::string::concat "vector[" (:wat::core::str (:wat::core::length self)) "]")))

;; ── POLYMORPHIC CONSUMER — accepts ANY Shape; the dispatcher routes :T/<accessor> by runtime type ──
(:wat::core::defn :geo::describe [s <- :geo::Shape] -> :wat::core::String
  (:wat::string::concat
    (:geo::Shape/color s) " " (:geo::Shape/label s) " area="
    (:wat::core::str (:geo::Shape/area s))))

(:wat::core::defn :geo::demo [] -> :wat::core::String
  (:wat::string::concat
    (:geo::describe (:geo::Circle :color "red" :radius 2.0))
    "  |  "
    (:geo::describe (:geo::Square :color "blue" :side 3.0))
    "  |  "
    (:geo::describe (:wat::core::Vector :wat::core::i64 10 20 30))))
