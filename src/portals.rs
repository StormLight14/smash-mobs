use valence::entity::text_display::TextDisplayEntityBundle;
use valence::prelude::*;

use crate::player::{PlayerCharacter, PlayerState};

#[derive(Component)]
pub struct CharacterPortal;

#[derive(Bundle)]
pub struct CharacterPortalBundle {
    pub character_portal: CharacterPortal,
    pub text_display_entity_bundle: TextDisplayEntityBundle,
    pub to_character: PlayerCharacter,
}

pub fn check_for_players(
    mut players: Query<
        (&Position, &mut PlayerCharacter),
        (With<PlayerState>, Without<CharacterPortal>),
    >,
    portals: Query<(&Position, &PlayerCharacter), (With<CharacterPortal>, Without<PlayerState>)>,
) {
    for (player_pos, mut player_character) in players.iter_mut() {
        for (portal_pos, portal_player_character) in portals.iter() {
            if (player_pos.x - portal_pos.x).abs() <= 1.1
                && (player_pos.y - portal_pos.y).abs() <= 1.1
                && (player_pos.z - portal_pos.z).abs() <= 1.1
            {
                println!("player entered portal");
                if *player_character != *portal_player_character {
                    *player_character = *portal_player_character;
                }
            }
        }
    }
}
