#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DropRole {
    Dropper,
    Checker,
}

#[derive(Debug, Clone)]
pub enum DropState {
    Dropped,
    Hand,
}

#[derive(Debug, Clone)]
pub enum NearDeath {
    Death,
    Alive,
}

#[derive(Debug)]
pub enum DropCheck {
    Failed,
    Sucess(u64),
}

#[derive(Debug)]
pub enum DropChannel {
    StartRound,
    Rcp,
}
