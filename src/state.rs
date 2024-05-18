use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, KeyDeserialize, Key, Map, PrimaryKey};

#[cosmwasm_schema::cw_serde]
pub struct Config {
    pub native_denom: String,
    pub task_creation_amount: Uint128,
    pub refill_threshold: Uint128,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const COUNT: Item<i32> = Item::new("count");
pub const DEFAULT_ID_MAP: Map<Addr, String> = Map::new("default_id_map");
pub const NEXT_ID: Item<TaskId> = Item::new("next_id");
pub const TASK_LIST: Map<TaskId, TaskEntry> = Map::new("task_list");

#[cosmwasm_schema::cw_serde]
pub struct TaskEntry {
    pub frequency: String,
    pub domain_name: String,
}

#[cosmwasm_schema::cw_serde]
#[derive(Copy, Default)]
pub struct TaskId(pub u64);

impl TaskId {
    pub fn next_id(self) -> Self {
        Self(self.0 + 1)
    }
}

impl<'a> PrimaryKey<'a> for TaskId {
    type Prefix = ();

    type SubPrefix = ();

    type Suffix = Self;

    type SuperSuffix = Self;

    fn key(&self) -> Vec<Key> {
        self.0.key()
    }
}

impl KeyDeserialize for TaskId {
    type Output = u64;

    fn from_vec(value: Vec<u8>) -> cosmwasm_std::StdResult<Self::Output> {
        u64::from_vec(value)
    }
}

// Convert it to croncat tag
impl From<TaskId> for String {
    fn from(TaskId(id): TaskId) -> Self {
        format!("task_{id}")
    }
}