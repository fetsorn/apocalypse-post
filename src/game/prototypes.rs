use coord::Coord;

use ecs::*;
use game::*;
use game::data::*;

pub const ENV_TURN_OFFSET: u64 = 0;
pub const NPC_TURN_OFFSET: u64 = 1;
pub const PC_TURN_OFFSET: u64 = 2;

pub fn pc<E: EntityPopulate>(mut entity: E, position: Coord) -> E {
    entity.insert_position(position);

    entity.insert_tile(TileType::Van);

    entity.insert_tile_depth(2);
    entity.insert_collider();
    entity.insert_behaviour_state(BehaviourState::new());
    entity.insert_behaviour_type(BehaviourType::PlayerInput);
    entity.insert_turn_offset(PC_TURN_OFFSET);
    entity.insert_drawable_knowledge(DrawableKnowledge::new());
    entity.insert_vision_distance(16);
    entity.insert_door_opener();
    entity.insert_pc();
    entity.insert_turn_time(TURN_DURATION_BASE);
    entity.insert_should_render();
    entity.insert_message_log(MessageLog::new());
    entity.insert_you_see(YouSeeMessageType::Player);
    entity.insert_description(DescriptionMessageType::Player);
    entity.insert_projectile_collider();
    entity.insert_hit_points(HitPoints::new(10));
    entity.insert_bump_attackable();

    entity
}

pub fn zombie<E: EntityPopulate>(mut entity: E, position: Coord) -> E {
    entity.insert_position(position);

    entity.insert_tile(TileType::Zombie);

    entity.insert_tile_depth(2);
    entity.insert_collider();
    entity.insert_behaviour_state(BehaviourState::new());
    entity.insert_behaviour_type(BehaviourType::SimpleNpc);
    entity.insert_turn_offset(NPC_TURN_OFFSET);
    entity.insert_vision_distance(8);
    entity.insert_simple_npc_knowledge(SimpleNpcKnowledge::new());
    entity.insert_path_traverse(PathTraverse::new());
    entity.insert_turn_time(TURN_DURATION_BASE * 2);
    entity.insert_enemy();
    entity.insert_projectile_collider();
    entity.insert_hit_points(HitPoints::new(2));
    entity.insert_bump_attacker(1);

    entity
}

pub fn bullet<E: EntityPopulate>(mut entity: E, position: Coord, velocity: RealtimeVelocity) -> E {

    entity.insert_position(position);
    entity.insert_realtime_velocity(velocity);
    entity.insert_destroy_on_collision();
    entity.insert_projectile();
    entity.insert_projectile_damage(1);

    entity.insert_tile(TileType::Bullet);

    entity.insert_tile_depth(1);

    entity
}

pub fn road<E: EntityPopulate>(mut entity: E, position: Coord, rng: &GameRng) -> E {
    entity.insert_position(position);

    let rest_tiles = [
        TileType::Road1,
    ];

    let tile = *rng.select_or_select_uniform(0.95, &TileType::Road0, &rest_tiles);

    entity.insert_tile(tile);
    entity.insert_tile_depth(0);
    entity.insert_floor();

    entity
}

pub fn dirt<E: EntityPopulate>(mut entity: E, position: Coord, rng: &GameRng) -> E {
    entity.insert_position(position);

    let rest_tiles = [
        TileType::Dirt1,
    ];

    let tile = *rng.select_or_select_uniform(0.95, &TileType::Dirt0, &rest_tiles);

    entity.insert_tile(tile);
    entity.insert_tile_depth(0);
    entity.insert_floor();

    entity
}

pub fn acid<E: EntityPopulate>(mut entity: E, position: Coord, rng: &GameRng) -> E {
    entity.insert_position(position);

    let rest_tiles = [
        TileType::Acid1,
    ];

    let tile = *rng.select_or_select_uniform(0.95, &TileType::Acid0, &rest_tiles);

    entity.insert_tile(tile);
    entity.insert_tile_depth(0);
    entity.insert_floor();

    entity
}

pub fn wreck<E: EntityPopulate>(mut entity: E, position: Coord, rng: &GameRng) -> E {
    entity.insert_position(position);

    let tiles = [
        TileType::Wreck0,
        TileType::Wreck1,
        TileType::Wreck2,
    ];

    let tile = *rng.select_uniform(&tiles);

    entity.insert_tile(tile);
    entity.insert_tile_depth(0);
    entity.insert_floor();
    entity.insert_solid();

    entity
}
