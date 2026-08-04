# BRIEF — #74: `<Op>Response` is LAW. Enforce it once, then delete the machinery that guessed.

Anchor your cwd at `/home/watmin/work/holon/wat-rs/` and use `git -C /home/watmin/work/holon/wat-rs`
for anything git. Verify with `pwd` first.

## The work, in one paragraph

A service's op declares its response type. Nothing required that type to be named after the op, so
`wat/service.wat` — which must build a `<Response>::RequestTooLarge` value inside generated code —
could only *guess* the name by concatenation. One scratch probe named it otherwise, the loader gate
caught it, and the cure shipped as a Rust-emitted runtime constant plus an EDN-decode workaround at
two sites. The builder has since ruled the convention into law: **an op's response type IS
`<Op>Response`.** So the guess becomes correct by construction, and everything built to avoid the
guess becomes dead. You will add ONE check that makes the law real, migrate the ten declarations
that violate it, and delete the emitter, the constant, and both decode branches.

## Read in order

1. **`docs/arc/2026/06/278-rules-engine/DESIGN-STONE-the-surface-speaks-at-expand-time.md`** — read
   the SUPERSEDED banner at the top and stop at the horizontal rule. That banner is the whole spec:
   the ruling, the corrected check site, the census, the proven mechanism, and every STOP. Below the
   rule is a pre-ruling design kept only for its grounding; do not build from it.
2. **`src/types.rs:2690-2925`** — `synthesize_surface_protocol`. This is where the check goes. Note
   three things you will reuse rather than invent: `enforce_rtl_lock` (`:2758`) is the serviceable
   gate; the `:max-request-bytes` lock (`:2776`) is the shape of a located refusal in this loop;
   and `kebab_to_pascal_with_acronyms(name, ns_acronyms)` at `:2916` is the pascal conversion — the
   same one `wat/service.wat` reaches through `kebab->pascal-in`, so there is one implementation and
   you must call it, never re-derive it.
3. **`src/types.rs:3088-3156`** — `build_op_response_type_constants`, the emitter you delete. Its
   call site is `:3234`; the `rest.extend(...)` that ships its output is `:3293`; the `let mut`
   that holds it is `:3206`. All four go.
4. **`wat/service.wat:1078-1268`** — `serve-op-arms`. `resp-type-const-kw` (`:1093`),
   `resp-dotted-sym`/`rtl-edn-sym`/`rm-edn-sym` (`:1096-1098`), and the two decode blocks inside
   `shape-guarded` (`:1223-1242`) and `guarded-arm` (`:1248-1265`).
5. **`wat/service.wat:1485-1610`** — `op-methods`. The same shape once more: `resp-type-const-kw`
   (`:1517`), the symbol binders (`:1520-1521`), and the decode block at `:1597-1608`.
6. **`wat-scripts/scratch-pad/probe-arc278-74-literal-rtl-ctor.wat`** — the committed proof that a
   literal ctor on a parametric response's bare base name type-checks. This is your worked
   reference for what the macro must emit. Copy its shape; do not re-derive the question.

## The strike, in five parts

### 1. The check

In `synthesize_surface_protocol`'s per-member loop, immediately after the `:max-request-bytes` lock
at `:2793` and **before** the request-arg bail at `:2797`:

```rust
// #74 — `<Op>Response` is LAW (builder ruling, 2026-08-05). Compare the BASE name:
// type args are stripped, and a Parametric head is stored WITHOUT its leading colon
// while a Path re-prepends one (see the note at build_op_response_type_constants).
if enforce_rtl_lock {
    let declared_base: String = match ret {
        TypeExpr::Path(p) => p.clone(),
        TypeExpr::Parametric { head, .. } => format!(":{head}"),
        other => /* not a nameable response — refuse, located, naming what was declared */,
    };
    let surface_base = /* surface.name with any `<...>` suffix stripped */;
    let required = format!(
        "{surface_base}::{}Response",
        crate::string_ops::kebab_to_pascal_with_acronyms(name, ns_acronyms),
    );
    if declared_base != required {
        return Err(/* MalformedDecl on decl_span, naming op, surface, DECLARED and REQUIRED */);
    }
}
```

The error message must print **both** names. A reader who has just been refused needs to see what
they wrote and what the law wants, side by side, or the diagnostic is half a diagnostic.

### 2. Migrate the ten violators

Arm the check, then read what the corpus screams. It should scream exactly these ten and nothing
else — that agreement is itself a load-bearing row.

| file | surface | op | declared | do |
|---|---|---|---|---|
| `wat-scripts/probes/arc-170/probe-kwargs-peer.wat` | `:probe::Echo` | `echo` | `::Resp` | rename → `::EchoResponse` |
| `wat-scripts/probes/arc-170/probe-kwargs-peer.wat` | `:probe::Kv` | `get` | `::GetResp` | rename → `::GetResponse` |
| `wat-scripts/probes/arc-170/probe-strikeB-fields.wat` | `:probe::Kv` | `get` | `::R` | rename → `::GetResponse` |
| `wat-scripts/probes/arc-170/probe-surface-ships.wat` | `:probe::Foo` | `f` | `::Resp` | rename → `::FResponse` |
| `wat-scripts/probes/arc-170/scout-kwargs-expand.wat` | `:probe::Kv` | `get` | `::GetResp` | rename → `::GetResponse` |
| `wat-scripts/scratch-pad/probe-sift-rules-stop1-bare-defsurface.wat` | `:probe::Bare` | `echo` | `:wat::core::i64` | mint `::EchoResponse` |
| `wat-scripts/scratch-pad/probe-sift-rules-stop1-dump.wat` | `:probe::Bare2` | `echo` | `:wat::core::i64` | mint `::EchoResponse` |
| `wat-scripts/scratch-pad/probe-sift-rules-stop1-dump.wat` | `:probe::Wrapped` | `echo` | `:wat::core::i64` | mint `::EchoResponse` |
| `tests/services/probe_arc278_response_type_from_declaration.{wat,rs}` | `:probe::Odd` | `put` | `::Verdict` | **INVERT** |
| `wat-scripts/scratch-pad/probe-repl-durable-forms.wat` | `:probe::Repl` | `eval-src` | `::EvalResponse` | **INVERT** |

