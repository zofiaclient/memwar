use std::ffi::c_void;

use memwar::mem::{Allocation, Vector2, Vector3};

use crate::pointers;

#[derive(Debug)]
pub struct Entity {
    health: i32,
    is_blue_team: bool,
    name: [u8; 15],
    head_position: Vector3,
    view_angles: Vector2,
}

impl Entity {
    pub fn calc_distance(&self, dest: &Entity) -> f32 {
        (dest.head_position.0 - self.head_position.0).powf(2.0)
            + (dest.head_position.1 - self.head_position.1).powf(2.0)
            + (dest.head_position.2 - self.head_position.2)
                .powf(2.0)
                .sqrt()
    }

    unsafe fn read_from(p_entity: *mut c_void, alloc: &Allocation) -> Result<Self, String> {
        let health = alloc
            .read_i32(p_entity.add(pointers::OFFS_ENTITY_HEALTH))
            .map_err(|e| format!("({e}) Failed to read entity health"))?;

        let is_blue_team = alloc
            .read_u8(p_entity.add(pointers::OFFS_ENTITY_TEAM))
            .map_err(|e| format!("({e}) Failed to read entity team"))?
            > 0;

        let name: [u8; 15] = alloc
            .read_const(p_entity.add(pointers::OFFS_ENTITY_NAME))
            .map_err(|e| format!("({e}) Failed to read entity name"))?;

        let head_position =
            Vector3::read_from(p_entity.add(pointers::OFFS_ENTITY_HEAD_POSITION), alloc)
                .map_err(|e| format!("({e}) Failed to read entity head position"))?;

        let view_angles = Vector2::read_from(p_entity.add(pointers::OFFS_ENTITY_VIEW_ANGLE), alloc)
            .map_err(|e| format!("({e}) Failed to read entity view angles"))?;

        Ok(Self {
            health,
            is_blue_team,
            name,
            head_position,
            view_angles,
        })
    }

    pub unsafe fn from_list(alloc: &Allocation) -> Result<Vec<Self>, String> {
        let p_player_count = alloc
            .deref_chain_with_base(pointers::PLAYER_COUNT as _, pointers::OFFS_PLAYER_COUNT)
            .map_err(|e| format!("({e}) Failed to dereference player count"))?;

        let player_count = alloc
            .read_i32(p_player_count)
            .map_err(|e| format!("({e}) Failed to read player count"))?;

        if player_count <= 0 {
            return Err(format!("Invalid player count ({player_count})"));
        }

        let mut entities = vec![];

        // The local player is stored elsewhere.
        for i in 0..player_count as usize - 1 {
            let p_entity_list = alloc
                .read_u32(pointers::ENTITY_LIST as _)
                .map_err(|e| format!("({e}) Failed to read pointer to entity list"))?
                as usize;

            let p_entity = alloc
                .read_u32((p_entity_list + i * 0x4) as _)
                .map_err(|e| format!("({e}) Failed to read entity pointer"))?;

            entities.push(
                Self::read_from(p_entity as _, alloc)
                    .map_err(|e| format!("Failed to read entity at index {i}: {e}"))?,
            )
        }
        Ok(entities)
    }

    pub const fn health(&self) -> i32 {
        self.health
    }

    pub const fn is_alive(&self) -> bool {
        self.health > 0
    }

    pub const fn head_position(&self) -> &Vector3 {
        &self.head_position
    }

    pub const fn view_angle(&self) -> &Vector2 {
        &self.view_angles
    }

    pub const fn name(&self) -> [u8; 15] {
        self.name
    }

    pub fn name_as_string(&self) -> String {
        String::from_utf8_lossy(&self.name).to_string()
    }

    pub const fn is_blue_team(&self) -> bool {
        self.is_blue_team
    }
}

pub struct LocalPlayer {
    p_base: *mut c_void,
    entity: Entity,
}

impl LocalPlayer {
    fn calc_view_angles(&self, dest: &Entity) -> Vector2 {
        let delta_x = dest.head_position.0 - self.entity.head_position.0;
        let delta_y = dest.head_position.1 - self.entity.head_position.1;
        let delta_z = dest.head_position.2 - self.entity.head_position.2;

        let magn = (delta_x.powf(2.0) + delta_y.powf(2.0) + delta_z.powf(2.0)).sqrt();

        let yaw = delta_y.atan2(delta_x).to_degrees() + 90.0;
        let pitch = (delta_z / magn).tan().to_degrees();

        Vector2(yaw, pitch)
    }

    pub unsafe fn aim_at(&self, dest: &Entity, alloc: &Allocation) -> Result<(), String> {
        let view_angles = self.calc_view_angles(dest);

        alloc
            .write_f32(
                self.p_base.add(pointers::OFFS_ENTITY_VIEW_ANGLE),
                view_angles.0,
            )
            .map_err(|e| format!("({e}) Failed to write entity view angle yaw"))?;

        alloc
            .write_f32(
                self.p_base.add(pointers::OFFS_ENTITY_VIEW_ANGLE + 4),
                view_angles.1,
            )
            .map_err(|e| format!("({e}) Failed to write entity view angle pitch"))?;

        Ok(())
    }

    pub unsafe fn read_from(alloc: &Allocation) -> Result<Self, String> {
        let p_base = alloc
            .read_u32(alloc.inner().add(pointers::LOCAL_PLAYER))
            .map_err(|e| format!("({e}) Failed to read local player pointer"))?
            as _;

        Ok(Self {
            p_base,
            entity: Entity::read_from(p_base, alloc)?,
        })
    }

    pub const fn entity(&self) -> &Entity {
        &self.entity
    }
}
