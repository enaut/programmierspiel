---
name: grilling
description: Grill the user relentlessly about a plan, decision, or idea. Use when the user wants to stress-test their thinking, or uses any 'grill' trigger phrases.
---

Interview the user relentlessly until you reach a shared understanding. Map the topic as a **design tree**: every decision branches into the decisions that depend on it.

## Interview rounds

Work the tree in rounds. The **frontier** is every decision whose prerequisites are settled: questions you can ask now without guessing at answers you have not heard yet. Ask the whole frontier in one round, then wait for the user's response before recomputing it.

Use the Zed thread's native question interaction when one is available. Otherwise, ask the questions in a normal thread message and continue when the user replies. Keep each round together in one interaction:

- Give enough context to make each choice clear, including relevant trade-offs and project file links.
- Put your recommendation first and label it `(Recommended)`. Phrase choices as direct answers, and leave room for a write-in or alternative.
- Say when multiple choices can be selected if the interaction supports that format.
- Ask simple confirmations plainly rather than turning them into a formal choice list.
- Treat unanswered or ambiguous choices as open; do not silently decide for the user.

A question that depends on another open question belongs in a later round. Recompute the frontier after every response. If the user answers only part of a round, resolve what they answered and ask the remaining questions without repeating settled ones.

Finding **facts** is your job, never the user's. When a question depends on the filesystem, codebase, or other available tools, investigate it or delegate the lookup. Ask independent questions while lookups run, and defer only questions that depend on those results.

The interview is complete when every decision needed for the requested outcome is settled, or any remaining uncertainty is explicitly named and accepted as unresolved. Do not keep probing branches that cannot affect the outcome.

## Plan and proceed

When the agreed outcome calls for implementation or other concrete work, turn the decisions into an actionable plan. Use the Zed thread's native plan capability when available; otherwise present the plan in the thread. Include the goal, the ordered steps, relevant files or systems, and how you will validate the result. Call out unresolved assumptions.

Use the thread's native approval or proceed interaction when available. Otherwise ask plainly in chat whether the user wants you to proceed, and wait for an affirmative response before starting the planned work. An explicit instruction from the user to carry out the work after the interview already counts as approval.

After approval, carry out the plan in the same thread using the available tools, validate the result, and summarize what changed. If new information changes a settled decision, pause, revisit the affected branch with the user, and update the plan before continuing. For a design-only discussion, summarize the agreed direction and next steps without beginning implementation unless the user asks you to.

Use ordinary Zed thread interactions and capabilities; do not rely on Antigravity-only question or artifact APIs.
