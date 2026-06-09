# Arc 257 — EDN-native collections: first-class `Map` + `Set`, `StructPattern` eliminated

**Status: DESIGN (lair studied 2026-06-09; not yet built).** Branch
`arc-170-gap-j-v5-deadlock-state`.

> ## ⛔ RESUME HERE (post-compaction / new session)
> Run `recolligere` from the signed channel, read this whole doc, then continue.
> The DESIGN is complete; the BUILD has not started. This arc was discovered while
> fixing the arc-213 spawn-process deadlock (see that arc's DESIGN-EXECVE doc) and
> is now a **prerequisite** for the clean arc-213 program-over-the-wire serializer.
> Decisions are locked (four-questions + builder). What remains is the build, sliced below.

---

## 1. How we got here (the diagnosis chain)

1. Arc 213: `spawn-process` deadlocks (clone3 without exec). The fix ships the program
   over the wire as **EDN**. The committed design said "serializer already built — use
   `watast_to_holon` + `wat_edn::write`."
2. A disconfirming probe of that serializer (in runtime.rs tests) **failed**: the frame
   was a wall of `#wat-edn.holon/{Bundle,Keyword,Symbol}` tags — the **VSA hologram
   encoder**, not EDN transport. This is the contract-vs-encoding abuse class
   (`feedback_contract_not_encoding`): *holon is one representation of EDN, never the wire
   envelope.* It also fed raw `::`-keywords into `Keyword::new`, bypassing the established
   `::`↔`.` convention (wat-edn `translate_and_validate_ns`), producing keywords wat-edn's
   own lexer rejects.
3. Correcting to **plain EDN** (a wat program *is* EDN) exposed the real gap: wat has
   surface forms that are **not valid EDN**. The holon-tagging had hidden this (it tagged
   everything, so non-EDN-ness never surfaced).
4. The non-EDN forms:
   - **struct-destructure `{x y z}`** — odd-arity ordered bare symbols. Not an EDN map
     (maps are even-arity key/value). A wat invention; not even standard Clojure.
   - **map literal `{:k v}`** and **set literal `#{x y z}`** — these *parse* but are
     **eagerly desugared at parse time** into constructor-call Lists
     (`(:wat::core::HashMap :Infer :Infer k v)` / `(:wat::core::HashSet :Infer x y)`).
     So wat's AST has **no map/set node** — a wat map serializes as a *function call*,
     not an EDN map. wat's AST has native `List` + `Vector` but is **missing `Map` + `Set`**.

The builder's framing: *"I thought this was already done… it not being done is an issue
and it's blocking us."* The fix is to make **wat's AST node set equal EDN's collection
set**: `List`, `Vector`, **`Map`, `Set`** — all first-class.

---

## 2. The decision (locked)

Introduce first-class **`WatAST::Map`** and **`WatAST::Set`** nodes. `{…}`/`#{…}` parse to
them (no eager desugar). **Eliminate `WatAST::StructPattern`**; destructure becomes
"a `Map` in binder/pattern position." The arc-213 serializer then maps `Map`→EDN-map,
`Set`→EDN-set, natively — no tags, no special cases.

**Scope boundary (locked):** this arc does the **map/set/destructure** half of EDN-
conformance. The **`::`-keyword → `:ns/name`** rip-out is the *separate, later*
clojure-ination arc (cf. arc 219 `wat-edn-strict-edn-keywords`). Until then the arc-213
serializer uses the existing `::`↔`.` translation for keywords.

### Surface consequences (all four-questions-clean)
- `{:keys [x y z]}` → keys-destructure (in binder position) — the Clojure idiom replacing
  the old `{x y z}`. In value position it's an ordinary map literal.
- `{x :field y :field2}` → hash-destructure (binder) / map literal (value). Already
  even-arity EDN; unchanged semantics, now a `Map` node.
- `{x y z}` (the old non-EDN struct-destructure) → **parse error** ("map body must
  alternate key/value") — correct: it should be rejected; the error guides migration to
  `{:keys [x y z]}`.
- `{:k v}` value position → `Map` node evaluating to `Value::wat__std__HashMap` (same value
  as today; only the AST representation changed).

---

## 3. Contract decisions (pinned)

1. **Node shapes:** `WatAST::Map(Vec<(WatAST, WatAST)>, Span)` (key/value pairs — odd arity
   unrepresentable by construction); `WatAST::Set(Vec<WatAST>, Span)`. `children()` flattens
   Map pairs to `[k,v,k,v,…]` for the generic tree-walk; `Hash`/`span`/`variant_name`
   follow `Vector`'s pattern.
2. **Literal vs constructor coexist.** `Map`/`Set` nodes are the *literal* forms; the typed
   verbs `:wat::core::HashMap`/`:wat::core::HashSet` (explicit `(… :K :V k v)`) **stay** as
   callable constructors. The literal `infer`/`eval` arms reuse the constructor logic
   (`infer_hashmap_constructor`/`eval_hashmap_ctor`) but **skip the leading type-keyword
   sentinels** (a literal carries no explicit `:K :V`; inference always starts fresh).
   Both produce the same `Value::wat__std__HashMap/HashSet`. **Runtime Value types are
   unchanged.**
3. **Destructure = `Map` in binder/pattern position.** Replace every
   `matches!(b, StructPattern(..))` with one helper `is_map_destructure_binder(b)` reading:
   `:keys [..]` → keys-destructure; symbol→keyword pairs → hash-destructure. The 14
   load-bearing sites swap the predicate; the ~75 generic-recursion/diagnostic arms vanish
   with the node.
4. **One `is_metadata_map(node)` helper** (DRY) accepting `WatAST::Map` **and** the legacy
   `List`-with-`:wat::core::HashMap`-head (explicit constructor metadata) — called at all 8
   metadata-sniff sites (parser.rs, check.rs, runtime.rs, types.rs, closure_extract.rs,
   function/metadata.rs). Eliminates 8 ad-hoc head-keyword checks.
5. **`watast_to_holon`** gains `Map`/`Set` arms producing the classifier shape
   `Bind(Atom(String("Map")), Bundle([Bind(k,v), …]))` / `Bind(Atom("Set"), Bundle([…]))`,
   matching `from_holon_item`'s existing `"Map"`/`"Set"` classifiers (so the holon round-trip
   stays symmetric).
6. **Closure round-trip stays constructor-form.** `closure_extract::encode_value_with_path`
   re-encodes a runtime HashMap/HashSet *value* (K/V types known) back to AST; it keeps
   emitting the explicit constructor-verb `List` (types preserved, lowest re-eval risk).
   This is internal capture, not user source — not a literal.
7. **Macro purity** (`macros/eval.rs`): add `WatAST::Map`/`Set` arms to the pure-form check
   (literal collections are pure), since they no longer reach the `:wat::core::HashMap`
   head-keyword table.
8. **arc-213 serializer:** the WatAST↔EDN bridge maps `Map`↔`OwnedValue::Map`,
   `Set`↔`OwnedValue::Set` directly (wat-edn already has these). Keywords still via `::`↔`.`.

---

## 4. Migration map (from two lair studies)

**StructPattern elimination** — 14 load-bearing sites (rewrite predicate → `is_map_
destructure_binder`), ~75 vanishing (generic `children()` arms, diagnostic labels, error
guards):
- Runtime: `parse_let_binding` (5772), `try_match_pattern` (11456), `try_match_pattern_ast`
  (21807).
- Check: `process_let_binding` (11325), `infer_match` (5856), `detect_match_shape` (6222),
  `check_let_for_scope_deadlock_inferred` (9929).
- Closure: `walk_let_form` (723), `collect_pattern_bindings` (1045), `rewrite_let` (2139),
  `rewrite_with_scope` (2096), `walk_free_symbols` (668). **Tricky:** rewrite passes must NOT
  substitute binder symbols inside the map pattern (preserve `:keys` vector verbatim).

**Map/Set node introduction** — ~10 sites move (literal desugar + dispatch), ~8 metadata-
sniff sites get the shared helper, ~50 value-keyed sites stay untouched:
- Parser: `parse_map_literal_body`/`parse_hashset_literal_body` → emit `Map`/`Set`; delete
  the `BraceKind` destructure dispatch + the two destructure-body parsers.
- Check: new `Map`/`Set` arms in `infer` (reuse `infer_hashmap/hashset_constructor` minus
  sentinels); the `infer_list` head arms (4180/4207) stay for the verb.
- Runtime: new `Map`/`Set` eval arms (reuse `eval_hashmap/hashset_ctor`); dispatch entries
  (3805/3806) stay for the verb. `watast_to_holon` Map/Set arms. AST node machinery
  (ast.rs: enum, span, children, variant_name, Hash, constructors).
- **Canonical hash** (`hash.rs`) changes for any form containing `{…}`/`#{…}` — expected for
  a breaking arc; note in migration. No persisted hashes cross this branch.

---

## 5. Slicing plan (examinare; each slice compiles the FULL workspace before commit)

- **257.0 — probe.** Disconfirming probe (RED at HEAD): `{:keys [x y z]}` in a `let` binder
  destructures + freezes + runs, AND a `{:k v}` literal round-trips through the (corrected)
  WatAST↔EDN bridge as a real EDN map. Proves the gaps before building.
- **257.1 — AST nodes.** Add `WatAST::Map`/`Set` + machinery (ast.rs, hash.rs). No parser
  change yet; nodes unreachable. Compiles green (exhaustive matches get `todo!`-free arms
  that mirror Vector, or a deliberate `unreachable!` until 257.2 wires the parser).
- **257.2 — parser.** `{…}`→`Map`, `#{…}`→`Set`; delete eager desugar + destructure-body
  parsers + `BraceKind`. `{x y z}`→parse error. Fixes the value-position infer/eval arms so
  literals type+eval (reuse ctor logic). Full non-destructure map/set tests green.
- **257.3 — destructure.** `is_map_destructure_binder` helper; rewrite the 14 load-bearing
  StructPattern sites; delete the node + ~75 vanishing arms. Closure hygiene care.
- **257.4 — metadata-sniff + holon + macro.** `is_metadata_map` helper at 8 sites;
  `watast_to_holon` Map/Set arms; macro purity arms.
- **257.5 — migration + verify.** Migrate `tests/types/struct_destructure.rs`
  (`{outcome grace-residue}`→`{:keys [outcome grace-residue]}`); full `cargo test --workspace`
  clean unaided; the arc-213 bridge probe green (program with map/set/destructure round-trips
  as plain EDN).

---

## 6. Verification

Structural + deterministic (no race): the corrected arc-213 round-trip probe (a program with
a map literal, a set literal, and a `{:keys […]}` destructure freezes identically after a
plain-EDN wire round-trip), the migrated destructure tests, and the full workspace. Plus:
`{x y z}` now produces a clear parse error (negative test).

## 7. Out of scope (named cuts, not deferrals-in-disguise)
- `::`-keyword → `:ns/name` rip-out — the separate clojure-ination arc (219 lineage).
- Clojure `:as` / `:or` / nested destructure extensions — future; this arc reaches feature
  parity with today's struct/hash destructure, no more.
- First-class handling beyond Map/Set (e.g. tagged-literal AST nodes) — not needed for EDN
  collection parity.
