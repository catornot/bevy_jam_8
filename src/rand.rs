use bevy::prelude::*;
use rand::rngs::Xoshiro128PlusPlus;
use std::ops::{Deref, DerefMut};

pub fn rand_plugin(app: &mut App) {
    app.insert_resource(Rand {
        algo: rand::make_rng(),
    })
    .add_systems(Update, seed_randomizer);
}

#[derive(Resource)]
pub struct Rand {
    algo: Xoshiro128PlusPlus,
}

impl Deref for Rand {
    type Target = Xoshiro128PlusPlus;

    fn deref(&self) -> &Self::Target {
        &self.algo
    }
}

impl DerefMut for Rand {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.algo
    }
}

fn seed_randomizer(mut rand: ResMut<Rand>) {
    if rand.is_changed() {
        rand.bypass_change_detection().algo = rand::make_rng();
    }
}
