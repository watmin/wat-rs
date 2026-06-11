# NOTE — the redundant `-> :T` return-annotation class (match, expect, and the deeper root)

**Surfaced 2026-06-11** (builder, mid arc-251/4.2b planning): *"what else is using `-> :T` where it
shouldn't? … that feels like a thing we need to kill off entirely, right? we hard cut it from
`(do …)` early on, then cond and if in the last few days."* Builder agreed match+infer; **attack
soon** (this note). The deeper seeds (empty-container, recv/IO) are the **next priority** — see the
companion attack note.

## The class

A **mandatory `-> :T` return annotation on an expression form whose result type is resolvable from
context** (sub-expression types, recipient type, or branch unification) is pure redundancy — the
checker can synthesize it. Already pulled: `do` (arc 145), `if`/`cond` (arc 258.1/258.2). Remaining
stems below.

## match — CONFIRMED redundant (grounded: `infer_match`, `check.rs:5759`)

`infer_match` **requires** `-> :T` (refuses the no-annotation form with a migration hint, 5771-5784),
parses `declared_ty` from `args[2]`, and checks **each arm body against `:T` independently** — it does
NOT unify the arms. But it trivially could: the arms determine T by unification, exactly the if/cond
pattern. **Fix (258.5):** infer T by unifying arm bodies (like cond); drop the mandatory annotation;
divergent arms surface a branch-mismatch (the 258.1 `if` model), not a per-arm "declared type" miss.

## Option/expect (46 uses) + Result/expect (2) — redundant

