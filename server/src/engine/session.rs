use super::player::Player;
use super::rules::RuleSet;

pub enum SessionError {
    /// Thrown when a disconnect is requested for a non exsisting player
    NoSuchPlayer,
    /// Thrown when a player is trying to connect to a full lobby
    LobbyFull,
}

struct Lobby<R: RuleSet, const CAPACITY: usize> {
    players: Vec<Box<dyn Player>>,
    rules: R,
}

impl<R: RuleSet, const CAPACITY: usize> Lobby<R, CAPACITY> {
    pub fn new<ID: Sized>(id: ID) -> Self {
        Self {
            players: Vec::with_capacity(CAPACITY),
            rules: R::new(),
        }
    }

    pub async fn start(&self) {
        todo!();
    }

    /// Closes the session
    pub fn close(self) -> Vec<Box<dyn Player>> {
        // Maybe we should notify the players here.
        self.players
    }
    // Called when session goes out of scope.
    fn free(self) {
        self.close();
    }
    // Some trait things
    /// Connects a specific player to a specific session  
    pub fn connect<P: Player + 'static>(&mut self, player: P) -> Result<(), SessionError> {
        let num_players = self.players.len();
        match num_players >= CAPACITY {
            true => Err(SessionError::LobbyFull),
            _ => {
                self.players.push(Box::new(player));
                Ok(())
            }
        }
    }
    /// Disconnects a player from a session
    pub fn disconnect<P: Player>(&mut self, player: &P) -> Result<Box<dyn Player>, SessionError> {
        let mut id = None;
        for (idx, el) in self.players.iter().enumerate() {
            if player.getid() == el.getid() {
                id = Some(idx);
                break;
            }
        }
        match id {
            Some(idx) => Ok(self.players.remove(idx)),
            None => Err(SessionError::NoSuchPlayer),
        }
    }
}
