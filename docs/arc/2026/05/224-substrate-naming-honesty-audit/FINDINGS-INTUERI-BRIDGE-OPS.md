# Intueri Findings — Bridge Operations (arc 225 rename targets)

**Spell:** intueri (datamancy grimoire)
**Target:** four bridge functions in `src/runtime.rs`
**Functions:** `value_to_atom` (13838), `eval_atom_value` (13633), `eval_holon_from_watast` (14095), `eval_holon_to_watast` (14144)
**Agent:** claude-sonnet-4-6
**Date:** 2026-05-22 (late)
**Cast by:** orchestrator (claude-opus-4-7) per `feedback_spells_cast_via_subagent`
**Prior cast:** FINDINGS-INTUERI-RUNTIME.md (Stone 224.2) — surfaced L1-1 and named `atomize`/`materialize`; this cast evaluates those proposed renames under typed-entities doctrine
**Note on harness:** sonnet's harness blocked file creation; findings delivered as text and written to disk by orchestrator.

---

## Spell verdict

**Spark verdict: the two existing bridge ops (`from-watast`, `to-watast`) are honest and clearly named. The two being renamed (`Atom`/`atom-value`) carry confirmed Level 1 lies. The proposed rename `atomize` is itself a Level 1 lie of the same family as the original. `materialize` is honest as an operation-name but breaks the symmetric directional family pattern.**

The load-bearing question for this cast: **does `atomize` fall into the same lie-family as `:wat::holon::Atom`?** Answer: **yes.** Full audit below.

---

## Per-function audit

### 1. `value_to_atom` / `:wat::holon::Atom` (lines 13820-13982)

**What the name promises:** "produce an Atom from a value" — specifically, a `HolonAST::Atom` node.

**What the body actually does:** polymorphic dispatcher across 9 input arms. Output by input:

| Input | Output HolonAST shape | Is it `HolonAST::Atom`? |
|---|---|---|
| `i64 / f64 / bool / String` | typed primitive leaf | NO |
| `keyword` | `HolonAST::Keyword` | NO |
| `Unit` | `HolonAST::Nil` | NO |
| `Char` | `HolonAST::Char` | NO |
| `Uuid` | `HolonAST::Bind(Tag("uuid"), String(hex))` | NO — composite |
| `HolonAST` | `HolonAST::Atom(inner)` | YES — opaque-identity wrap |
| `WatAST` | `watast_to_holon(a)` — full structural tree | NO |
| `HashSet<T>` | `HolonAST::bundle(bare items)` | NO — Bundle |
| `Vec<T>` / `Tuple` | `HolonAST::bundle(positional-Bind pairs)` | NO — Bundle |
| `HashMap<K,V>` | `HolonAST::bundle(K-V Bind pairs)` | NO — Bundle |

**Verdict: L1 lie.** The name promises `HolonAST::Atom`; the body produces that for exactly 1 out of 10 input arms. Confirmed by prior cast (FINDINGS-INTUERI-RUNTIME.md L1-1).

### 2. `eval_atom_value` / `:wat::core::atom-value` (lines 13614-13816)

**What the name promises:** "extract the value from an Atom" — open a `HolonAST::Atom` and return what is inside.

**What the body actually does:** polymorphic decoder across HolonAST variants:

| Input HolonAST | Output Value | Consistent with "atom-value"? |
|---|---|---|
| `Symbol / Keyword / Nil / Char / String / I64 / F64 / Bool` | corresponding runtime primitive | ambiguous — none of these are Atoms |
| `Atom(inner)` | `Value::holon__HolonAST(inner)` | YES — this is the "atom-value" case |
| `Bundle(items)` | `Vec` / `HashMap` / `HashSet` (three-way dispatch) | NO — Bundle is not an Atom |

The function opens primitive leaves AND unwraps `HolonAST::Atom` AND decodes Bundles into three distinct collection types. The name `atom-value` captures only the `Atom(inner)` arm. For the Bundle arm, "atom-value" is a confirmed lie: a Bundle is not an Atom and `(atom-value bundle)` returns a Vec or HashMap.

**Verdict: L1 lie.** The name promises an Atom unwrap; the body is a full HolonAST-to-Value materializer.

