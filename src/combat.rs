use crate::player::KnockbackScale;
use bevy_ecs::query::QueryData;
use rand::Rng;
use valence::entity::EntityStatuses;
use valence::math::Vec3Swizzles;
use valence::prelude::*;
use valence::protocol::packets::play::ExperienceBarUpdateS2c;
use valence::protocol::sound::SoundCategory;
use valence::protocol::WritePacket;
use valence::protocol::{Sound, VarInt};

pub const SPAWN_Y: i32 = 64;
pub const ARENA_RADIUS: i32 = 32;

/// Attached to every client.
#[derive(Component, Default)]
pub struct CombatState {
    /// The tick the client was last attacked.
    last_attacked_tick: i64,
    has_bonus_knockback: bool,
}

#[derive(QueryData)]
#[query_data(mutable)]
pub struct CombatQuery {
    client: &'static mut Client,
    pos: &'static Position,
    state: &'static mut CombatState,
    statuses: &'static mut EntityStatuses,
    knockback_scale: &'static mut KnockbackScale,
}

pub fn handle_combat_events(
    server: Res<Server>,
    mut clients: Query<CombatQuery>,
    mut sprinting: EventReader<SprintEvent>,
    mut interact_entity: EventReader<InteractEntityEvent>,
) {
    for &SprintEvent { client, state } in sprinting.read() {
        if let Ok(mut client) = clients.get_mut(client) {
            client.state.has_bonus_knockback = state == SprintState::Start;
        }
    }

    for &InteractEntityEvent {
        client: attacker_client,
        entity: victim_client,
        ..
    } in interact_entity.read()
    {
        println!("InteractEntityEvent read!");
        let Ok([mut attacker, mut victim]) = clients.get_many_mut([attacker_client, victim_client])
        else {
            // Victim or attacker does not exist, or the attacker is attacking itself.
            continue;
        };

        if server.current_tick() - victim.state.last_attacked_tick < 10 {
            // Victim is still on attack cooldown.
            continue;
        }

        victim.state.last_attacked_tick = server.current_tick();

        let victim_pos = victim.pos.0.xz();
        let attacker_pos = attacker.pos.0.xz();

        let dir = (victim_pos - attacker_pos).normalize().as_vec2();

        let mut rng = rand::thread_rng();

        let knockback_xz = if attacker.state.has_bonus_knockback {
            victim.knockback_scale.0 += rng.gen_range(0.3..=0.4);
            12.0
        } else {
            victim.knockback_scale.0 += rng.gen_range(0.2..=0.3);
            8.0
        };

        let knockback_y = if attacker.state.has_bonus_knockback {
            8.432
        } else {
            6.432
        };

        victim.client.set_velocity([
            dir.x * knockback_xz * victim.knockback_scale.0,
            knockback_y,
            dir.y * knockback_xz * victim.knockback_scale.0,
        ]);

        victim.client.write_packet(&ExperienceBarUpdateS2c {
            bar: 1.0,
            level: VarInt(((victim.knockback_scale.0 * 50.0) as i32) - 50),
            total_xp: VarInt(0),
        });

        victim.client.play_sound(
            Sound::EntityPlayerHurt,
            SoundCategory::Player,
            victim.pos.0,
            1.0,
            1.0,
        );

        attacker.client.play_sound(
            Sound::EntityPlayerHurt,
            SoundCategory::Player,
            victim.pos.0,
            1.0,
            1.0,
        );

        attacker.state.has_bonus_knockback = false;
        victim.client.trigger_status(EntityStatus::PlayAttackSound);

        victim.statuses.trigger(EntityStatus::PlayAttackSound);
    }
}

pub fn teleport_oob_clients(
    mut clients: Query<(&mut Client, &mut KnockbackScale, &mut Position), With<Client>>,
) {
    for (mut client, mut knockback_scale, mut pos) in &mut clients {
        if pos.0.y < 0.0 {
            client.play_sound(
                Sound::EntityPlayerLevelup,
                SoundCategory::Player,
                pos.0,
                1.0,
                1.0,
            );
            knockback_scale.0 = 1.0;
            pos.set([0.0, f64::from(SPAWN_Y), 0.0]);
            /*
            client.write_packet(&ExperienceBarUpdateS2c {
                bar: 0.0,
                level: VarInt(0),
                total_xp: VarInt(0),
            });
            */
        }
    }
}