`(Option/expect -> :T <opt> "msg")`: T = the arg's `Option<T>` element type (Result: the `Ok` type) —
inferable from the argument. **Fix (258.6):** infer T from the arg; keep an *optional* ascription only
for the rare bare-`None`/ambiguous seed (resolved by recipient-propagation — the do-model). **At build:
read the `Option/expect` infer arm in full** (this note grounded the *layout* requires `-> :T`; the
infer-arm body itself wasn't read — confirm it reads the annotation rather than already inferring).

## Four questions (both directions, protocol)

KILL (infer): Obvious YES (type comes from arms/arg) · Simple YES (require+check → infer-by-unify, the
proven do/if/cond shape) · Honest YES (keeps optional seed where genuinely ambiguous; keeps true
contracts) · UX YES (the no-redundant-token form is what an LLM reaches for). KEEP (mandatory): fails
Obvious + Simple. → **KILL.**

## KEEP — true contracts, NOT this class

`defn` / `fn` return types are **signature contracts** (the function's declared interface), not
synthesizable redundancy. They stay.

## ⚠️ THE DEEPER ROOT (revealed by the builder's next-priority pick)

I first classified **empty-container literals** and **recv/send/readln/IO** as "load-bearing seeds —
keep." **That was wrong** — the same rationalization error as the `ast-span` `:file` "can't know"
(accepting the *current checker's* limitation as inherent). They are NOT load-bearing:

- **empty container** — `(Vector)` should be `Vector<?fresh>`; HM unification resolves the element from
  *use* (wat already has Robinson unification). The element annotation is a crutch for the checker
  demanding a concrete type *at the literal* instead of fresh-var-and-unify.
- **recv/send/IO** — the type lives in the channel (typed peers, arc 214). `(recv chan)` should infer T
  from `chan : Channel<T>`. The `-> :T` is a crutch from before channels carried their type.

So the whole class is **one root: the checker requiring an inline `-> :T` seed where HM unification +
bidirectional expected-type propagation should flow the type in from context.** Pull that root and the
seeds die alongside match/expect. The seed-kill (bidirectional inference upgrade) is the active attack;
match/expect (synthesis-by-unification) is the soon-after.

## COMPLETE MAP (authoritative — checker-side `->`-detecting sites, 2026-06-11)

The corpus grep MISSED several (it sees *use*, not *what accepts `-> :T`*). The authoritative set =
every `s.as_str() == "->"` site in `check.rs` (the same corpus-blind-spot as the `to-i64` miss; the
checker grep caught 7+ more). Labeled by enclosing fn:

| Form (fn) | Class | Verdict |
|---|---|---|
| `defn` / `fn` | **contract** | KEEP (signature) |
| `if` (`infer_if`, 7127) | synthesis | killing (258.1/258.4) |
| `match` (`infer_match`, 5759) | synthesis (arms unify to T) | **KILL** |
| `option_expect` (8580) / `result_expect` (8689) | synthesis (arg elem / Ok type) | **KILL** |
| **`apply`** (`infer_apply`, 8891) ← *corpus-missed* | synthesis (f's return type) | **KILL** |
| `kernel_readln` (`infer_kernel_readln`, 8808) ← *corpus-missed* | IO seed (reader's type) | **KILL** (infer from reader) |
| legacy `recv` / `send` | IO seed | **KILL** (mirror the prime verbs — see below) |
| **`program::Env` ×6** (`get`/`expect_get`/`get_default`/`dig`/`expect_dig`/`dig_default`, 8988-9649) ← *corpus-missed* | **🔥 HERESY — dynamic-typing escape hatch** | **ANNIHILATE THE DESIGN** (not the annotation) — see verdict below |

**⭐ The kill pattern is ALREADY PROVEN ON DISK:** the arc-214 γ-1 peer prime verbs
(`infer_recv_prime` 10836 / `send_prime` 10759 / `select_prime` 11021) made `-> :T` an **optional
ascription** (line 10847: *"optional `-> :T` ascription"*). The annihilation MIRRORS γ-1 — it does not
invent a mechanism. Synthesis forms infer from a sub-expression; the optional ascription survives only
as a seed for genuinely-ambiguous positions.

## 🔥 `program::Env` — the heresy verdict (2026-06-11, builder: *"i see blatant heresy"*)

**Evidence:**
- `types.rs:608` — `typealias :wat::program::Env = :HashMap<:wat::core::keyword, :wat::holon::HolonAST>`.
  A stringly-keyed map of **type-erased** `HolonAST` values.
- `runtime.rs:7439` — `(Env/get-default env key dflt -> :T) → :T (default on miss/wrong-type)`. The
  accessor does a **runtime type-cast with fallback** — `-> :T` is the *cast target*, wrong-type → default.

**Verdict:** this is `dict.get(key) as T` — **dynamic typing smuggled into a statically-typed ADT
language.** The `-> :T` is NOT a redundant-synthesis annotation (you *cannot* infer T — the value is
erased to `HolonAST`) and NOT a recipient seed. It is load-bearing *because the design is heretical*:
you assert the type at runtime and hope. Deepest violation of wat's "typed from birth" identity. Same
class as `feedback_contract_not_encoding` — `HolonAST` is the *wire encoding*, abused here as the
*in-language representation* you cast out of.

**⚠️ CORRECTION (2026-06-11, same session) — I over-prosecuted; the builder narrowed it.**
My first verdict indicted the *whole dynamic store* ("KILL the HashMap, replace with a typed record").
**Wrong.** Builder: *"program-env is meant to be like process ENVs but not limited to string to string —
i wanted keyword to data."* The **dynamic keyword→data env is the INTENT** (a process-env analogue,
richer than `String→String`) — legitimate, not heresy. The actual abuse is **narrower and exactly what
the builder named**: the **value type `HolonAST`** imposes an unwanted **VSA hologram** where the
contract only needs *representable data*.

Grounded: `lower.rs:154` `lower(WatAST) -> HolonAST` (HolonAST is the *lowered* hologram form; WatAST is
upstream); `types.rs:597` arc-214 picked HolonAST as "the representable type" — but that **predates
251/257 making `WatAST` EDN-native**. Now `WatAST` *is* arbitrary data (EDN, `watast_to_edn`) **without**
the hologram. Textbook `feedback_contract_not_encoding`.

**The actual fix (narrow):** `:wat::program::Env = HashMap<keyword, WatAST>` — keyword→data (the intent),
hologram-free data type. Four-Q ratified (Obvious/Simple/Honest/UX YES; keep-HolonAST fails Obvious +
Honest). Verify: WatAST's wire-representability path at the `spawn-program'` boundary.

**The `-> :T` RECLASSIFIES:** with `keyword→WatAST`, `(Env/get key -> :T)` is a legitimate **parse-target**
("interpret this data as T," like parsing an env var) — NOT a dynamic cast. So it likely comes **OFF** the
redundant-`-> :T` kill list (unless EDN carries enough type info to infer T — open). Blast radius: zero
corpus callers, no typed-param programs → cheap. Still touches the unfinished **task #211** thread.

### ✅ FINAL RESOLUTION (2026-06-11) — typed extensible record; `-> :T` annihilates after all

I then proposed `Env/get -> Option<WatAST>` + `match` to kill the `-> :T`. **Builder caught that as a
second cheat:** *"are we cheating strong types? … encouraging runtime errors? … should it be a record
with a minimum accessor set (larger on user def)?"* — and yes: `keyword→WatAST` is an `any`-typed map;
`match` makes the dynamism *honest* but not *static*, and `parse-i64(ast-name …)` manufactures runtime
error paths. **Four-Q disqualifies it on Honest** (relocates typing to runtime).

**THE ANSWER (builder's, ratified): `program::Env` is a typed extensible `recordtype`.**
- Base `recordtype` (parent `:wat::Record`, `Record.wat:32`) with a **minimum field/accessor set**; a
  program **extends** it with its own TYPED fields. Access via generated, statically-typed accessors
  (`:myprog::Env/port` : i64) — no cast, no `-> :T`, no runtime parse, no silent fallback.
- "keyword→data, like process env but richer" = a **typed extensible record** (fields = keys, each a
  declared type; extension = per-program flexibility), NOT a dynamic any-map. Strongly typed *and* richer
  than `String→String`.

**Annihilation is nearly FREE — it's dead scaffolding.** Grounded 2026-06-11: `program::Env` has **zero
value-construction sites, zero corpus refs, zero accessor callers** — arc-214-slice-4 declared it
(typealias + 6 accessors) but never wired/populated it. So: DELETE the `HashMap<keyword, HolonAST>`
typealias + the 6 `-> :T` cast-accessors (+ Rust eval fns + check arms); REPLACE with the base
`recordtype`; WIRE into `spawn-program'` arg[1] = **closes task #211** (which never felt clean *because*
it was waiting on this typed env). The "minimum base set" is a fresh design choice (nothing to preserve;
likely minimal/near-empty, there to be extended).

This is a real arc (typed program env), cheap (no migration). Two cheats logged above stay visible
(WatAST-value, then WatAST+match) — both were dynamic typing in ADT costume; the record is the strong-typed
truth. Pairs `feedback_contract_not_encoding` + `feedback_absence_claim_needs_all_forms`.

### Reserved system-field prefix: **`wat.`** (LOCKED 2026-06-11)

System-injected fields (the runtime reaching into a program's env) need a reserved prefix so they can never
collide with user-declared fields. EDN constraint: a symbol has one `/` (spent on the record namespace), so
the marker is a **dot-prefix inside the name part** — `(:Env/wat.worker-id env)` — legal because dots in a
name are **inert** in wat (`ns_to_wat_path` dot-processes ONLY the namespace, name passes verbatim; grounded
`edn_shim.rs`). This is wat-clean where it is JVM-broken in Clojure: `(foo.bar)` throws `ClassNotFoundException`
there because a dotted head = Java-class lookup; wat has no JVM, so the constraint doesn't exist.

**Naming journey (intueri, two casts + a requirement-correction):**
- Builder's lean `wat.` → intueri cast #1 flagged **Level 2 mumble** (collides with the language root
  `:wat::core::`), proposed `rt.`. Weighed: `rt.` is an abbreviation at the *widest* scope (intueri's own rule
  condemns it). Cast #2 (with "the kernel is the literal injector") → **`kernel.`** (names the actual injector;
  the `:wat::kernel::` echo = semantic continuity).
- **Builder broke it open:** `kernel.` is TOO NARROW — it would **lie** the moment a *non-kernel* subsystem
  injects (scheduler tick, trace ctx, deadline). The real requirement is **subsystem-agnostic system-ownership**
  ("whichever part of wat reached in"), not "name the injector." Both intueri passes judged the wrong spec.
- **Resolution `wat.`:** under the corrected requirement, intueri's mumble objection DISSOLVES — these fields
  *are* wat's (the system planting its own fields), so `wat.` is **earned, not borrowed**. The two readings
  (language-root / system-owned) **collapse into one truth: wat owns it.** Position disambiguates (namespace
  before `/`, reserved-field after). It is the most honest possible name for "the system owns this" — literally
  the system's name. LOCKED by deduction, not by the ward (the ward judged the wrong spec; the duet found the
  right one). Lesson: a cast is only as right as the *requirement* it's given — correct the spec before trusting
  the verdict.

**First system field LOCKED: `wat.started-at` (`:wat::time::Instant`).** Runtime stamps it at spawn (`time/now`);
a program reads `(:Env/wat.started-at env)` → its start instant (uptime = `(time/now) - that`). intueri named it:
builder's `boot-time` was Level 2 (machine-boot overclaim + "-time" floats point/span); `started-at` keeps the
promise — `started` = the *program* coming up, `-at` = the EDN point-in-time idiom (`:created-at`/`:updated-at`).
It VINDICATES `wat.` over `kernel.` (a start timestamp is system-y, not kernel-y). So the **A2 base is NOT empty**:
`(recordtype :wat::program::Env :wat::Record [wat.started-at <- :wat::time::Instant])`.

## Sequencing (attack-and-reassess; map now complete enough — builder: *"the annihilation is obvious"*)

1. **Synthesis cluster** (one mechanism — infer from sub-expr, mirror γ-1): `match` (FIRST, agreed) →
   `option_expect`/`result_expect` → `apply`.
2. **IO cluster**: `kernel_readln` + legacy `recv`/`send` → mirror the prime verbs.
3. **`program::Env` ×6** — ON DECK. Ground Env-typing; kill if redundant, else affirm as genuine keepers.
4. **Corpus `-> :T` strip** (258.3) now covers if/cond/**match/expect/apply/readln/recv/send** — performed
   by the 4.2b comment-faithful codemod engine. Hard-cut the mandatory forms last.

Cross-ref: `feedback_absence_claim_needs_all_forms` (the "inference gap as law of nature" rationalization
— twice this session: ast-span `:file`, then these seeds; corrected both by grounding — AND the corpus-vs-
checker blind spot: a corpus grep sees *use*, the checker grep sees *what accepts it*; trust the latter
for a completeness claim).