**Additional finding:** the doc comment at lines 13619-13629 says "Composite (Bundle/...) -> error" but the body handles `HolonAST::Bundle` with three-way dispatch (arc 216 Stones 1/2/3 extended the body; the doc stayed at the pre-arc-216 contract). The doc is a lie about the function's own contract — L1-3 below.

### 3. `eval_holon_from_watast` / `:wat::holon::from-watast` (lines 14085-14125)

**What the name promises:** convert FROM a WatAST INTO a HolonAST. Direction explicit. Inverse of `to-watast`.

**What the body actually does:** accepts exactly `Value::wat__WatAST`, calls `watast_to_holon`, returns `Value::holon__HolonAST`. Single input type, single output type. Error on non-WatAST with helpful redirect.

**Verdict: honest.** No rename needed.

### 4. `eval_holon_to_watast` / `:wat::holon::to-watast` (lines 14127-14170)

**What the name promises:** convert a HolonAST TO a WatAST. Direction explicit. Inverse of `from-watast`.

**What the body actually does:** accepts `Value::holon__HolonAST`, calls `holon_to_watast`, returns `Value::wat__WatAST`. Single input type, single output type. Doc explicitly names the lossy parts (scope dropped, span not preserved).

**Verdict: honest.** No rename needed.

---

## L1 Findings

### L1-1 — `:wat::holon::Atom` / `value_to_atom` — constructor name imports a specific variant; body produces 9 other shapes

**File:lines:** `src/runtime.rs:13820` (dispatcher) + `13838` (body)

Confirmed from prior cast. Produces `HolonAST::Atom` for exactly 1 of 10 input arms.

### L1-2 — `:wat::core::atom-value` / `eval_atom_value` — name implies Atom unwrap; body decodes Bundles into Vec/HashMap/HashSet

**File:line:** `src/runtime.rs:13633`

A `HolonAST::Bundle` is not an Atom. `(atom-value bundle)` returning a HashMap is not what the name promises.

### L1-3 — `eval_atom_value` doc comment (lines 13619-13629) contradicts the body (NEW finding)

**File:lines:** `src/runtime.rs:13619-13629`

Doc says "Composite (Bundle/...) -> error." Body handles Bundle with three-way dispatch. Doc is a lie about the function's own contract. Arc 216 Stones 1/2/3 extended the body; the doc stayed at the pre-arc-216 contract.

**Honest direction:** update doc to describe the Bundle three-way dispatch.

---

## L2 Findings

### L2-1 — `holon_item_to_value` error arm hardcodes the wrong op name (latent)

**File:line:** `src/runtime.rs:13605-13610`

Confirmed from prior cast L1-3. Helper hardcodes `op: ":wat::core::atom-value"` but is called from multiple sites. Currently only called from `eval_atom_value`, so the lie is latent not active.

**Honest direction:** thread `op: &str` through the helper signature.

---

## Family-pattern audit

The four verbs as they stand today:

| Rust fn | wat verb | Direction | Honest? |
|---|---|---|---|
| `eval_algebra_atom` / `value_to_atom` | `:wat::holon::Atom` | runtime-Value to HolonAST (UP) | L1 lie |
| `eval_atom_value` | `:wat::core::atom-value` | HolonAST to runtime-Value (DOWN) | L1 lie |
| `eval_holon_from_watast` | `:wat::holon::from-watast` | WatAST to HolonAST (structural UP) | honest |
| `eval_holon_to_watast` | `:wat::holon::to-watast` | HolonAST to WatAST (structural DOWN) | honest |

The `from-watast` / `to-watast` pair is the honest template. The broken pair should mirror it: directional, layer-relative, no variant-name borrowing. The 2x2 the family wants to be:

```
from-watast   WatAST       -> HolonAST      structural UP from source tier
to-watast     HolonAST     -> WatAST        structural DOWN to source tier

???           runtime-Value -> HolonAST      UP from runtime tier
???           HolonAST      -> runtime-Value DOWN to runtime tier
```

---

## The atomize question — verdict

**`atomize` is a Level 1 lie of the same family as the original `Atom`.**

The argument:

1. `atomize` means "turn into an Atom." A reader encountering `(atomize 42)` will ask: "does this produce a `HolonAST::Atom`?" The answer is no — it produces `HolonAST::I64(42)`.

