# BRIEF — a dial failure says which of three things happened

**Read `DESIGN.md` beside this first.** It carries the one contract decision (one stdlib helper, not
105 hand-written arms), why the helper *must* be stdlib, the 18 already-correct sites that are out of
scope, and five trap-doors — the first of which says this brief's own number is a lower bound.

⛔ **Whether a dial failure raises is UNCHANGED.** Only what it says changes.

## The work, in one paragraph

Thirty-five redial sites in live `defservice` code swallow all three `ConnectOutcome` failure variants
in a `_` wildcard and hardcode *"peer is dead, not a broken pipe"* — which is true for `Refused`, false
for `Rejected` (a **different process** answered: a stale address) and false for `Failed` (an io error).
Add one stdlib helper that names the variant and carries `(Failure/message c)`, then **codemod** those
sites to call it. The 18 sites that already name the variants are out of scope.

## The rooms

| where | why you are going there |
|---|---|
| `wat-scripts/queue/sqs.wat:255`–`:258` and `:1949`–`:1950` | ⭐ THE HONEST EXEMPLAR — three variants named, each raising with `(:wat::kernel::Failure/message c)` so the carried cause survives. This is the behaviour the helper packages. |
| `wat-scripts/queue/sqs.wat:250` · `:832` | two collapsing sites, verbatim, for the shape you are replacing. |
| `wat-scripts/fanout/circuit.wat:1133`–`:1140` | a collapsing site **in full context** — a `RecvOutcome::Lost` arm that redials and continues, whose nested `_` is the raise. Read it to see why the census could not see this (the body is a `let`). |
| `wat/service.wat:4139`–`:4168` `require-stopped` / `stop-faced` / `require-granted` | the stdlib-helper precedent — signature shape, doc-comment style, and where yours goes. |
| `the-benchmark-has-more-than-one-publisher/SCORE.md` | why the helper cannot live in a script: *"a process child is assembled from service-forms only. It cannot see `circuit.wat` helpers."* |
| `wat/fix.wat` — the framework | ⛔ THE CODEMOD. Read its header, including BOOTSTRAP/STASH-DANCE (only relevant to the `wat/service.wat` half). |
| `wat-scripts/fixes/*.wat` | recorded migrations — **copy one as the shape.** `add-malformed-arm.wat` is the closest: it too adds arms to a match over an outcome enum. |
| `wat/service.wat:2549` | the comment stating the doctrine this repairs: *"its `cause` must NOT vanish … the exact masking this arc forbids."* |

## Implementation sketch

```wat
;; wat/service.wat — beside require-granted / stop-faced
(:wat::core::defn :wat::service::redial-failed! :- [I O]
  [site <- :wat::core::String  o <- (:wat::kernel::ConnectOutcome :- [:I :O])] -> :T
  (:wat::core::match o
    ;; Connected is the caller's business — it never reaches here
    ((:wat::kernel::ConnectOutcome::Refused c)
      (:wat::kernel::assertion-failed! <site + "dial REFUSED (nothing listening)" + (Failure/message c)> …))
    ((:wat::kernel::ConnectOutcome::Rejected c)
      (:wat::kernel::assertion-failed! <site + "dial REJECTED — a DIFFERENT process holds that address (stale capability, NOT a death)" + (Failure/message c)> …))
    ((:wat::kernel::ConnectOutcome::Failed c)
      (:wat::kernel::assertion-failed! <site + "dial FAILED (io error reading peer-cred)" + (Failure/message c)> …))))
```

⚠ **The `Connected` arm is the caller's**, so the helper's parameter cannot be the whole outcome unless
it also handles `Connected`. Two shapes are available — the helper takes the outcome and returns the
peer on `Connected`, or the call site keeps its `Connected` arm and passes the outcome only on the
wildcard. **Pick one, say why, and make the call sites uniform.**

Each collapsed site then becomes:

```wat
(_ (:wat::service::redial-failed! "queue: redial inbox" o))
```

## Blast radius

`wat/service.wat` (+1 helper, stdlib — rebuild), the three service scripts (35 sites via codemod), and
one recorded migration under `wat-scripts/fixes/`. **Expected 0 elsewhere.**

## STOP triggers

1. ⛔ **STOP-1 — the codemod's census is the authority, not this brief's "35".** If it finds a
   materially different number, report it **with the matches** and do not reconcile it to 35. The
   orchestrator's pattern only saw single-line wildcards.
2. ⛔ **STOP-2 — if the census finds a `_` over `ConnectOutcome` that genuinely should not care which
   variant**, STOP and report it. Do not rewrite it to satisfy a count.
3. ⛔ **STOP-3 — do NOT change whether a dial failure raises.** Supervision is a deferred builder
   ruling; this stone is the diagnosis.
4. **STOP-4 — do NOT sweep the 18 already-correct sites** into the helper. Report whether they should
   later; that is a separate, cheap stone.
5. **STOP-5 — if this cannot be done as a codemod** and needs hand-edits across three files, STOP and
   say why. Hand-editing `.wat` across a corpus is the thing the codemod exists to prevent.

## Verify

- `cargo build --release` **and** `cargo nextest run --release --no-run`.
- `./scripts/floor.sh`, read the **Summary line**, never a piped exit code — and never `$?` after a
  pipe (that is `tail`'s status).
- ⭑ **The codemod is idempotent:** apply it twice; the second run changes nothing.
- ⭑ **A `Rejected` actually says so.** Drive a dial whose answering pid ≠ minter pid if reachable; if
  it is not reachable from userland, **say so** — the crash-surface matrix lists `ConnectOutcome::Rejected`
  as UNREACHABLE, so an unverifiable arm is an honest outcome, not a gap to paper over.
- ⭑ **A `Refused` still reads correctly** — that one IS reachable
  (`probe-crash-surface-connect-refused.wat`) and must still name the site.
- ⭑ **Happy path unchanged:** the record invocation → `distinct=8000;dup=0`, every counter identical.
- ⚠ Report the **comment** pass separately: the codemod walks forms, so prose still saying "peer is
  dead" is a manual pass (trap-door 4).

## Shape to copy

`wat-scripts/fixes/add-malformed-arm.wat` — a recorded migration that adds arms to a match over an
outcome enum, which is structurally this job. And `wat/service.wat:4139`+ for the helper's shape.
