---
name: command-cards
description: Add a German reference card whenever player-facing Rust or SDK surface is introduced
trigger: always_on
---

# Befehlskarten

Treat `curriculum/src/lib.rs` as the source of truth for player-visible command cards and their tutorial unlocks. Add a card whenever tutorial examples or public SDK guidance introduce a Rust macro, syntax feature, standard-library method/type, SDK function or method, or readable SDK field.

## Add each card

Add a unique `Command` entry to `COMMANDS` with a stable key, its `box_step`, icon, category, title, and German `summary`, `what`, `how`, and `example` text. Use one card per readable struct field. Exclude internal fields that players cannot access.

## Wire it to the tutorial

- Put the key in exactly one step's `unlocked_keys`; keep the card's `box_step` equal to that step index.
- Keep `LOOT_BOX_STEPS`, `is_loot_box_step`, the `STEPS` catalog, and the final Free Play step consistent.
- Let `client-web` derive mission presentation from `curriculum::STEPS`; do not maintain a duplicate step catalog there.
- Update the curriculum, server, and client catalog tests when card contents or lootbox progression changes. Add progression tests when a new lootbox affects tutorial advancement.

## Check the example

Verify every name, type, and behavior in the card against the current `space_bot_sdk` API. Keep examples short, syntactically valid Rust, and consistent with the step at which the card is unlocked.
