---
name: tutorial-quests
description: Update or extend the beginner tutorial quests, Befehls-Ruhmeshalle cards, and lootboxes. Use when adding, changing, reviewing, or reordering a tutorial step, a command card, a lootbox, or any public command, method, field, constant, or research track in space_bot_sdk.
---

The tutorial teaches Rust to **beginners with no programming experience**. Learn the house style from steps 0–13 (the reviewed ones). Steps 14+ are not yet reviewed, so do not copy them blindly.

## Model
- Steps 0–4 are onboarding, with no code.
- From step 5 the steps alternate: **lootbox step N (odd)** → **coding step N+1 (even)**.
- Every new code element used in a coding step gets one `CommandInfo` card in the Befehlshalle, revealed in the lootbox **before** the first coding step that uses it (`box_step` in `5, 7, 9, 11, 13, 15, 17, 19, 21`).
- **Complete SDK coverage in the Befehlshalle**: Every public command, method, field, constant, and research track in `space_bot_sdk` must have a `CommandInfo` card in `COMMANDS` (`client-web/src/commands.rs`). SDK items that are not required for a specific tutorial quest get `box_step: 22` (unlocked automatically when reaching Free Play at step 22 via `free_play_commands()` in `server/src/lib.rs`). Whenever a command, method, field, or research track is added to `space_bot_sdk`, add its card and server key in the same change.
- Completion of a coding step is verified by the server, never trusted from the client.

## Steps
1. **Inventory the code elements and SDK surface.** List every identifier, keyword, macro, operator, and format string a coding step makes the learner type (`if let`, `println!`, `{:.0}`, `drone.position.x`, `&&`, ...), plus any new public API in `space_bot_sdk`. Test each tutorial element against "would someone who has never programmed be able to guess this?". Done when each tutorial element is in an earlier box or gets a card in the box right before the step, and every public `space_bot_sdk` item has a card in either a lootbox (`5..=21`) or Free Play (`box_step: 22`).
2. **Write the cards** in `client-web/src/commands.rs` (`COMMANDS`):
   - `key`: UPPER_SNAKE_CASE, unique and identical to the server key; `box_step`: the lootbox step (`5..=21`) or `22` for Free Play SDK reference cards; the order within a box is the teaching order.
   - German, informal "du". Explain every technical term on first use in plain words (Option, Block, Bedingung, Variable) and tie it to the game.
   - `summary`: one sentence. `what`: what it does and what happens in the failure case. `how`: the parts of the syntax in reading order. `example`: a short snippet that compiles in `tick`.
   - One concept per card (`if let` and `first_drone()` are separate cards).
3. **Register the keys** in all of: `LOOT_BOX_STEPS` and `loot_box_commands` (or `free_play_commands` for `box_step: 22`) in `server/src/lib.rs`, `LOOT_BOX_STEPS` and the `COMMANDS` array length in `client-web/src/commands.rs`, and **both** `expected` / `expected_free_play` test tables (server and client). Done when `cargo test -p server` and `cargo check --tests -p client-web --target wasm32-unknown-unknown` are green (client-web is a wasm app; its native tests do not compile).
4. **Write the step texts** in `get_step_info` (`client-web/src/tutorial.rs`):
   - Lootbox step: title `🎁 Befehls-Lootbox #N: Thema`, a story that says what was achieved and what is learned, and the two standard subtasks (open the Heimatbasis via the `🏛️ Heimatbasis öffnen` button, then click the glowing lootbox and collect every card). Use identical subtask wording in every box.
   - Coding step: the story gives the context, subtasks are small and verifiable, and `code_hint` uses only elements already unlocked. Scaffold progressively: placeholders (`...`) for the first use, the full line once the learner has done it themselves.
   - Keep code names out of a lootbox story before the cards have explained them, except as the topic title.
5. **Wire the server check.** The completion of the coding step needs a server-side `advance_tutorial(ctx, owner, from, to)` trigger. Any new or changed reducer requires regenerating the bindings (see the `generate_bindings` rule).
6. **Update the totals.** If the number of steps changes, update `TUTORIAL_COMPLETED_STEP`, the "/ 22" strings, the progress bar and the tests together.
7. **Verify.** Run the tests. Then read the new steps top to bottom as a beginner. Done when every code element in every `code_hint` has a card in an earlier or the same box.
