# Architectural Decision Records (ADRs)

This repository records significant architectural decisions as ADRs in `docs/ADR/`. An ADR
captures *why* the code looks the way it does: the context, the options weighed, and the
consequences accepted. Code and git history show *what* changed; only an ADR keeps the
reasoning available to the next person (or agent) who wants to change it.

**You must write an ADR whenever you make, or are asked to implement, a significant
architectural decision.** Do it in the same change as the code, not afterwards.

## What counts as significant

Write an ADR when a decision is hard or costly to reverse, or constrains future work. Typical
triggers:

- Adding, replacing or removing a framework, library or runtime dependency that shapes the code
  (web framework, ORM / query layer, async runtime, auth library, serialization format).
- Data and persistence: database engine, schema conventions, migration strategy, ID format,
  soft vs. hard deletes, multi-tenancy model.
- API design: versioning, error format, pagination, authentication and authorization scheme.
- Module or layer boundaries: how `routes`, `repo`, `models`, `auth` depend on each other;
  introducing a new layer or crate; splitting into a workspace.
- Cross-cutting concerns: configuration, logging and tracing, error handling, security posture
  (password hashing, token format, CORS policy).
- Build, test, CI and deployment strategy: how tests reach the database, sqlx offline/online
  mode, container layout, release process.
- Deliberately deviating from an existing ADR or from a convention used across the codebase.

If you are unsure, assume it is significant and write the ADR — a short ADR is cheap, a lost
rationale is not.

Do **not** write ADRs for local implementation details that are easy to change: naming a
function, refactoring inside one module, fixing a bug, bumping a patch version.

## How to write one

1. Check `docs/ADR/README.md` for existing ADRs that cover the topic. If your decision changes
   one, supersede it (see below) rather than contradicting it silently.
2. Copy `docs/ADR/template.md` to `docs/ADR/NNNN-short-kebab-title.md`, where `NNNN` is the next
   free four-digit number (`0001`, `0002`, …). Never reuse or renumber.
3. Fill in every section of the template. Keep it to about one page:
   - **Context** states the problem and the forces at play, neutrally, without the answer.
   - **Considered options** lists the real alternatives, including "do nothing" when relevant,
     each with honest pros and cons.
   - **Decision** names the chosen option in one or two sentences and says why it won.
   - **Consequences** lists what becomes easier *and* what becomes harder or riskier, plus any
     follow-up work.
4. Set the status to **Proposed** and today's date. Only the user accepts a decision: change the
   status to **Accepted** when they have explicitly approved it, and tell them the ADR exists
   when you report your work.
5. Add a row to the index table in `docs/ADR/README.md`.

## Lifecycle

- Accepted ADRs are immutable records. Do not rewrite their content; fixing typos or adding a
  link is fine.
- To change a decision, write a new ADR and set the old one's status to
  `Superseded by [ADR-NNNN](NNNN-....md)`; the new one links back with `Supersedes`.
- A decision that no longer applies but has no replacement becomes **Deprecated**.
- A proposal the user turns down becomes **Rejected** — keep the file, it records why.

## Using ADRs

Before changing something in an area listed above, read the relevant ADRs and follow them. If
the user asks for something that contradicts an accepted ADR, point out the conflict and
propose superseding it instead of quietly diverging.
