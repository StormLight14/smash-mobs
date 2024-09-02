use valence::entity::text_display::TextDisplayEntityBundle;
use valence::prelude::*;

use crate::player::PlayerCharacter;

#[derive(Component)]
pub struct CharacterPortal;

#[derive(Bundle)]
pub struct CharacterPortalBundle {
    pub character_portal: CharacterPortal,
    pub text_display_entity_bundle: TextDisplayEntityBundle,
    pub to_character: PlayerCharacter,
}
