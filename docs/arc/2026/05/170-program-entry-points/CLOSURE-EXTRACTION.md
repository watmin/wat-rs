# Arc 170 — Closure extraction (substrate primitive deep-dive)

> **Status:** v2 (2026-05-09; supersedes v1 below). v1 spec'd a
> synthetic-name + `entry: String` approach that re-introduced the
> entry-keyword ceremony DESIGN explicitly killed. v2 retires the
> synthesis: the fn-form AST evaluates to a fn Value directly. See
> [`REALIZATIONS-SLICE-1.md`](./REALIZATIONS-SLICE-1.md) for the
> discipline lesson. Slice 1 (commit `787c977`) shipped against v1;
> slice 1b reshapes to v2.

The load-bearing substrate work for arc 170. Arc 170's spawn-process
takes a fn directly; the substrate must turn that fn into a portable
description (a captured-environment prologue + an entry expression
that evaluates to a fn Value) suitable for shipping to a forked OS
process. This doc is the algorithm + invariants + test strategy.

**Scope:** Rust-internal capability in arc 170. NOT exposed at wat
level. Future remote-program arc may surface it for over-the-wire
transport.

---

## v2 — corrected algorithm + shape

**The honest principle:** "the fn IS the program." A fn-form
expression like `(fn [stdin :IOReader stdout :IOWriter stderr :IOWriter] :nil ...)`
already evaluates to a fn Value. The substrate's evaluator turns
fn-forms into fn Values directly. Closure extraction does NOT need
to wrap the fn in a `define` + look up by name; it keeps the entry
as an expression.

### Public shape (v2)

```rust
pub struct ClosurePackage {
    pub prologue: Vec<WatAST>,  // type defs + dep defs (the captured
                                // environment); does NOT include the
                                // entry as a trailing define
    pub entry_form: WatAST,     // an expression evaluating to a fn Value:
                                //   - inline-lambda input: the fn-form
                                //     AST itself
                                //   - keyword-path input:  a Symbol AST
                                //     that resolves into prologue's defines
}
```

**No `entry: String` field.** The entry is an expression, not a name.

### Consumer pattern (v2)

```rust
let pkg = extract_closure(&fn_value, sym, &types)?;
let frozen = startup_from_forms(pkg.prologue, ...)?;
let fn_value = eval(&pkg.entry_form, env, frozen.symbols())?;
let result = apply_function(fn_value, args, frozen.symbols(), span)?;
```

The consumer freezes the prologue (which seeds types + symbol
table), then evaluates `entry_form` in that frozen world — the
fn-form AST evaluates to a fn Value, which gets applied to args.

### Algorithm (v2 deltas vs v1)

The free-symbol walker, dep-closure builder, capture encoding,
portability check, and topological sort are all unchanged from v1
(slice 1 implemented them correctly). The deltas are:

**Step 1 (entry resolution) — v2:**

- For inline-lambda input: `entry_form = the fn-form AST itself`,
  reconstructed from the fn Value's params + body + ret_type. No
  synthetic name is generated. No define wrapper.
- For keyword-path input: `entry_form = (Symbol :my::worker)` (a
  Symbol AST). The user's existing `(:wat::core::define :my::worker (fn ...))`
  is in `prologue` as a dep (it's a user defn the closure-extraction
  walker pulled in); evaluating the symbol resolves to the fn Value
  bound there.

**Step 6 (assembly) — v2:**

`prologue: Vec<WatAST>` contains:

1. Type definitions (struct / enum / newtype / typealias) in
   topological order
2. Capture binding defines (if any)
3. User dependency defines in topological order (deps before
   consumers) — INCLUDING the user's existing define for the entry
   fn IF the input was a keyword path

`entry_form: WatAST` contains:

- The reconstructed fn-form AST (inline-lambda input), OR
- The Symbol AST naming a fn defined in `prologue` (keyword-path
  input)

The entry is NEVER a trailing define in `prologue`. Position is
the contract: prologue is the captured environment; entry_form is
the program.

### Invariants (v2)

**I1. Self-contained freeze.** `startup_from_forms(package.prologue)`
succeeds. No undefined symbols, no missing types.

**I2-v2. Entry evaluates to a fn Value.** `eval(&package.entry_form,
env, frozen.symbols())` returns `Value::wat__core__fn` (or
equivalent). Replaces v1's "entry resolvable by name" invariant.

