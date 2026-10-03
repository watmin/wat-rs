;; Scratch probe for BRIEF-STONE-2a: confirm the bracket parametric-annotation form
;; `(:wat::core::PersistentMap [K V])` / `(:wat::core::PersistentVector [T])` — including
;; NESTED forms and references to already-stdlib nominal types (:wat::rete::Element,
;; :wat::rete::Token, :wat::core::Record) — type-checks against the current Persistent
;; family schemes (landed 9c82f157). Not a re-check of wat/rete.wat's own edits (that file
;; is baked into the stdlib at build time; only a rebuild reflects on-disk edits — orchestrator's
;; job). This just proves the bracket mechanics + nominal references I used are legal.

;; network-shaped: (PersistentMap :- [i64 Record])
(wat.core/defn scratch.stone2a/network-get
  [m :- (wat.type/PersistentMap :- [wat.type/i64 wat.type/Record])
   k :- wat.type/i64]
  :- (wat.core/Option :- [wat.type/Record])
  (wat.core/get m k))

;; alpha-mem-shaped: (PersistentMap :- [i64 (PersistentVector :- [Element])])
(wat.core/defn scratch.stone2a/alpha-mem-get
  [m :- (wat.type/PersistentMap :- [wat.type/i64 (wat.type/PersistentVector :- [wat.rete/Element])])
   k :- wat.type/i64]
  :- (wat.core/Option :- [(wat.type/PersistentVector :- [wat.rete/Element])])
  (wat.core/get m k))

;; beta-mem-shaped: (PersistentMap :- [i64 (PersistentVector :- [Token])])
(wat.core/defn scratch.stone2a/beta-mem-assoc
  [m :- (wat.type/PersistentMap :- [wat.type/i64 (wat.type/PersistentVector :- [wat.rete/Token])])
   k :- wat.type/i64
   v :- (wat.type/PersistentVector :- [wat.rete/Token])]
  :- (wat.type/PersistentMap :- [wat.type/i64 (wat.type/PersistentVector :- [wat.rete/Token])])
  (wat.core/assoc m k v))

;; bindings-shaped: (PersistentMap :- [String Value])
(wat.core/defn scratch.stone2a/bindings-get
  [b :- (wat.type/PersistentMap :- [wat.type/String wat.type/Value])
   k :- wat.type/String]
  :- (wat.core/Option :- [wat.type/Value])
  (wat.core/get b k))

;; query-memory-shaped: (PersistentMap :- [String (PersistentVector :- [(PersistentMap :- [String Value])])])
(wat.core/defn scratch.stone2a/query-memory-get
  [qm :- (wat.type/PersistentMap :- [wat.type/String
           (wat.type/PersistentVector :- [(wat.type/PersistentMap :- [wat.type/String wat.type/Value])])])
   k  :- wat.type/String]
  :- (wat.core/Option :- [(wat.type/PersistentVector :- [(wat.type/PersistentMap :- [wat.type/String wat.type/Value])])])
  (wat.core/get qm k))

;; support-shaped: (PersistentMap :- [Record Support])
(wat.core/defn scratch.stone2a/support-get
  [s :- (wat.type/PersistentMap :- [wat.type/Record wat.rete/Support])
   f :- wat.type/Record]
  :- (wat.core/Option :- [wat.rete/Support])
  (wat.core/get s f))

(wat.kernel/println "probe-stone-2a-bracket-mechanics: loaded")
