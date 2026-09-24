# BRIEF — STONE 255.17a: the generated child main says `Wire`

**Drawn 2026-09-24 against `main` @ `2b9f6aa94`.** Floor 6027/6027, clippy 0, census `no STOP-8`, delta
**3 / RECOVERY 0**. **Executor tier: Opus.**

## Where this sits (ruled 2026-09-24)

255.17 stopped at STOP-1. The root is that **the checker knows by spelling what it should know by
declaration**: `is_type_param_letter` ("one uppercase letter, or `Xt`") decides what a type parameter
is (8 sites), and "the last argument, spelled like a letter" decides which slot is the transport (5
sites). The four questions decomposed the cure into three stones, ruled in this order:

- **C-a (this stone):** the generated child main says `Wire`.
- **C-b:** the transport slot is declared, and D2 closes.
- **C-c:** a type parameter is known from its declaration.

The narrow 255.17 fix and its 20 test files are **parked**, uncommitted, in the session scratchpad
`park-255.17/`: `check.rs.patch` (sha256 `fb959833…`) plus the `tests/types/` files listed in
`untracked.txt`. The patch re-applies cleanly to `2b9f6aa94`.

## Why — the child's transport is a free, undeclared name

`defservice` emits a child `:user::main` that runs **only in a forked process**. It types its self-peer
with `status-ty-runtime` (`wat/service.wat:1108`), which is today a copy of `status-ty-ann`, i.e.
`(<S>::Status :- [~@fqdn-tp-syms <transport-param>])`. The transport letter there is not declared by
anything in the child. Its own comment (`:2420-2428`) calls it *"FREE type vars … one erased
instantiation."* It passes only because of the spelling hole. With the 255.17 fix applied, **57 tests in
35 files go red on exactly this site**: `:wat::kernel::send: parameter payload expects (<S>::Status :-
[… :T]); got (<S>::Status.Started :- [… :wat::kernel::Wire])`, every frame `ProcessOpts/launch`. The
child is a process, so its transport **is** `Wire`. Saying so is the truth, not a workaround.

## The work

1. In `wat/service.wat`, make `status-ty-runtime` spell its transport slot `:wat::kernel::Wire`:
   `fqdn-tp-syms` followed by `Wire`, instead of `handle-tp-syms`. `status-ty-runtime`'s one use is
   `:2432`. **Measure that**, and measure whether `admin-ty-runtime` (`:1094`) or any other type inside
   `child-main-form` carries the free transport letter. Every one gets the same treatment, and each is
   named in the SCORE.
2. Rewrite the `:2420-2428` comment so it states what is now true: the transport is `Wire`. The service's
   own type parameters are a separate matter: say what they are, and do not claim they are closed.
3. **Measurement, not landing:** with your change committed, re-apply the parked patch **plus** its
   tests on top and run the floor. Report whether the 57 clear and what, if anything, stays red, each
   whole block verbatim. Then **revert the patch and the parked tests out of the tree.** They land in
   C-b, not here.

## STOP triggers

1. The change turns any test red on its own (without the parked patch) → STOP, report each red block
   verbatim.
2. With the patch applied, a red remains that is **not** the child main → report it verbatim. That is
   C-b's fallout, measured here. Do not fix it.

## Expectations — fixed before the strike

| what | command | expected |
|---|---|---|
| C-a alone | `scripts/floor.sh` | 6027 passed, 0 failed |
| C-a + the parked patch and tests | `scripts/floor.sh` | **measured**: the 57 are expected to clear; report the Summary and every remaining red |
| clippy · census · delta (C-a alone) | the usual | 0 · `no STOP-8` · NEW 3 / RECOVERY 0 |
| the tree after | `git status --porcelain` | empty; one local commit |

Runtime prediction: 45–75 min, of which two floors are about 10.

## Doctrine

- ⛔ **THERE IS NO KNOWN FLAKE. A RED IS A RED.** Capture the whole log, never re-run to green, name the
  arm. (`harvest_wrap_split`: cite it, report it.)
- `wat/service.wat` is the macro's source, not a corpus migration: a direct edit is correct here.
- If a macro form confuses you, `macroexpand` first. A `defservice` expands at the form level.
- **If this brief contradicts the code, the code wins — say so plainly.** Commit locally with `git add --
  <paths>`; **do not push.** Spawn no subagents.

## Out of scope

C-b (the transport declaration, D2), C-c (declared type parameters), the service's own type parameters
being free in the child.
