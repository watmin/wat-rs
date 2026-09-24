# BRIEF — STONE 255.15: infer a type variable from the implementor's surface binding

**Drawn 2026-09-23 against `main` @ `1a93de938`.** Floor 6014/6014, clippy 0, census `no STOP-8`,
delta **3 / RECOVERY 0**, heresy ledger **220**. **Executor tier: Opus.**

## Why this stone exists — step 1 of the narrow waist

The builder ruled the **locus is a narrow waist** (SEAM): services × loci meet at one interface,
**N + M, never N × M**, and loci are *open but discrete* — many networked loci are coming (UDS, loopback
and remote TCP/UDP/HTTP/HTTPS/DTLS/QUIC/mTLS…). Today each locus stamps a **transport type** into the
service handle (`Address<S,R,T>`, Shared vs Wire — `a6da457e3`), which makes *a process dialling a
shared-memory address* a **type error**. To do that, every service mints one `start` copy **per
transport** — N × M. **One generic `start` needs the type checker to infer the transport from the
locus it is given.** That inference is this stone. Nothing else.

Read first, in order: `FINDING-generic-lifecycle-and-the-transport-type.md` (the measurement, with the
full probe text at its end) and `FINDING-the-locus-waist-is-three-layers-wide.md` (why it matters).

## The gap — measured, A/B

Same surface, same implementor, only the parameter's type argument changes:

| parameter typed as | given | today |
|---|---|---|
| `(Loc :- [Shared])` — concrete | `Th` (`extend-type Th (Loc :- [Shared])`) | ✅ rc=0 |
| `(Loc :- [Shared])` — concrete | `Pr` (binds `Wire`) | ✅ rc=1 `expects (Loc :- [Shared]); got Pr` |
| `(Loc :- [T])` — **type variable** | `Th` | ⛔ rc=1 `expects (Loc :- [_]); got Th` |

**The checker does not bind a function's type variable from the `extend-type` binding of the implementor
it is given.**

## The rooms — read in this order

1. **`src/check.rs:17297` `assignable`, the arm at ~`:17372`** — *"Arc 170 C2 Gap 1 — a CONCRETE type
   satisfies a PARAMETRIC-SURFACE param iff its FULL-ARGS extend-type edge exists … keyed by the full
   parametric string"*. **This is where the concrete case is decided and the type-variable case falls
   through.**
2. **`src/types.rs:2151`** — the extend-type edge is stored **keyed by the target keyword verbatim**. A
   type-variable expectation cannot match a string key.
3. ⭐ **`src/check.rs:5449`** — *"Arc 170 C2 Gap 2 — ABSTRACT parametric-surface receiver"*. **The
   receiver case already binds type variables from the implementor** (measured: a surface method with
   `self <- (Svc :- [H St])` dispatches to two implementors binding different types, each enforced).
   **The fix is likely Gap 2's binding, brought to Gap 1's position.** Read it before designing.
4. `src/check.rs:16713` `unify` — where type variables are bound into the substitution.

## The sketch

When the expected type is a parametric surface whose arguments contain type variables, find the actual
type's extend-type edge(s) to that surface **by head**, and **unify** the edge's arguments with the
expected arguments so the variables bind in the substitution — structurally, as `TypeExpr`s.
⛔ **Not by building a string and looking it up**: comparing type names as strings is the recurring
defect class `CLAUDE.md` names by name, and the ruling *exactly one way to do things* forbids a second
matching path beside a structural one.

**Blast radius:** `src/check.rs` (and `src/types.rs` only if the edge store needs a by-head lookup),
plus tests. **No `.wat` stdlib change. No service-macro change. No transport registry.** Those are
steps 2 and 3.

## ⛔⛔ THIS CURE RUNS IN THE PERMISSIVE DIRECTION

It makes the checker **accept more.** 255.12 is the precedent — its adversarial row caught its own
first draft. **The soundness rows are the stone:**

1. ⭐ `(defn start :- [T] [loc <- (Loc :- [T])] -> :T …)` given `Th` **checks, and `T` binds to
   `Shared`** — the result is usable as `Shared`, and running it returns the right value.
2. ⛔ **Wrong transport, now refused for the RIGHT reason:** `(defn f [] -> Wire (start (Th …)))` must be
   rc=1 **`ReturnTypeMismatch`** — `T` inferred `Shared`, declared `Wire`. (Today it fails too, but for
   the gap — which proves nothing.)
3. ⛔ **Conflicting bindings:** `(defn g :- [T] [a <- (Loc :- [T]) b <- (Loc :- [T])] …)` given `Th`
   and `Pr` must be refused — one `T` cannot be both `Shared` and `Wire`.
4. ⛔ **A non-implementor is still refused** (`NotLoc`).
5. ⛔ **Ambiguity:** can one type `extend-type` the same parametric surface twice with different
   arguments? **Measure it.** If it can, inference must refuse the ambiguous call, not pick one.
6. ✅ **The concrete case is unchanged** — both rows of the A/B above, byte-identical diagnostics.

## STOP triggers

1. If binding a type variable from inside `assignable` needs the substitution to be threaded somewhere
   it is not today, and that is more than a local change — **STOP, report the shape of the change, do
   not restructure the unifier.**
2. If the fix needs a second, string-keyed lookup beside the structural one — **STOP.**
3. If soundness row 3 or 5 cannot be made to refuse — **STOP.** A permissive inference that picks a
   binding silently is worse than the gap.

## Expectations — fixed before the strike

| what | command | expected |
|---|---|---|
| the gap closes | `--check` the type-variable probe | rc=0 (today rc=1) |
| `T` flows to the result | run it | the bound transport's value |
| wrong transport | `--check` row 2 | rc=1 `ReturnTypeMismatch` |
| conflicting bindings | `--check` row 3 | rc=1 |
| non-implementor | `--check` row 4 | rc=1 `TypeMismatch` |
| concrete A/B unchanged | `--check` both | rc=0 / rc=1, diagnostics byte-identical |
| floor | `scripts/floor.sh` | green |
| clippy | `cargo clippy --all-targets --workspace -- -D warnings` | 0, **not cached** |
| census · delta | `census.sh --diff` · `scripts/replay/delta.sh` | `no STOP-8` · NEW 3, RECOVERY 0 |
| ledger | `keyword_heresy` | 4/4; **report the shape mix** if it moves |

Runtime prediction: 1–3 hours. Trap-doors: the verbatim-string edge key; substitution access from
`assignable`; a binding that succeeds on one parameter and conflicts on another.

## Doctrine

- ⛔ **THERE IS NO KNOWN FLAKE. A RED IS A RED.** Capture whole, never re-run to green, name the arm.
  ⚠ Bookkeeping exception: `harvest_wrap_split` (`278-rules-engine/FINDING-harvest-cost-is-not-a-gate.md`)
  — **cite and report, never a pass, never re-run to clear. Name it either way.**
- ⭐ **Prove every probe can say BOTH words** before trusting it; strip ANSI from live tool output
  (`sed 's/\x1b\[[0-9;]*m//g'`); carry a control in each run.
- ⛔ **If this brief contradicts the code, THE CODE WINS — say so plainly.** The rooms above were located
  this session, but five briefs this week named the wrong address. **Verify them.**
- Fixtures are committed tests (a `.wat` and `.wat.bad` family, driven from Rust). Work only in
  `/home/john/work/holon/wat-rs`. **Commit locally; do not push.**