**I3. Behavioral equivalence.** For any `Vec<Value>` inputs that
match the entry fn's signature: applying the fn Value (resolved
from `entry_form` in the frozen fresh world) produces the same
observable side effects as invoking the original fn in the
parent's world (modulo the IPC mechanism's wire effects).

**I4. No substrate primitive leakage.** `package.prologue` does
NOT contain `(:wat::core::define ...)` for substrate primitives.

**I5-v2. No synthesized names.** `package.prologue` does NOT
contain any `:__closure::__pkg_<n>` synthetic identifier. The
counter machinery is retired. Capture-binding names (e.g.,
`:wat::kernel::__closure::__captured_X` from v1's step 4) follow
their own naming convention; they are NOT entry names.

### Tests (v2 deltas)

T1-T15 from slice 1 stay structurally; assertions update:

- T1-T15 assertions on `pkg.entry` (string) → assertions on
  `pkg.entry_form` (WatAST shape)
- For inline-lambda inputs: assert `pkg.entry_form` is a
  fn-form AST `(fn [params...] -> :T body...)` matching the
  input fn's signature
- For keyword-path inputs: assert `pkg.entry_form` is a Symbol AST
  matching the input keyword
- Behavior-equivalence tests: freeze `prologue` + `eval(entry_form)`
  → fn Value → apply to test args → compare against parent-world
  invocation

---

## v1 — original spec (HISTORICAL; supersded by v2)

> The text below is the original CLOSURE-EXTRACTION.md spec that
> drove slice 1's implementation. It is retained as historical
> record per `feedback_inscription_immutable.md`. The
> synthetic-name + `entry: String` approach below is RETIRED in
> v2 above; do not implement against v1.

---

## Purpose

Given a `Value::wat__core__fn` value, produce a Vec\<WatAST\> that:

1. Includes the fn's body AST as a top-level definition
2. Includes the transitive closure of every type/symbol the body
   references that's NOT a substrate primitive
3. Includes synthesized binding forms for every captured runtime
   value (let-scope locals captured by the fn closure)
4. Is freezable in a fresh wat world such that the entry fn can be
   invoked there

The produced Vec\<WatAST\> is the **portable description of the
program**. Where it goes (forked process via `fork-program-ast`,
remote host via socket, disk via serialization) is the consuming
operation's concern.

---

## The algorithm

### Inputs

- `fn_value: &Value::wat__core__fn` — the function to package
- `parent_world: &SymbolTable` — the parent's symbol table (for
  resolving free symbols)
- `parent_types: &TypeEnv` — the parent's type environment (for
  resolving type references)
- `entry_name: Option<&str>` — caller-supplied name if the fn was
  passed as a keyword path; else None (lambda case)

### Output

- `Result<ClosurePackage, ExtractionError>`
- `ClosurePackage`:
  - `forms: Vec<WatAST>` — top-level forms ready for child-world
    freeze
  - `entry: String` — the fn's name in the produced forms
    (canonical if caller passed a keyword; synthetic if lambda)

### Steps

**1. Resolve the entry fn's body + signature.**

If `fn_value` is a top-level defn (resolvable via `parent_world.get`),
use its existing `(:wat::core::define ...)` AST as the entry form.
Use the original keyword path as the entry name.

If `fn_value` is an inline lambda or factory result (no canonical
name in `parent_world`), mint a synthetic name:

```
:wat::kernel::__closure::__pkg_<counter>
```

Synthesize a `(:wat::core::define ... lambda-form)` AST at that
name. Use the synthetic name as the entry.

**2. Walk the entry fn's body for free references.**

Algorithm:
- Track scope: fn parameters + nested let bindings + nested fn
  parameters all introduce scope. Names bound at any level are
  LOCAL.
- A reference is FREE if it's not bound in any enclosing scope.
- Recurse into list / vector / struct-pattern / etc.

For each free reference:
- If it's a Symbol matching a parent symbol-table entry:
  - If the entry is a `:wat::core::*` substrate primitive: SKIP
    (already in child substrate)
  - Else: this is a USER DEPENDENCY → record for extraction
- If it's a Keyword matching a parent type registry entry:
  - If the entry is a `:wat::core::*` substrate type
    (i64/f64/bool/String/etc.): SKIP
  - Else: this is a USER TYPE → record for extraction
- If it's a Symbol bound in the fn's CLOSURE ENV (let-scope local
  captured by lambda):
  - This is a CAPTURED VALUE → record the value for AST encoding

