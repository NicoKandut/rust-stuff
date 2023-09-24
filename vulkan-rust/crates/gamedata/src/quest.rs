#[allow(unused)]
pub struct Item {
    id: u64,
}

#[allow(unused)]
pub struct Entity {
    id: u64,
}

#[allow(unused)]
pub enum QuestArea {
    Circle {
        center: (i32, i32, i32),
        radius: usize,
    },
    Structure {
        id: u64,
    },
}

#[allow(unused)]
pub enum QuestType {
    Collect {
        item: Item,
        amount: usize,
    },
    Sell {
        item: Item,
        amount: usize,
    },
    Investigate, // ???
    KillType {
        entity: Entity,
        amount: usize,
    },
    KillSpecific {
        entity: Entity,
        amount: usize,
    },
    Escort {
        entity: Entity,
        checkpoints: Vec<(i32, i32, i32)>,
    },
}