2. The typed-entities doctrine (2026-05-23 evening) states every typed user-surface value compiles to `(Bind (Atom ClassName) (Atom data))`. So "atomize" at the user-surface level promises a Bind-of-Atom composition. But at the runtime level, `value_to_atom` produces bare primitive leaves for primitives, Bundles for collections, `Bind(Tag, String)` for UUID — not consistently Bind-of-Atom compositions. The name still over-promises.

3. The lie family is: **a name that imports the `Atom` vocabulary (whether as a noun-constructor or a verb `atomize`) and produces shapes other than `HolonAST::Atom` for most inputs.** `atomize` is the verb form of exactly this lie.

4. The prior cast's "Honest direction" noted: "The verb is doing the boundary-crossing UP into the algebra; the name should signal that, not promise a specific HolonAST variant." `atomize` does not signal boundary-crossing; it signals Atom-production.

**`materialize` verdict:** honest for the DOWN direction. "Materialize the algebra back into runtime values" implies no specific HolonAST variant, correctly suggests going from abstract to concrete, reads well at call sites: `(materialize h)`. Passes all four questions. However, see family-consistency argument below.

---

## Recommended honest verb family

**Evaluated candidates against the four questions:**

**`to-holon` / `from-holon`**
- `to-holon`: Obvious YES / Simple YES / Honest YES (no variant promise; direction explicit) / Good UX YES (mirrors `to-watast` / `from-watast` exactly; 2x2 family complete)
- `from-holon`: Obvious YES / Simple YES / Honest YES / Good UX YES
- Both: YES / YES / YES / YES. Clean pass.

**`lift` / `lower`**
Both pass all four questions. Strong cross-language idiom. However, they do not make the layer explicit — "lift into what?" In a module already using `to-watast` / `from-watast`, a bare `lift` / `lower` is a weaker signal.

**`holonize` / `dewatize`**
`dewatize` fails the obvious test immediately. Not a word. Out.

**`encode` / `decode`**
Immediate namespace collision with `:wat::holon::encode` (VSA-bytes encoder). Out.

**`to-holon` / `materialize` (mixed asymmetric pair)**
Both pass individually. The asymmetry (one directional, one semantic) is a minor UX cost. Acceptable if the orchestrator prefers semantic expressiveness for the DOWN direction over strict symmetry.

---

## Final recommendation

**Recommended verb family (all four ops):**

| Op | Recommended wat verb | Rust fn name | Direction |
|---|---|---|---|
| runtime-Value to HolonAST | `:wat::holon::to-holon` | `eval_holon_to_holon` | UP |
| HolonAST to runtime-Value | `:wat::holon::from-holon` | `eval_holon_from_holon` | DOWN |
| WatAST to HolonAST | `:wat::holon::from-watast` | `eval_holon_from_watast` | structural UP (no change) |
| HolonAST to WatAST | `:wat::holon::to-watast` | `eval_holon_to_watast` | structural DOWN (no change) |

**Parity articulation:**

```
wat-tier:    (quote      wat-form)  -> :WatAST     ;  hold source-form unevaluated
              (to-watast  holon)    <- :HolonAST   ;  lower algebra -> source tier

holon-tier:  (to-holon   value)    -> :HolonAST   ;  lift runtime value -> algebra tier
              (from-holon holon)   <- :HolonAST   ;  materialize algebra -> runtime tier
```

The family is a consistent 2x2 across two layer-pair boundaries. Directional names; no variant-name borrowing; no variant promises. A reader at any call site knows: `to-holon` goes up, `from-holon` comes down, `to-watast` goes to source, `from-watast` comes from source.

**If the orchestrator prefers `materialize` for the DOWN direction:** `to-holon` / `materialize` is an honest asymmetric pair. Both pass the four questions. The family consistency cost is minor.

**Do NOT ship `atomize`.** It is a lie of the same family as the original `Atom` verb.

---

## Cross-references

- `wat-rs/docs/arc/2026/05/224-substrate-naming-honesty-audit/FINDINGS-INTUERI-RUNTIME.md` — Stone 224.2; surfaced L1-1; proposed `atomize`/`materialize`
- `wat-rs/src/runtime.rs` lines 13614-13982, 14085-14170 — the four bridge functions examined
- arc 225 DESIGN.md — the arc that will execute the rename
- [[typed-entities-doctrine]] memory entry — doctrine context
- [[atom-is-holder]] memory entry — earlier framing
- intueri SKILL.md at `~/work/holon/datamancy/intueri/SKILL.md`
