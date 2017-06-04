#[derive(Debug)]
pub struct Card {
    pub card_id: String,
    pub ids: Vec<String>,
}

#[derive(Debug)]
pub struct Player {
    pub deck: Vec<Card>,
}

#[derive(Debug)]
pub struct PlayState {
    pub players: [Player; 2],
}

#[derive(Debug)]
pub struct Play {
    pub id: String,
    pub card_id: String,
    pub player: String,
}