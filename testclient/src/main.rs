use space_bot_sdk::{Actions, BaseController, UpgradeTrack, World};

struct MyBase;

impl MyBase {
    fn new() -> Self {
        Self
    }
}

impl BaseController for MyBase {
    fn tick(&mut self, world: &World, actions: &mut Actions) {
        // Willkommen, Commander!
        // Hier programmierst du deine Flottensteuerung.
        // Folge den Missionen im Web-Dashboard.
        for drone in world.fleet {
            if let Some(asteroid) = world.closest_asteroid_with_ore(drone.position.clone()) {
                if drone.cargo_is_full() {
                    actions.dock(drone.ship_id);
                } else {
                    actions.mine(drone.ship_id, asteroid.asteroid_id);
                }
            }
        }

        if world.fleet.len() < 5 && world.base.stored_ore >= 50 {
            actions.build_drone(1);
        }

        if world.active_riddle.is_none() {
            actions.request_research(UpgradeTrack::BaseAsteroidScanner);
        }
    }
}

fn main() {
    space_bot_sdk::run(MyBase::new());
}
