use std::ffi::c_void;

use memwar::mem::{Allocation, Vector2, Vector3};

use crate::pointers;

#[derive(Debug)]
pub struct Entity {
    p_base: *mut c_void,
    health: i32,
    armor: i32,
    ammo: i32,
    is_alive: bool,
    is_blue_team: bool,
    name: [u8; 15],
    head_position: Vector3,
    view_angles: Vector2,
}

impl Entity {
    pub unsafe fn aim_at(&self, dest: &Entity, alloc: &Allocation) -> Result<(), u32> {
        let view_angles = self.calc_view_angles(dest);

        alloc.write_f32(
            self.p_base.add(pointers::OFFS_ENTITY_VIEW_ANGLE),
            view_angles.0,
        )?;
        alloc.write_f32(
            self.p_base.add(pointers::OFFS_ENTITY_VIEW_ANGLE + 4),
            view_angles.1,
        )?;
        Ok(())
    }

    pub fn calc_view_angles(&self, dest: &Entity) -> Vector2 {
        let delta_x = dest.head_position.0 - self.head_position.0;
        let delta_y = dest.head_position.1 - self.head_position.1;
        let delta_z = dest.head_position.2 - self.head_position.2;

        let yaw = delta_y.atan2(delta_x).to_degrees() + 90.0;
        let angle = (delta_z / delta_y).atan();
        let mut pitch = angle.to_degrees();
        
        if angle >= 1.0 {
            pitch -= 90.0;
        }
        if angle <= -1.0 {
            pitch += 90.0;
        }
        Vector2(yaw, pitch)
    }

    pub fn calc_distance(&self, dest: &Entity) -> f32 {
        (dest.head_position.0 - self.head_position.0).powf(2f32)
            + (dest.head_position.1 - self.head_position.1).powf(2f32)
                .sqrt()
    }

    pub unsafe fn read_local_player(alloc: &Allocation) -> Result<Self, u32> {
        let p_local_player = alloc.read_u32(alloc.inner().add(pointers::LOCAL_PLAYER))?;
        Self::read_from(p_local_player as _, alloc)
    }

    unsafe fn read_from(p_entity: *mut c_void, alloc: &Allocation) -> Result<Self, u32> {
        let health = alloc.read_i32(p_entity.add(pointers::OFFS_ENTITY_HEALTH))?;
        let armor = alloc.read_i32(p_entity.add(pointers::OFFS_ENTITY_AMMO))?;
        let ammo = alloc.read_i32(p_entity.add(pointers::OFFS_ENTITY_AMMO))?;

        let is_alive = alloc.read_u8(p_entity.add(pointers::OFFS_ENTITY_IS_ALIVE))? > 0;
        let is_blue_team = alloc.read_u8(p_entity.add(pointers::OFFS_ENTITY_TEAM))? > 0;

        let name: [u8; 15] = alloc.read_const(p_entity.add(pointers::OFFS_ENTITY_NAME))?;

        let head_position =
            Vector3::read_from(p_entity.add(pointers::OFFS_ENTITY_HEAD_POSITION), alloc)?;

        let view_angles =
            Vector2::read_from(p_entity.add(pointers::OFFS_ENTITY_VIEW_ANGLE), alloc)?;

        Ok(Self {
            p_base: p_entity,
            health,
            armor,
            ammo,
            is_alive,
            is_blue_team,
            name,
            head_position,
            view_angles,
        })
    }

    pub unsafe fn from_list(alloc: &Allocation) -> Result<Vec<Self>, u32> {
        let p_player_count = alloc
            .deref_chain_with_base(pointers::PLAYER_COUNT as _, pointers::OFFS_PLAYER_COUNT)?;

        let player_count = alloc.read_i32(p_player_count)?;
        let player_count = player_count as usize;

        let mut entities = vec![];

        // The local player is stored elsewhere.
        for i in 0..player_count - 1 {
            let p_entity_list = alloc.read_u32(pointers::ENTITY_LIST as _)? as usize;
            let p_entity = alloc.read_u32((p_entity_list + i * 0x4) as _)?;
            entities.push(Self::read_from(p_entity as _, alloc)?)
        }
        Ok(entities)
    }

    pub const fn health(&self) -> i32 {
        self.health
    }

    pub const fn armor(&self) -> i32 {
        self.armor
    }

    pub const fn ammo(&self) -> i32 {
        self.ammo
    }

    pub const fn is_alive(&self) -> bool {
        self.is_alive
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
