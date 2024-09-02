use valence::prelude::*;

#[derive(Component)]
pub enum PlayerState {
    Spawn,
    Playing,
    Spectating,
}

#[derive(Component)]
pub struct KnockbackScale(pub f32);

impl Default for KnockbackScale {
    fn default() -> Self {
        Self(1.0)
    }
}

#[derive(Component)]
pub enum PlayerCharacter {
    Zombie,
    IronGolem,
    Creeper,
}
