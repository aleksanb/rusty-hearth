use regex;
use models;

lazy_static! {
    static ref CARD_UPDATE_PATTERN: regex::Regex = regex::Regex::new(
        r"^.*id=(?P<id>\d*) .*cardId=(?P<card_id>[a-zA-Z0-9_]*) .*player=(?P<player>\d*)")
            .unwrap();
    static ref GAME_COMPLETE_PATTERN: regex::Regex = regex::Regex::new(
        r".*TAG_CHANGE Entity=GameEntity tag=STATE value=COMPLETE.*")
            .unwrap();
}

#[derive(Debug)]
pub enum LogEvent {
    GameComplete,
    PowerLogRecreated,
    Play(models::Play),
}

pub fn parse_log_line(line: &str) -> Option<LogEvent> {
    if GAME_COMPLETE_PATTERN.is_match(line) {
        return Some(LogEvent::GameComplete);
    }

    CARD_UPDATE_PATTERN
        .captures(line)
        .and_then(|group| {
            let id = group.name("id").map(|m| m.as_str());
            let card_id = group.name("card_id").map(|m| m.as_str());
            let player = group.name("player").map(|m| m.as_str());

            match (id, card_id, player) {
                (Some(id), Some(card_id), Some(player)) if card_id != "" => {
                    Some(LogEvent::Play(models::Play {
                                            id: id.to_string(),
                                            card_id: card_id.to_string(),
                                            player: player.to_string(),
                                        }))
                }
                _ => None,
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_complete() {
        let log_line = r"D 15:36:19.5943367 PowerTaskList.DebugPrintPower() -     TAG_CHANGE Entity=GameEntity tag=STATE value=COMPLETE";
        assert!(GAME_COMPLETE_PATTERN.is_match(log_line));
    }

    #[test]
    fn test_card_update() {
        let log_line = r"D 14:50:27.0664788 GameState.DebugPrintEntityChoices() -   Entities[4]=[name=La pièce id=68 zone=HAND zonePos=5 cardId=GAME_005 player=1]";
        assert!(CARD_UPDATE_PATTERN.is_match(log_line));
    }
}
