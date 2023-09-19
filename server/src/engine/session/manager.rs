
use crate::engine::player::Player;

use super::{Lobby, SessionError};
use std::cell::{RefCell, Ref};
use super::{LobbyInterface};
use std::sync::mpsc::{Sender,Receiver};


pub struct Move{
    source_lobby:Box<dyn LobbyInterface>,
    dest_lobby: Box<RefCell<dyn LobbyInterface>>,
    player:Box<RefCell<dyn Player>>
}

pub struct LobbyManager{
    lobbys:Vec<Box<RefCell<dyn LobbyInterface>>>,
    lobby_counter:usize
}


impl LobbyManager {
    pub fn lobbys(&self) -> Ref<Vec<Box<RefCell<dyn LobbyInterface>>>>{
        todo!();
    }
    pub fn get<'a>(&'a self,idx:usize) -> Box<RefCell<dyn LobbyInterface>>{
        todo!();
    }
    fn mv(action:Move) -> Result<(),SessionError>{
        let player = action.source_lobby.disconnect(action.player)
    }
    pub async fn main(self,move_listener:Receiver<Move>){
        loop{
            let rec = move_listener.recv();
            match rec{
                Ok(mv) => {
                    
                }
                Err(e) =>{
                    todo!();
                }
            }
        }

    }
}


