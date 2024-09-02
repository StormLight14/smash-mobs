use combat::*;
use player::{KnockbackScale, PlayerCharacter, PlayerState};
use portals::{CharacterPortal, CharacterPortalBundle};
use rand::Rng;
use valence::entity::block_display::BlockDisplayEntityBundle;
use valence::entity::display::Scale;
use valence::entity::text_display::{Text as TextDisplayText, TextDisplayEntityBundle};
use valence::prelude::*;

mod combat;
mod player;
mod portals;

#[derive(Resource, Default)]
struct Globals {
    game_running: bool,
    online_players: u8,
}

fn main() {
    App::new()
        .insert_resource(NetworkSettings {
            connection_mode: ConnectionMode::Offline,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, (setup, create_portals.after(setup)))
        .add_systems(EventLoopUpdate, handle_combat_events)
        .add_systems(
            Update,
            (init_clients, client_disconnect, teleport_oob_clients),
        )
        .insert_resource::<Globals>(Globals::default())
        .run();
}

fn setup(
    mut commands: Commands,
    server: Res<Server>,
    biomes: Res<BiomeRegistry>,
    dimensions: Res<DimensionTypeRegistry>,
) {
    let mut spawn_layer = LayerBundle::new(ident!("overworld"), &dimensions, &biomes, &server);

    for z in -5..5 {
        for x in -5..5 {
            spawn_layer.chunk.insert_chunk([x, z], UnloadedChunk::new());
        }
    }

    let mut rng = rand::thread_rng();

    for z in -ARENA_RADIUS..ARENA_RADIUS {
        for x in -ARENA_RADIUS..ARENA_RADIUS {
            let dist = f64::hypot(f64::from(x), f64::from(z)) / f64::from(ARENA_RADIUS);

            if dist > 1.0 {
                continue;
            }

            let block = if rng.gen::<f64>() < dist {
                BlockState::STONE
            } else {
                BlockState::DEEPSLATE
            };

            for y in 0..SPAWN_Y {
                spawn_layer.chunk.set_block([x, y, z], block);
            }
        }
    }

    let layer_id = commands.spawn(spawn_layer).id();

    commands.spawn(BlockDisplayEntityBundle {
        block_display_block_state: valence::entity::block_display::BlockState(
            BlockState::GRASS_BLOCK,
        ),
        position: Position::new([5.0, 66.0, 5.0]),
        display_scale: Scale(Vec3::new(3.0, 3.0, 3.0)),
        layer: EntityLayerId(layer_id),
        ..Default::default()
    });
}

fn create_portals(
    mut commands: Commands,
    layers: Query<Entity, (With<ChunkLayer>, With<EntityLayer>)>,
) {
    let layer = layers.single();

    commands.spawn(CharacterPortalBundle {
        character_portal: CharacterPortal,
        text_display_entity_bundle: TextDisplayEntityBundle {
            text_display_text: TextDisplayText("Zombie".into_text().color(Color::RED)),
            position: Position::new([-5.0, 66.0, -5.0]),
            layer: EntityLayerId(layer),
            ..Default::default()
        },
        to_character: player::PlayerCharacter::Zombie,
    });
}

fn init_clients(
    mut clients: Query<
        (
            Entity,
            &mut Client,
            &mut EntityLayerId,
            &mut VisibleChunkLayer,
            &mut VisibleEntityLayers,
            &mut Position,
            &mut GameMode,
        ),
        Added<Client>,
    >,
    mut commands: Commands,
    mut globals: ResMut<Globals>,
    layers: Query<Entity, (With<ChunkLayer>, With<EntityLayer>)>,
) {
    for (
        client_entity,
        mut client,
        mut layer_id,
        mut visible_chunk_layer,
        mut visible_entity_layers,
        mut pos,
        mut game_mode,
    ) in &mut clients
    {
        let layer = layers.single();

        layer_id.0 = layer;
        visible_chunk_layer.0 = layer;
        visible_entity_layers.0.insert(layer);
        pos.set([0.5, 65.0, 0.5]);
        *game_mode = GameMode::Survival;
        commands
            .entity(client_entity)
            .insert(PlayerState::Spawn)
            .insert(CombatState::default())
            .insert(KnockbackScale::default());
        globals.online_players += 1;

        client.send_chat_message(
            "Welcome to Super Smash Mobs! Current players online: "
                .into_text()
                .color(Color::rgb(100, 255, 100))
                + globals
                    .online_players
                    .into_text()
                    .color(Color::rgb(125, 180, 255)),
        );

        if globals.game_running {
            client.send_chat_message(
                "The game is currently running. You may spectate it while you wait!"
                    .into_text()
                    .color(Color::rgb(255, 120, 120)),
            );
        } else {
            client.send_chat_message(
                "The game has not started! Join the queue before it starts."
                    .into_text()
                    .color(Color::rgb(255, 255, 120)),
            );
        }
    }
}

// exists as despawn_disconnected_clients, but I want to change globals as well.
fn client_disconnect(
    mut commands: Commands,
    mut disconnected_clients: RemovedComponents<Client>,
    mut globals: ResMut<Globals>,
) {
    for entity in disconnected_clients.read() {
        globals.online_players -= 1;
        if let Some(mut entity) = commands.get_entity(entity) {
            entity.insert(Despawned);
        }
    }
}