**3. Recursively extract user dependencies.**

For each user defn / type recorded in step 2:
- Find the defining AST in `parent_world` (the original
  `(:wat::core::define ...)` or `(:wat::core::struct ...)` etc.)
- Walk it for further free references (recurse into step 2)
- Add to the dep set

Continue until no new references are added (fixpoint).

**4. Encode captured runtime values to AST.**

For each captured Value:

| Value kind | Encoding |
|---|---|
| `i64`, `f64`, `bool` | direct literal AST |
| `String` | string literal AST |
| `nil` | `:wat::core::nil` keyword |
| `Vector<T>` | `(:wat::core::Vec elem1 elem2 ...)` recursing on elements |
| `HashMap<K,V>` | `(:wat::core::HashMap (k1 v1) (k2 v2) ...)` recursing |
| `Struct(fields)` | `(:wat::core::TypeName/new field1 field2 ...)` via existing `struct→form` (arc 091 slice 8) |
| `Enum::Variant(payload)` | variant constructor form |
| `Option::Some(v)` | `(:wat::core::Some <v>)` recursing |
| `Option::None` | `:wat::core::None` keyword |
| `Result::Ok(v)` | `(:wat::core::Ok <v>)` recursing |
| `Result::Err(e)` | `(:wat::core::Err <e>)` recursing |
| `Tuple(elems)` | `(:wat::core::Tuple e1 e2 ...)` recursing |
| `Bytes` | `(:wat::core::Bytes/from-hex "...")` |
| Channel-bearing types (Sender / Receiver / Channel / Thread / Process / IOReader / IOWriter / HandlePool / ...) | NOT PORTABLE — see step 5 |

For each captured value with a name `X` in the closure env:

```scheme
(:wat::core::define :wat::kernel::__closure::__captured_X <encoded-ast>)
```

The fn's body is rewritten to reference `:wat::kernel::__closure::__captured_X`
instead of the original local name `X`. Or — preserve the original
name `X` and emit the binding as a top-level define under that
name (might collide with extracted symbols; mint synthetic name if
collision). Lean: synthetic names with `__captured_` prefix to
avoid collisions; rewrite body references.

**5. Portability type-check.**

Walk the captured values' types. If ANY type is in the
non-portable set, FAIL with `ExtractionError::NonPortableCapture`:

Non-portable type set:
- `:wat::kernel::Sender<T>`
- `:wat::kernel::Receiver<T>`
- `:wat::kernel::Channel<T>`
- `:wat::kernel::Thread<I,O>`
- `:wat::kernel::Process<I,O>`
- `:wat::kernel::HandlePool<T>`
- `:wat::io::IOReader`
- `:wat::io::IOWriter`
- Any type that transitively contains one of the above (e.g., a
  struct with a `Sender<T>` field)

Diagnostic shape (substrate-as-teacher):

```
spawn-process closure captures `:my::tx` of type `:wat::kernel::Sender<i64>`.
Channel-bearing types cannot cross process boundaries (different memory).
Use stdin/stdout/stderr pipes for inter-process communication, or
restructure the program so the channel is created in the spawned program.
```

**6. Assemble the ClosurePackage.**

Output Vec\<WatAST\>:

1. Type definitions (struct / enum / newtype / typealias) in
   topological order
2. Capture binding defines (if any)
3. User dependency defines in topological order (deps before
   consumers)
4. The entry fn's defining AST (last)

Entry name: keyword path of the entry's defining symbol.

Return `ClosurePackage { forms, entry }`.

---

## Invariants

The produced ClosurePackage must satisfy:

**I1. Self-contained freeze.** `startup_from_forms(package.forms)`
succeeds. No undefined symbols, no missing types.

**I2. Entry resolvable.** `frozen.symbols().get(&package.entry)`
returns Some(fn).

**I3. Behavioral equivalence.** For any `Vec<Value>` inputs that
match the entry fn's signature: invoking the entry in the frozen
fresh world produces the same observable side effects as invoking
the original fn in the parent's world (modulo the IPC mechanism's
wire effects — pipes vs in-memory channels).

**I4. No substrate primitive leakage.** `package.forms` does NOT
contain `(:wat::core::define ...)` for substrate primitives. Those
are already in the child substrate.

**I5. Type closure is complete.** Every type referenced by any
form in `package.forms` is either (a) a substrate primitive type or
(b) defined within `package.forms`.

