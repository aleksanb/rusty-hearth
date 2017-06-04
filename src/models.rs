use std::collections::{HashMap, HashSet};

#[derive(Debug, Default, Serialize, Clone)]
pub struct Player {
    pub deck: HashMap<String, HashSet<String>>,
}

#[derive(Debug, Default, Serialize, Clone)]
pub struct GameState {
    pub players: HashMap<String, Player>,
}

impl GameState {
    pub fn handle_play(&mut self, play: Play) -> bool {
        self.players
            .entry(play.player)
            .or_insert(Player::default())
            .deck
            .entry(play.card_id)
            .or_insert(HashSet::default())
            .insert(play.id)
    }
}

#[derive(Debug)]
pub struct Play {
    pub id: String,
    pub card_id: String,
    pub player: String,
}
