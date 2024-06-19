pub const LOCAL_PLAYER: usize = 0x0017E0A8;

pub const ENTITY_LIST: usize = 0x00591FCC;

pub const PLAYER_COUNT: usize = 0x0005C434;

/// Value type: i32
pub const OFFS_PLAYER_COUNT: [usize; 1] = [0x0];

/// Value type: Vector3
pub const OFFS_ENTITY_HEAD_POSITION: usize = 0x4;

/// Value type: ASCII text (15 chars including null)
pub const OFFS_ENTITY_NAME: usize = 0x205;

/// Value type: bool
pub const OFFS_ENTITY_TEAM: usize = 0x30C;

/// Value type: Vector2
pub const OFFS_ENTITY_VIEW_ANGLE: usize = 0x34;

/// Value type: bool
pub const OFFS_ENTITY_IS_ALIVE: usize = 0x104;

/// Value type: i32
pub const OFFS_ENTITY_HEALTH: usize = 0xEC;

/// Value type: i32
pub const OFFS_ENTITY_AMMO: usize = 0x140;
