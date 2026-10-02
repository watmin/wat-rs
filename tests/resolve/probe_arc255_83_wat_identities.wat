(:wat::core::defn :user::form [src <- wat.type/String] -> wat.type/AST
  (:wat::core::let [tree (:wat::core::match (:wat::core::read-string src)
                         [:wat::core::ReadOutcome.Forms {:forms __forms} __forms]
                         [:wat::core::ReadOutcome.Malformed {:cause __cause}
                          (:wat::kernel::assertion-failed! :message "unreadable")])]
    (:wat::core::first (:wat::core::ast->children tree))))
(:wat::core::defn :user::bit [b <- wat.type/bool
                              yes <- wat.type/String
                              no <- wat.type/String] -> wat.type/String
  (:wat::core::if b yes no))
(:wat::core::defn :user::probe [] -> wat.type/String
  (:wat::string::concat
    (:user::bit (:wat::core::= (:wat::lint::eq-sym-name (:user::form "(:wat::core::= x 1)")) "x") "E1" "E0")
    (:user::bit (:wat::core::= (:wat::lint::eq-sym-name (:user::form "(wat.core/= x 1)")) "x") "e1" "e0")
    (:user::bit (:wat::core::= (:wat::lint::eq-sym-name (:user::form "(:wat.core/= x 1)")) "x") "e.1" "e.0")
    (:user::bit (:wat::fix::annotated-if? (:user::form "(:wat::core::if c -> :T t e)")) "A1" "A0")
    (:user::bit (:wat::fix::annotated-if? (:user::form "(wat.core/if c -> :T t e)")) "a1" "a0")
    (:user::bit (:wat::fix::annotated-if? (:user::form "(:wat.core/if c -> :T t e)")) "a.1" "a.0")
    (:user::bit (:wat::fix::annotated-if? (:user::form "(:wat::core::Option/expect c -> :T t)")) "X1" "X0")
    (:user::bit (:wat::fix::enum-purity-marker? (:user::form ":wat::enum::Pure")) "P1" "P0")
    (:user::bit (:wat::fix::enum-purity-marker? (:user::form "wat.enum/Pure")) "p1" "p0")
    (:user::bit (:wat::fix::enum-purity-marker? (:user::form ":wat.enum/Pure")) "p.1" "p.0")
    (:user::bit (:wat::fix::enum-purity-marker? (:user::form ":wat::enum::Impure")) "U1" "U0")
    (:user::bit (:wat::fix::enum-purity-marker? (:user::form "wat.enum/Impure")) "u1" "u0")
    (:user::bit (:wat::fix::first-of-drop? (:user::form "(:wat::core::first (:wat::core::drop x 1))")) "F1" "F0")
    (:user::bit (:wat::fix::first-of-drop? (:user::form "(wat.core/first (wat.core/drop x 1))")) "f1" "f0")
    (:user::bit (:wat::fix::first-of-drop? (:user::form "(:wat.core/first (:wat.core/drop x 1))")) "f.1" "f.0")))

(:wat::core::defsurface :probe::Scope :nature wat.type/Record
  :features [namespace <- wat.type/String])
(:wat::core::defrecord :user::KwMetric
  [~@:probe::Scope
   value <- wat.type/i64])
(wat.core/defrecord user/SymMetric
  [(wat.core/unquote-splicing :probe::Scope)
   value <- wat.type/i64])
(:wat::holon::defrecord :user::HKw
  [~@:probe::Scope
   value <- wat.type/i64])
(wat.holon/defrecord user/HSym
  [(wat.core/unquote-splicing :probe::Scope)
   value <- wat.type/i64])
(:wat::core::defn :user::kw-metric [] -> wat.type/String
  (:user::KwMetric/namespace (:user::KwMetric' "ns" 7)))
(:wat::core::defn :user::sym-metric [] -> wat.type/String
  (:user::SymMetric/namespace (:user::SymMetric' "ns" 7)))
(:wat::core::defn :user::hkw-metric [] -> wat.type/String
  (:user::HKw/namespace (:user::HKw' "ns" 7)))
(:wat::core::defn :user::hsym-metric [] -> wat.type/String
  (:user::HSym/namespace (:user::HSym' "ns" 7)))

(:wat::core::defn :user::ctor [] -> wat.type/i64 1)
(:wat::core::defn :user::kw-agg [] -> wat.type/i64
  (:wat::core::kwargs-lower :user::ctor :wat::core::agg-positional [] 0 :user))
(:wat::core::defn :user::sym-agg [] -> wat.type/i64
  (wat.core/kwargs-lower :user::ctor wat.core/agg-positional [] 0 :user))
(:wat::core::defn :user::dot-agg [] -> wat.type/i64
  (:wat::core::kwargs-lower :user::ctor :wat.core/agg-positional [] 0 :user))

(:wat::core::defn :user::consumes-kw [] -> wat.type/String
  (:user::consumes "(:wat::rete::exists (:user::T))"))
(:wat::core::defn :user::consumes-sym [] -> wat.type/String
  (:user::consumes "(wat.rete/exists (:user::T))"))
(:wat::core::defn :user::consumes [src <- wat.type/String] -> wat.type/String
  (:wat::core::let [form (:user::form src)
                    rule (:wat::rete::Rule
                           :name "r"
                           :lhs (wat.type/PersistentVector :- [wat.type/AST] form)
                           :rhs (wat.type/PersistentVector :- [wat.type/AST]))
                    got (:wat::rete::rule-consumes rule)]
    (:wat::core::Option/expect (:wat::core::get got 0) "no consume")))
