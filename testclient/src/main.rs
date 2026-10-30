use space_bot_sdk::{Actions, BaseController, UpgradeTrack, World};

struct MyBase;

impl MyBase {
    fn new() -> Self {
        Self
    }
}

impl BaseController for MyBase {
    #[allow(unused_variables)]
    fn tick(&mut self, world: &World, actions: &mut Actions) {
        // Willkommen, Commander!
        // Hier programmierst du deine Flottensteuerung.
        // Folge den Missionen im Web-Dashboard.
    }
}

fn main() {
    space_bot_sdk::run(MyBase::new());
}
