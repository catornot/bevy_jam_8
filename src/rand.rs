use std::{
    ops::{Deref, DerefMut},
    time::{SystemTime, UNIX_EPOCH},
};

use bevy::prelude::*;
use rand::{SeedableRng, rngs::Xoshiro128PlusPlus};

#[derive(Resource)]
pub struct Rand {
    algo: Xoshiro128PlusPlus,
}

pub fn rand_plugin(app: &mut App) {
    app.insert_resource(Rand {
        algo: Xoshiro128PlusPlus::from_seed(
            (SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as u128)
                .to_ne_bytes(),
        ),
    });
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
