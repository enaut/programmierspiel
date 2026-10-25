use testclient::ai;

fn main() {
    println!("🚀 [COMMAND CENTER] Starte Drohnenflotten-Steuerung...");
    space_bot_sdk::run(ai::decide);
}
