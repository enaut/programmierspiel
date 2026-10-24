---
name: grilling
description: Grill the user relentlessly about a plan, decision, or idea. Use when the user wants to stress-test their thinking, or uses any 'grill' trigger phrases.
---

Interview the user relentlessly until you reach a shared understanding. Map this as a **design tree**: every decision branches into the decisions that hang off it.

Work the tree in **rounds**. The **frontier** is every decision whose prerequisites are already settled: the questions you can ask _now_ without guessing at answers you haven't heard yet. Ask the whole frontier in one round using the `ask_question` tool. Execution blocks until the user responds via the interactive modal.

## Using `ask_question`

Batch all questions on the current frontier into a single `ask_question` call with the `questions` array:

- **Options**:
  - List your recommended answer first and prefix it with `(Recommended)`.
  - Format options as the user's direct response (e.g. `"Use WebAssembly bindings"`, not `"I will use..."`).
  - Do not add an `"Other"` option; the UI provides a write-in response by default.
  - Do not enumerate options (e.g. `"1."`, `"A."`); the UI numbers them automatically.
- **Question text**:
  - Include necessary background context, trade-offs, and file links (`[filename](file:///path/to/file)`).
  - Do not include `"Select all that apply"` in the question title/text; set `is_multi_select: true` when multiple choices apply.
- **Trivial questions**: When a question is a simple one-word confirmation rather than a structured decision, output regular text instead.

## Process

Each round the user answers reshapes the tree: settled decisions push the frontier outward and unblock questions that depended on them. Recompute the frontier and ask the next round. A question whose answer depends on another question still open in this round belongs to a _later_ round, not this one.

Finding _facts_ is your job, never the user's. When a frontier question needs a fact from the environment (filesystem, codebase, tools), look it up or dispatch a sub-agent to find it; don't ask the user for anything you could determine yourself. Don't block independent questions on running lookups: ask settled frontier questions now, and defer questions downstream of the investigation to a later round.

The session is done when the frontier is empty: every branch of the design tree visited, nothing left silently assumed. 

Summarize the agreed-upon design and write the final plan to the Antigravity artifact directory (`<appDataDir>/brain/<conversation-id>/plan.md`) using `write_file` with `ArtifactMetadata` (`RequestFeedback: true`, `UserFacing: true`, `Summary: "Agreed-upon execution plan after grilling interview"`). Setting `RequestFeedback: true` renders a "Proceed" button in the Antigravity UI for the user to approve execution.