A rename is a rename: the enum's declaration, every construction, every match pattern. A mint is a
new `defenum` carrying `:Ok`, `:RequestTooLarge [bytes cap]` and `:RequestMalformed [path expected
got]` in the mandated shapes — copy them verbatim from any conforming service; the ops are `echo`,
so the response is `<Surface>::EchoResponse`.

**INVERT means the fixture's subject changes and it keeps proving something.** Both of those files
exist to demonstrate *"the response type's name is READ, not guessed."* Under the ruling that
proposition is false, so the fixtures must not be migrated into conformance and must not be deleted
— they become the proof that the wall stands: the declaration stays non-conforming, and the test
asserts it is now **REFUSED**, located, naming the required name. `probe-repl-durable-forms.wat` is
the file that caught R64; it becomes the file that proves R64's defect is unrepresentable. Rewrite
each file's header to say what it now proves.

### 3. Delete the emitter and the constant

`build_op_response_type_constants` (`types.rs:3106`), its `let mut` (`:3206`), its call (`:3234`),
and its `rest.extend` (`:3293`). Nothing else in that function moves — `op_budget_consts` and the
`MAX-REQUEST-BYTES` constant stay exactly as they are.

### 4. Restore the literal ctor at both wat sites

At each of the four decode blocks, the value goes back to a literal constructor call:

```clojure
(:wat::core::keyword/from-string
  (:wat::core::string::concat proto-base
    (:wat::core::string::interpolate "::{vp}Response::RequestTooLarge" :vp variant-pascal)))
```

— except it is a **call head**, so it must be spliced as a literal keyword into the quasiquote, not
built as a runtime keyword value. `variant-pascal` is already in scope at both sites (`:890`,
`:1018`). The `resp-type-const-kw` binders, the `resp-dotted`/`rtl-edn`/`rm-edn` symbol nodes, and
every `string::join`/`split`/`subs`/`edn::read` line that existed only to reach the value by decode
all go. `cap-const-kw` and the budget comparison stay untouched. Do the `RequestMalformed` twin in
the same pass — it is the same strike, and leaving it on the decode path would be half a cure.

### 5. Update the comments you invalidate

Several long comment blocks at these sites explain *why* the name must be read and cannot be a call
head. Those explanations become false the moment the law lands. Replace them with the new fact: the
name is REQUIRED, enforced at surface registration, so the concatenation is guaranteed. A comment
that survives the change it described is the next reader's trap.

## Blast radius

`src/types.rs`, `wat/service.wat`, the ten files in the table, and any test that names the deleted
constant. **No new types. No new macros. No change to `MAX-REQUEST-BYTES` anywhere.**

## ⛔ STOPs — each rejects; none is a permission slot

- **⛔ STOP-1 — if the check refuses anything outside the ten-row table, STOP and report the
  extra sites verbatim.** The table is a census taken independently on 2026-08-05 and validated
  with three controls. An eleventh violator means the census missed a class and the orchestrator
  must re-scope. Do not migrate it on your own judgement.
- **⛔ STOP-2 — if any `wat/` stdlib file or any file under `crates/` is refused, STOP.** The census
  says zero production and zero stdlib violate the law. A stdlib refusal falsifies that and the
  strike does not proceed on a falsified premise.
- **⛔ STOP-3 — do NOT touch `types.rs:2827`'s `if let TypeExpr::Path(resp_path) = ret`.** That is
  the ruling-A SHAPE lock and it has a known parametric hole, filed as task #76 with the builder's
  ruling pending. It is in the same twenty lines you are editing and it will look like an obvious
  companion fix. It is not yours. Your check reads `Parametric` for its own name comparison and
  changes nothing about that block.
- **⛔ STOP-4 — if the literal ctor fails to type-check for any real corpus service**, STOP and
  report the exact service and error. The mechanism is proven for the parametric and monomorphic
  cases by `probe-arc278-74-literal-rtl-ctor.wat`; a failure means a case that probe does not cover
  and the orchestrator owns the re-scope.
- **⛔ Do not add a `_` wildcard arm on any enum scrutinee.** Doctrine; the exhaustiveness error's
  own suggestion text offers it and taking it is a rejected strike.
- **⛔ Do not commit, stash, push, or touch git in any way.** Leave the tree dirty. The orchestrator
  weighs and commits.
- **⛔ Do not hand-edit `.wat` for a multi-site structural rewrite.** Ten declarations in ten files
  is a hand-edit's size, so hand-editing is correct here — but if you find yourself doing the same
  mechanical rewrite across many more files than the table lists, that is STOP-1 firing.

## Verify

Run your verification in the **FOREGROUND** and block on it. Your turn ends when the numbers are in
your hands, not when a command is launched.

```
cargo build --release
target/release/wat --check <each file you touched>     # ~0.2s each, the fast per-file arbiter
cargo nextest run --release
```

`cargo build --release` going green proves nothing about this change — the bake does not run the
corpus sweep, and this arc has already been fooled by exactly that. Read the **Summary line** of
`cargo nextest run --release`; never a piped exit code. The pre-change floor is
**4348 run / 4348 passed / 0 failed / 262 skipped**.

Report: the Summary line verbatim, every file you touched, the full list of sites the new check
refused before you migrated them, and anything you had to assume.
