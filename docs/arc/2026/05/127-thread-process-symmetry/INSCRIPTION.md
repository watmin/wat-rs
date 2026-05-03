# Arc 127 — Thread/process protocol symmetry — INSCRIPTION

**Status:** **WITHDRAWN 2026-05-01** in writing, before any implementation.
**Closure:** 2026-05-03 (paperwork).

---

## Why withdrawn

Arc 127 proposed a substrate-level rethink of threads to mirror processes — retiring user-facing channel constructors (`make-bounded-channel<T>` / `make-unbounded-channel<T>`), making channels output-only from `spawn-thread` / `fork-program` / `spawn-program` calls, and exposing three pipes by construction (`stdin: PipeWriter<I>`, `stdout: PipeReader<O>`, `stderr: PipeReader<ThreadDiedError>`).

The four questions (obvious / simple / honest / good UX) plus a re-read of `docs/ZERO-MUTEX.md` overruled the proposal:

- **Honest** lost: removing user-facing channel constructors would force every consumer through spawn boundaries, hiding the substrate's actual primitives behind the spawn API. Channels are foundational; the substrate exposes them honestly.
- **Simple** lost: forcing channel allocation through spawn-output unifies one shape but multiplies the cases (now spawn must return whatever combination of channels the consumer needs; channels-as-spawn-outputs becomes a Cartesian product of spawn shapes).

`ZERO-MUTEX.md`'s discipline already establishes channel-as-substrate-primitive; arc 127's rethink would have inverted that without compensating gain.

## Discipline preserved

Per project convention: **rejected proposals stay; sequential numbering; no v1/v2.** Arc 127's DESIGN remains the honest record of an architectural alternative considered and overruled by the four questions + the prior ZERO-MUTEX doctrine.

## References

- `docs/arc/2026/05/127-thread-process-symmetry/DESIGN.md` (the withdrawn proposal)
- `docs/ZERO-MUTEX.md` (the doctrine that overruled it)

---

**Arc 127 — closed as withdrawn.**
