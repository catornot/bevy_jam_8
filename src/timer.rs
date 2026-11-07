use std::time::Duration;

use bevy::prelude::*;

pub fn timer_plugin(app: &mut App) {
    app.register_type::<TriggerTimer>()
        .add_systems(Update, tick_timer);
}

#[derive(Component, Reflect)]
pub struct TriggerTimer(Timer);

impl TriggerTimer {
    pub fn new(secs: f32) -> Self where {
        Self(Timer::new(Duration::from_secs_f32(secs), TimerMode::Once))
    }
}

#[derive(Event)]
pub struct TimerTimeout(pub Entity);

fn tick_timer(
    mut timers: Query<(&mut TriggerTimer, Entity)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (mut timer, ent) in &mut timers {
        timer.0.tick(time.delta());

        if timer.0.is_finished() {
            commands.trigger(TimerTimeout(ent));
        }
    }
}
