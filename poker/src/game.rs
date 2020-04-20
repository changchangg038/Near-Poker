use crate::deck::{Deck, DeckError, DeckStatus};
use crate::poker::{Poker, PokerStatus};
use crate::types::{CryptoHash, RoomId};
use borsh::{BorshDeserialize, BorshSerialize};
use serde::Serialize;

#[derive(BorshDeserialize, BorshSerialize, Serialize)]
pub enum GameError {
    DeckError(DeckError),
    RoomIdNotFound,
    OngoingRound,
}

impl From<DeckError> for GameError {
    fn from(deck_error: DeckError) -> Self {
        GameError::DeckError(deck_error)
    }
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Eq, PartialEq, Clone)]
pub enum GameStatus {
    // Start haven't been called. Players are able to enter the game.
    Initiating,
    // Round already finished. Call start to start next round.
    Idle,
    // Need action by some player in deck.
    DeckAction(DeckStatus),
    // Need action by some player in poker.
    PokerAction(PokerStatus),
    // Game have been closed
    Closed,
}

impl GameStatus {
    pub fn is_active(&self) -> bool {
        *self != GameStatus::Closed
    }

    pub fn is_initiating(&self) -> bool {
        *self == GameStatus::DeckAction(DeckStatus::Initiating)
    }
}

// TODO: Use temporary fake money. Then force to use near tokens.
#[derive(BorshDeserialize, BorshSerialize, Serialize)]
pub struct Game {
    pub name: String,
    pub id: RoomId,
    pub status: GameStatus,
    deck: Deck,
    poker: Poker,
}

impl Game {
    pub fn new(name: String, id: RoomId) -> Self {
        Self {
            name,
            id,
            status: GameStatus::Initiating,
            deck: Deck::new(52),
            poker: Poker::new(),
        }
    }

    pub fn enter(&mut self) -> Result<(), GameError> {
        self.deck.enter().map_err(Into::<GameError>::into)?;
        // TODO: Use near tokens
        // TODO: Put min tokens / max tokens caps
        self.poker.new_player(1000);
        Ok(())
    }

    pub fn start(&mut self) -> Result<(), GameError> {
        match self.status {
            GameStatus::Initiating | GameStatus::Idle => {
                self.deck.start().map_err(Into::<GameError>::into)?;
                self.status = GameStatus::DeckAction(self.deck.get_status());
                Ok(())
            }
            _ => Err(GameError::OngoingRound),
        }
    }

    // TODO: Don't allow to close a running game, unless it is in particular states.
    pub fn close(&mut self) -> Result<(), GameError> {
        match self.status {
            GameStatus::Initiating | GameStatus::Idle => {
                self.deck.close();
                self.status = GameStatus::Closed;
                Ok(())
            }
            _ => Err(GameError::OngoingRound),
        }
    }

    /// Currently in deck action
    fn check_next_status(&mut self) {
        let deck_status = self.deck.get_status();

        if deck_status != DeckStatus::Running {
            self.status = GameStatus::DeckAction(deck_status);
            return;
        }

        // TODO: Here
        self.poker.next()
    }

    pub fn deck_state(&self) -> Deck {
        self.deck.clone()
    }

    pub fn poker_state(&self) -> Poker {
        self.poker.clone()
    }

    pub fn state(&self) -> GameStatus {
        self.status.clone()
    }

    // TODO: Implement this method to find guilty that stalled the game.
    // /// Current player that should make an action.
    // pub fn required_action(&self) -> Option<PlayerId> {}
}

// Implement public interface for deck
impl Game {
    pub fn get_partial_shuffle(&self) -> Result<Vec<CryptoHash>, GameError> {
        self.deck.get_partial_shuffle().map_err(Into::into)
    }

    pub fn submit_shuffled(&mut self, new_cards: Vec<CryptoHash>) -> Result<(), GameError> {
        self.deck
            .submit_shuffled(new_cards)
            .map_err(Into::<GameError>::into)?;

        self.check_next_status();
        Ok(())
    }

    pub fn finish_reveal(&mut self) -> Result<(), GameError> {
        self.deck.finish_reveal().map_err(Into::<GameError>::into)?;

        self.check_next_status();
        Ok(())
    }

    pub fn submit_reveal_part(&mut self, card: CryptoHash) -> Result<(), GameError> {
        self.deck
            .submit_reveal_part(card)
            .map_err(Into::<GameError>::into)?;

        self.check_next_status();
        Ok(())
    }
}