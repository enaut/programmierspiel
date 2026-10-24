---
name: setup-matt-pocock-skills
description: "Configure this repo for the engineering skills: set up its issue tracker, triage label vocabulary, and domain doc layout. Run once before first use of the other engineering skills."
disable-model-invocation: true
---

# Setup Matt Pocock's Skills

Scaffold the per-repo configuration that the engineering skills assume:

- **Issue tracker**: where issues live (GitHub by default; local markdown is also supported out of the box)
- **Triage labels**: the strings used for the five canonical triage roles
- **Domain docs**: where `GLOSSARY.md` and ADRs live, and the consumer rules for reading them

This is a prompt-driven skill, not a deterministic script. Explore, present what you found, confirm with the user, then write.

## Process

### 1. Explore

Look at the current repo to understand its starting state. Read whatever exists; don't assume:

- `git remote -v` and `.git/config`: is this a GitHub repo? Which one?
- `AGENTS.md` and `GEMINI.md` at the repo root: does either exist? Is there already an `## Agent skills` section in either?
- `GLOSSARY.md` and `GLOSSARY-MAP.md` at the repo root
- `docs/adr/` and any `src/*/docs/adr/` directories
- `docs/agents/`: does this skill's prior output already exist?
- `.scratch/`: a sign that a local-markdown issue tracker convention is already in use
- Is the `triage` skill installed? (a `triage` skill folder alongside this one, or `triage` in your available skills.) This decides whether Section B runs at all.
- Monorepo signals: a `pnpm-workspace.yaml`, a `workspaces` field in `package.json`, or a populated `packages/*` with its own `src/`. These are present only in a genuinely large multi-package repo; their absence means single-context, which is almost every repo.

### 2. Present findings and ask

Summarise what's present and what's missing in regular chat. Then prompt each decision section in order using the `ask_question` tool.

Skip any section entirely when exploration already settled it (e.g. Section B when `triage` isn't installed, Section C when there's no monorepo).

**Section A: Issue tracker.**

Prompt the user with `ask_question`:
- **Question**: "Where are issues tracked for this repository?"
- **Options**:
  - List the detected repository host first with `(Recommended)` (e.g. `"(Recommended) GitHub Issues (via gh CLI)"` or `"(Recommended) GitLab Issues (via glab CLI)"`).
  - Remaining standard options: `"GitHub Issues (via gh CLI)"`, `"GitLab Issues (via glab CLI)"`, `"Local markdown files (.scratch/<feature>/)"`.
  - Do not add an "Other" option (the UI modal provides write-in input automatically).

Record the choice in `docs/agents/issue-tracker.md`. The GitHub and GitLab templates carry a "PRs as a request surface" flag, defaulted **off**. Leave it off and don't raise it: a user who wants external PRs in the triage queue can flip the flag in the file later.

**Section B: Triage label vocabulary.** Skip this section entirely if the `triage` skill isn't installed (exploration told you), since an uninstalled skill needs no labels.

If installed, prompt the user with `ask_question`:
- **Question**: "Do you want to use the canonical triage label vocabulary?"
- **Options**:
  - `"(Recommended) Yes, keep canonical labels (needs-triage, needs-info, ready-for-agent, ready-for-human, wontfix)"`
  - `"No, customize or map to existing labels on this tracker"`

On recommended selection, write them as-is. Only if customized, collect the overrides so `triage` applies existing labels instead of creating duplicates.

**Section C: Domain docs.** Default to **single-context** (one `GLOSSARY.md` + `docs/adr/` at the repo root). This fits almost every repo; write it without asking.

Offer **multi-context** via `ask_question` only when exploration found monorepo signals:
- **Question**: "Which domain documentation layout do you want to use?"
- **Options**:
  - `"(Recommended) Multi-context (root GLOSSARY-MAP.md pointing to per-package GLOSSARY.md files)"`
  - `"Single-context (single root GLOSSARY.md and docs/adr/)"`

### 3. Confirm and edit

Show the user a draft of:

- The `## Agent skills` block to add to whichever of `AGENTS.md` / `GEMINI.md` is being edited (see step 4 for selection rules)
- The contents of `docs/agents/issue-tracker.md`, `docs/agents/domain.md`, and `docs/agents/triage-labels.md` (the last only when `triage` is installed)

Let them edit before writing.

### 4. Write

**Pick the file to edit:**

- If `AGENTS.md` exists, edit it.
- Else if `GEMINI.md` exists, edit it.
- If neither exists, default to creating `AGENTS.md` (standard cross-agent instruction file).

If an `## Agent skills` block already exists in the chosen file, update its contents in-place rather than appending a duplicate. Don't overwrite user edits to the surrounding sections.

The block:

```markdown
## Agent skills

### Issue tracker

[one-line summary of where issues are tracked]. See `docs/agents/issue-tracker.md`.

### Triage labels

[one-line summary of the label vocabulary]. See `docs/agents/triage-labels.md`.

### Domain docs

[one-line summary of layout: "single-context" or "multi-context"]. See `docs/agents/domain.md`.
```

Include the `### Triage labels` sub-block, and write `docs/agents/triage-labels.md`, only when `triage` is installed and Section B ran. When it isn't, both are omitted.

When Section B ran on GitHub or GitLab, create each configured label the tracker lacks (`gh label create` / `glab label create`).

Then write the docs files using the seed templates in this skill folder as a starting point:

- [references/issue-tracker-github.md](./references/issue-tracker-github.md): GitHub issue tracker
- [references/issue-tracker-gitlab.md](./references/issue-tracker-gitlab.md): GitLab issue tracker
- [references/issue-tracker-local.md](./references/issue-tracker-local.md): local-markdown issue tracker
- [references/triage-labels.md](./references/triage-labels.md): label mapping (only if `triage` is installed)
- [references/domain.md](./references/domain.md): domain doc consumer rules + layout

For "other" issue trackers, write `docs/agents/issue-tracker.md` from scratch using the user's description.

### 5. Done

Tell the user the setup is complete and which engineering skills will now read from these files. Mention they can edit `docs/agents/*.md` directly later; re-running this skill is only necessary if they want to switch issue trackers or restart from scratch.