**I6. Topological dep ordering.** Forms appear in dep order — a
`(:wat::core::define :foo ...)` referencing `:bar` must appear
AFTER `(:wat::core::define :bar ...)`.

**I7. Captures match closure env.** For each captured value `X` in
the original fn's closure env, `package.forms` contains a
`(:wat::core::define :__captured_X ...)` whose evaluation produces
a value equal to the original captured value.

**I8. Portability check passes.** All captured values pass the
portability check. If any captured value is non-portable,
extraction returns `Err(NonPortableCapture { ... })` instead of a
package.

---

## Test strategy

### Rust integration tests (slice 1 deliverable)

Each test exercises closure extraction on a fn shape, asserts the
resulting Vec\<WatAST\>:
1. Re-freezes successfully
2. Has the expected entry symbol
3. When invoked produces the expected behavior

Test cases (mirroring the canonical fn shapes from DESIGN.md § "The
algorithm"):

**T1. Top-level defn, no deps, no captures.**
Defn body uses only substrate primitives. Extract, re-freeze,
invoke. Behavior matches.

**T2. Top-level defn, calls other top-level defns.**
Recursive dep extraction. Verify dep defns are in `package.forms`
in topological order.

**T3. Top-level defn, uses user types.**
Struct / enum / newtype / typealias types extracted. Verify type
defs precede their consumers.

**T4. Inline lambda, no captures.**
Lambda body uses only substrate primitives. Synthetic entry name
minted. Verify.

**T5. Inline lambda captures let-scope value.**
`(let [config (...)] (fn [...] (use config)))` Capture extracted;
binding form synthesized; body rewritten to reference synthetic
name. Re-frozen world produces same behavior.

**T6. Lambda captures multiple values, mixed types.**
Captures i64 + struct + Vector. All encoded; all bindings
synthesized. Re-freeze + invoke.

**T7. Factory pattern.**
`(let [worker (my-factory my-config)] worker)` returns fn
capturing `my-config`. Extract on `worker`. Same as T5
structurally — capture extracted, rebound in package.

**T8. Lambda captures non-portable value (NEGATIVE).**
Captures a Sender\<T\>. Extraction returns
`Err(NonPortableCapture { name: "tx", type: ":Sender<i64>" })`.
Diagnostic message includes the substrate-as-teacher hint.

**T9. Lambda captures struct holding a Sender field (NEGATIVE).**
Captured struct's transitive closure includes a non-portable type.
Same Err shape; diagnostic names the offending field path.

**T10. Captures with type alias.**
Captured value's type is a typealias for another type. Both
captured (typealias defn + underlying type if nominal).

**T11. Captures with recursive struct.**
Struct with a field of type :Vector\<:Self\>. Verify type def is
captured once; recursive ref doesn't loop.

**T12. Body uses macro that expands to use a primitive.**
After macro expansion, body references substrate primitives only.
Verify post-expansion body is in `package.forms`.

**T13. Body uses user-defined macro.**
Macro defn extracted; macro expanded into body; expanded form is
in `package.forms`.

**T14. Body calls user fn that calls another user fn (transitive).**
Three-level dep chain. All three defns extracted in topological
order.

**T15. Verify behavior equivalence end-to-end.**
For each of T1-T7, extract; freeze fresh world; invoke entry with
test input; compare against invoking the original fn directly.
Outputs match.

### Wat-level integration tests (slice 2 deliverable)

These exercise spawn-process(fn) end-to-end, indirectly validating
closure extraction.

- spawn-process(top-level-defn-fn) forks; child runs; behavior matches
- spawn-process(inline-lambda) forks; child runs; capture preserved
- spawn-process(factory-result) forks; child runs; captured config used
- spawn-process(fn-capturing-Sender) freeze fails with portability error

---

## Existing wat-rs pieces leveraged

The closure extraction is built on top of existing substrate
infrastructure:

| Capability | Source | Used for |
|---|---|---|
| AST walker patterns | check.rs / runtime.rs (free-variable analysis already happens during inference) | Step 2 — free-symbol walking |
| Symbol table lookup | runtime.rs::SymbolTable::get | Step 3 — finding defining ASTs |
| Type registry lookup | types.rs::TypeEnv::get | Step 3 — finding type defs |
| `struct→form` | runtime.rs (arc 091 slice 8) | Step 4 — Value→AST for structs |
| `:wat::test::program` builder | (arc 113) | Conceptual precedent for forms-as-data |
| `fork-program-ast` pathway | fork.rs::eval_kernel_fork_program_ast | Slice 2's spawn-process consumer of ClosurePackage.forms |
| `startup_from_forms` | freeze.rs | Slice 1 verification — re-freezing extracted forms |
| `apply_function` | runtime.rs | Slice 1 verification — invoking entry post-freeze |

---

## Why Rust-internal in arc 170

The user direction was "build this first then use this as a
dependency in the rest of the re-work" — keeping the closure
extraction internal to spawn-process for arc 170 minimizes
user-facing surface while landing the substrate capability.

Reasons NOT to expose at wat level in arc 170:

1. **YAGNI principle.** spawn-process is the only consumer needed
   in arc 170. Exposing the primitive without a clear
   user-facing use case adds API surface to maintain.
2. **Wire format hasn't settled.** Future remote-program work
   will lock the EDN-bytes serialization; closure extraction
   exposed at wat level should match that wire format. Better to
   defer the wat-level surface until remote-program shapes the
   wire.
3. **Naming hasn't been pressure-tested by usage.** Once we have
   real consumers (remote-program; serialize-to-disk; etc.), the
   wat-level naming concerns can be settled with concrete
   examples driving the choice.

Reasons to expose later:

- Future remote-program needs the wat-level surface to ship
  programs over sockets
- Serialize-to-disk for program persistence
- Test harnesses that want to verify extraction without spawning
- Debugging / introspection ("what does this fn capture?")

When that future arc opens, the slice 1 Rust capability becomes
the implementation; only the wat-level wrapper + naming need to
land.

---

## Future remote-program connection

The remote-program arc (queued post-170 per scratch/2026/05/007)
builds DIRECTLY on arc 170's closure extraction:

```
spawn-process(fn):
  package = closure_extract(fn)
  fork_pid = unsafe libc::fork()
  if child:
    world = startup_from_forms(package.forms)
    invoke world.symbols.get(&package.entry)(args)

spawn-remote-program(fn, endpoint):
  package = closure_extract(fn)              ← SAME primitive
  bytes = edn_serialize(package.forms)
  remote_handle = open_socket(endpoint)
  remote_handle.send(bytes + entry-name + args-init)
  // remote side: deserialize, freeze, invoke; respond via Q-channel
  return RemoteProgram(remote_handle)
```

Same closure extraction. Different transport. The substrate's
ABILITY to ship a wat program anywhere comes from this primitive.

The remote arc will add:
- EDN serialization wrapper around `package.forms` (already exists
  per arc 092)
- Socket transport (UDS / HTTP / TLS / mTLS — four-tier model)
- Q-channel multiplex protocol on the wire
- Endpoint addressing + auth

---

## Open questions for slice 1 implementation

These are implementation details that will surface during slice 1.
Pre-emptive flagging:

**Q-impl-1. Macro expansion in body.** When walking the entry fn's
body for free symbols, are macros already expanded? Today's
substrate expands macros pre-freeze. If the fn was created post-
expansion, the body is post-expansion. If pre-expansion (via
quoted form?), need to expand first. Lean: assume post-expansion;
if pre-expansion shapes surface during slice 1, extend.

**Q-impl-2. Captured fn values.** A captured value can itself be a
fn (closures-of-closures). The captured fn has its own deps +
captures. Recursive extraction. Lean: handle as a sub-extraction
when encountering a captured fn Value; merge into parent
ClosurePackage.

**Q-impl-3. Symbol-table snapshot timing.** The parent's symbol
table can change between Program/package-time and spawn-process-
time (in principle). Lean: snapshot at package-time; ship the
snapshot; child world is frozen from snapshot. But arc 170 doesn't
expose Program/package as a separate operation — it's all internal
to spawn-process. So timing is monotonic; no snapshot question.

**Q-impl-4. Recursive type definitions.** A struct with a field
of type :Vector<:Self>. Type extraction must avoid infinite
recursion. Lean: visited-set during type closure walk.

**Q-impl-5. Span preservation.** When extracting forms, should
spans (source locations) be preserved or stripped? Lean: preserve
for diagnostics; spans don't affect freeze.

These are all "implementation will reveal" questions. Slice 1
iterates against them; SCORE-SLICE-1 records the resolutions.
