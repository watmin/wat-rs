# Arc 265 — the namespace-scoped acronym registry (DESIGN)

> Restores the disparate case PascalCase⇄kebab can't carry — AWS's `WebACL ⇄ web-acl ⇄ WebACL`.
> The conversion is a lossy projection; the registry is a *memory* of the casing it drops. **Scoped
> by namespace** (`my::ns` owns its acronyms; no entry → default plain conversion). Threaded into
> defservice's op-name derivation. Supersedes this arc's `STUB.md` open questions (now decided).
> Four-Q + intueri both run (below). Grounded against HEAD `bf24ba17`.

## Decided (four-Q + intueri)

**Four-Q on the design (namespace-scoped registry, consulted with a default fallback):** Obvious
(a namespace owns its acronyms; none registered → plain), Simple (one map `ns→acronym-set` +
one lookup-with-default), Honest (acronyms ARE domain-specific; default fabricates none), Good-UX
(declare once per namespace; zero cross-namespace collision) — all YES.

**Names (intueri cast, weighed):**
- `:wat::core::string::declare-acronyms` — `(declare-acronyms :my::ns ["ACL" "HTTP" "URL" "ARN"])`.
  "declare" names the act (asserting a namespace's acronym vocabulary), verb-form like its siblings,
  dodges `def`'s weight (rejected `define-`), the query-reading `acronyms-for`, the plumbing
  `register-`, and the noun `acronyms`.
- `:wat::core::string::pascal->kebab-in` / `:wat::core::string::kebab->pascal-in` — the `-in <ns>`
  suffix (NOT a 2-arity overload of the pure shipped converters — that would blur the
  pure/registry-reading contract at call sites; distinct verbs for distinct contracts).

## The pieces (each placed by the OP-PLACEMENT macro-fence rubric)

1. **Registry** — `acronym_registry: HashMap<String, Vec<String>>` (namespace → canonical acronyms,
   e.g. `"ACL"`) on `SymbolTable` (beside `protocol_registrations` / `runtime_def_values`).
2. **`declare-acronyms`** — a form parsed → `(namespace, acronyms)` registered into the registry.
   **Must populate the registry BEFORE macro expansion** so a defservice expanding later sees it —
   the pre-registration discipline protocols use (`preregister_protocol_names`, freeze step 6.95).
   This ORDERING is the load-bearing risk; the probe gates it.
3. **`pascal->kebab-in`** — **Rust intrinsic, on `is_pure_total`** (defservice's macro calls it at
   EXPAND time → must be macro-reachable; reading the registry is a deterministic read of
   compile-time data, no mutation → fenceable). Tokenizes the PascalCase name using
   `registry[ns]` (a registered acronym is one segment; capital-boundary for the rest), downcases,
   joins with `-`. No entry for `ns` → the plain `pascal->kebab` behavior.
4. **`kebab->pascal-in`** — **Rust intrinsic** (NOT on `is_pure_total` — no macro needs it; it's an
   intrinsic because it READS the registry, Rust state — a wat helper can't reach it without a read
   intrinsic anyway, so the read IS the floor reason). Splits on `-`; each segment that matches a
   `registry[ns]` acronym (case-insensitive) → the canonical form (`ACL`); else capitalize. No
   entry → the plain `kebab->pascal` behavior.
5. **defservice thread** — `wat/service.wat` op-name derivation: `pascal->kebab op-str` →
   `pascal->kebab-in fqdn-str op-str` (the service's own fqdn is the namespace). So a defservice in
   `:my::aws` consults `my::aws`'s declared acronyms.

The plain `pascal->kebab` / `kebab->pascal` (shipped) STAY as the namespace-agnostic default.

## The algorithm note

Forward without the registry already gets acronym RUNS mostly right via a capital-run heuristic
(`WebACL`→`web-acl`); the registry makes it *correct* on the ambiguous cases (`ACLRule`→`acl-rule`,
not `acl-r-u-l-e` or `a-c-l-rule`) and — crucially — gives the REVERSE the information to restore
casing. Prior art: Rails `Inflector.acronym` / Go `initialisms` / Python `inflection`.

## Gate (RED at HEAD → GREEN) — `tests/probe_arc265_acronym_registry.rs`

- `(declare-acronyms :my::aws ["ACL"])` then `(pascal->kebab-in :my::aws "CreateWebACL")` →
  `"create-web-acl"`; `(kebab->pascal-in :my::aws "create-web-acl")` → `"CreateWebACL"`; round-trip
  holds.
- **Default:** `(kebab->pascal-in :other::ns "create-web-acl")` → `"CreateWebAcl"` (ACL not
  restored — no entry for `:other::ns`).
- **defservice (the expand-time-ordering gate):** `(declare-acronyms :my::aws ["ACL"])` BEFORE a
  `defservice :my::aws` with op `:CreateWebACL` → the generated `:my::aws/create-web-acl-request`
  constructor resolves + works (proves declare-acronyms populated the registry before the macro
  expanded, and defservice consulted it).
- lib (zero new) + nursery (zero new) + workspace compiles.

## STOP triggers
1. `declare-acronyms` can't populate the registry before defservice expands (the ordering) → STOP,
   surface — it's the whole point; mirror `preregister_protocol_names`.
2. `pascal->kebab-in` can't go on `is_pure_total` / the macro can't reach it → STOP.
3. Tempted to make the plain shipped `pascal->kebab`/`kebab->pascal` namespace-aware (changing their
   contract) → STOP; the `-in` variants are separate.

## Out of scope
- Acronym-run enforcement / discipline checks → not here.
- Revisiting `PASCAL-KEBAB-CONVERSION.md`'s "discipline the namespace" contract → update its bijection
   section to note the registry is the escape hatch (a doc touch, part of this stone).
