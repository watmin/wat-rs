;; Scratch probe for BRIEF-STONE-2a: confirm the bracket parametric-annotation form
;; `(:wat::core::PersistentMap [K V])` / `(:wat::core::PersistentVector [T])` — including
;; NESTED forms and references to already-stdlib nominal types (:wat::rete::Element,
;; :wat::rete::Token, :wat::core::Record) — type-checks against the current Persistent
;; family schemes (landed 9c82f157). Not a re-check of wat/rete.wat's own edits (that file
;; is baked into the stdlib at build time; only a rebuild reflects on-disk edits — orchestrator's
;; job). This just proves the bracket mechanics + nominal references I used are legal.

;; network-shaped: (PersistentMap :- [i64 Record])
(:wat::core::defn :scratch::stone2a::network-get
  [m <- (:wat::core::PersistentMap [:wat::core::i64 :wat::core::Record])
   k <- :wat::core::i64]
  -> (:wat::core::Option :- [:wat::core::Record])
  (:wat::map::get m k))

;; alpha-mem-shaped: (PersistentMap :- [i64 (PersistentVector :- [Element])])
(:wat::core::defn :scratch::stone2a::alpha-mem-get
  [m <- (:wat::core::PersistentMap [:wat::core::i64 (:wat::core::PersistentVector [:wat::rete::Element])])
   k <- :wat::core::i64]
  -> (:wat::core::Option :- [(:wat::core::PersistentVector :- [:wat::rete::Element])])
  (:wat::map::get m k))

;; beta-mem-shaped: (PersistentMap :- [i64 (PersistentVector :- [Token])])
(:wat::core::defn :scratch::stone2a::beta-mem-assoc
  [m <- (:wat::core::PersistentMap [:wat::core::i64 (:wat::core::PersistentVector [:wat::rete::Token])])
   k <- :wat::core::i64
   v <- (:wat::core::PersistentVector [:wat::rete::Token])]
  -> (:wat::core::PersistentMap [:wat::core::i64 (:wat::core::PersistentVector [:wat::rete::Token])])
  (:wat::map::assoc m k v))

;; bindings-shaped: (PersistentMap :- [String Value])
(:wat::core::defn :scratch::stone2a::bindings-get
  [b <- (:wat::core::PersistentMap [:wat::core::String :wat::core::Value])
   k <- :wat::core::String]
  -> (:wat::core::Option :- [:wat::core::Value])
  (:wat::map::get b k))

;; query-memory-shaped: (PersistentMap :- [String (PersistentVector :- [(PersistentMap :- [String Value])])])
(:wat::core::defn :scratch::stone2a::query-memory-get
  [qm <- (:wat::core::PersistentMap [:wat::core::String
           (:wat::core::PersistentVector [(:wat::core::PersistentMap [:wat::core::String :wat::core::Value])])])
   k  <- :wat::core::String]
  -> (:wat::core::Option :- [(:wat::core::PersistentVector :- [(:wat::core::PersistentMap :- [:wat::core::String :wat::core::Value])])])
  (:wat::map::get qm k))

;; support-shaped: (PersistentMap :- [Record Support])
(:wat::core::defn :scratch::stone2a::support-get
  [s <- (:wat::core::PersistentMap [:wat::core::Record :wat::rete::Support])
   f <- :wat::core::Record]
  -> (:wat::core::Option :- [:wat::rete::Support])
  (:wat::map::get s f))

(:wat::core::println "probe-stone-2a-bracket-mechanics: loaded")
