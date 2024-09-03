use valence::entity::entity::Flags;
use valence::{entity::zombie::ZombieEntityBundle, prelude::*};

#[derive(Component, PartialEq)]
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

#[derive(Component, Copy, Clone, PartialEq)]
pub enum PlayerCharacter {
    None,
    Zombie,
    IronGolem,
    Creeper,
}

#[derive(Component)]
pub struct Character;

#[derive(Component)]
pub struct CharacterId(u32); // same as player entity index

pub fn spawn_player_character(
    mut commands: Commands,
    mut players: Query<
        (
            Entity,
            &PlayerCharacter,
            &PlayerState,
            &Position,
            &mut Flags,
        ),
        Changed<PlayerCharacter>,
    >,
    layers: Query<Entity, (With<ChunkLayer>, With<EntityLayer>)>,
) {
    let layer = layers.single();

    for (player_entity, player_character, player_state, position, mut player_flags) in
        players.iter_mut()
    {
        if *player_state == PlayerState::Playing || *player_state == PlayerState::Spawn {
            player_flags.set_invisible(true);
            match player_character {
                PlayerCharacter::None => {
                    player_flags.set_invisible(false);
                }
                PlayerCharacter::Zombie => {
                    commands
                        .spawn(ZombieEntityBundle {
                            position: *position,
                            layer: EntityLayerId(layer),
                            ..Default::default()
                        })
                        .insert(CharacterId(player_entity.index()))
                        .insert(Character);
                }
                _ => {}
            }
        }
    }
}

pub fn character_follows_player(
    mut characters: Query<
        (&CharacterId, &mut Position, &mut Look, &mut HeadYaw),
        (With<Character>, Without<PlayerState>),
    >,
    players: Query<(Entity, &Position, &Look, &HeadYaw), Without<Character>>,
) {
    for (character_id, mut character_pos, mut character_look, mut character_head_yaw) in
        characters.iter_mut()
    {
        for (player_entity, player_pos, player_look, player_head_yaw) in players.iter() {
            if character_id.0 == player_entity.index() {
                *character_pos = *player_pos;
                *character_look = *player_look;
                *character_head_yaw = *player_head_yaw;
            }
        }
    }
}
