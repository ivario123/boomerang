extern crate server;
use std::marker::PhantomData;

use super::{cards::AustralianActivity, Event, GameMetaData};
use server::engine::rules::{Action, Error, New, Received};

pub mod dealing;
pub mod discard;
pub mod game_end;
pub mod pass;
pub mod score;
pub mod show;
pub mod syncing;
pub mod waiting;

pub trait GameState: Send + std::fmt::Debug {
    fn get_next_action(
        &mut self,
        players: &Vec<usize>,
    ) -> (
        tokio::time::Duration,
        Vec<Action<New, Event>>,
        Option<Box<dyn GameState>>,
    );
    fn register_message(
        &mut self,
        action: &Action<New, Event>,
    ) -> Result<Option<Box<dyn GameState>>, Error>;
    fn register_response(
        &mut self,
        action: (Event, &Action<Received, Event>),
    ) -> Result<Option<Box<dyn GameState>>, Error>;
    fn metadata(&mut self) -> Option<&mut GameMetaData>;
}

pub trait AsMetaData: GameState {
    fn metadata(&mut self) -> &mut GameMetaData;
}
#[derive(Debug)]
pub struct DealingCards {
    state: GameMetaData,
    pending_actions: Vec<u8>,
    #[allow(dead_code)]
    validated: Vec<usize>,
}
#[derive(Debug)]
pub struct WaitingForPlayers<Next: AsMetaData + Send> {
    ready: Vec<u8>,
    pending_ready: Vec<u8>,
    next_state: Option<Box<Next>>,
}

#[derive(Debug)]
pub struct DiscardCard {
    state: GameMetaData,
    pending: Vec<u8>,
    requested: bool,
}
#[derive(Debug)]
pub struct PassHand<Next: AsMetaData + Send + Sync + From<GameMetaData>> {
    state: GameMetaData,
    pending: Vec<u8>,
    requested: bool,
    direction: pass::Direction,
    next: PhantomData<Next>,
}

#[derive(Debug)]
pub struct ShowCard {
    state: GameMetaData,
    pending: Vec<u8>,
    requested: bool,
}

#[derive(Debug)]
pub struct Scoring {
    state: GameMetaData,
    pending: Vec<u8>,
    requested: bool,
    actions: Vec<(u8, Option<AustralianActivity>)>,
}

#[derive(Debug)]
pub struct Syncing<Next: AsMetaData + Send + Sync> {
    state: GameMetaData,
    pending: Vec<u8>,
    requested: bool,
    next_state: Option<Box<Next>>,
}

#[derive(Debug)]
pub struct Final {
    state: GameMetaData,
    delivered: bool,
}

macro_rules! represent {
    ($($state:ident$(<$generic:ident>)?)+) => {
        $(
            impl$(<$generic: AsMetaData +Send +Sync +From<GameMetaData>+'static>)? AsMetaData for $state$(<$generic>)?{
                fn metadata(&mut self) -> &mut GameMetaData{
                    &mut self.state
                }
            }
        )+

    };
}
represent! {
    Final
    DealingCards
    DiscardCard
    ShowCard
    Scoring
    Syncing<Next>
    PassHand<Next>

}

#[cfg(test)]
mod test {

    use server::engine::rules::{Action, New};

    use crate::rules::{
        cards::{AustraliaCard, AustraliaDeck},
        AustraliaPlayer, Event,
    };

    use super::{pass::Direction, DealingCards, GameState, WaitingForPlayers};

    #[test]
    fn test_1_2() {
        let players = vec![1, 2];
        let expected_result = (
            vec![
                Action::<New, Event>::new(1, Event::ReadyCheck),
                Action::<New, Event>::new(2, Event::ReadyCheck),
            ],
            None,
        );
        test_waiting_state_with_players(players, expected_result);
    }

    #[test]
    fn test_1_3() {
        let players = vec![1, 2, 3];
        let expected_result = (
            vec![
                Action::<New, Event>::new(1, Event::ReadyCheck),
                Action::<New, Event>::new(2, Event::ReadyCheck),
                Action::<New, Event>::new(3, Event::ReadyCheck),
            ],
            None,
        );
        test_waiting_state_with_players(players, expected_result);
    }

    #[test]
    fn test_1_4() {
        let players = vec![1, 2, 3, 4];
        let expected_result = (
            vec![
                Action::<New, Event>::new(1, Event::ReadyCheck),
                Action::<New, Event>::new(2, Event::ReadyCheck),
                Action::<New, Event>::new(3, Event::ReadyCheck),
                Action::<New, Event>::new(4, Event::ReadyCheck),
            ],
            None,
        );
        test_waiting_state_with_players(players, expected_result);
    }

    #[test]
    fn test_1_5() {
        let players = vec![1, 2, 3, 4, 5];
        let expected_result = (
            vec![
                Action::<New, Event>::new(1, Event::LobbyFull),
                Action::<New, Event>::new(2, Event::LobbyFull),
                Action::<New, Event>::new(3, Event::LobbyFull),
                Action::<New, Event>::new(4, Event::LobbyFull),
                Action::<New, Event>::new(5, Event::LobbyFull),
            ],
            None,
        );
        test_waiting_state_with_players(players, expected_result);
    }

    #[test]
    fn deal() {
        assert_eq!(AustraliaDeck::default().cards().len(), 28)
    }

    /// Deals cards until a state transition is triggered.
    ///
    /// This test will only pass if the number of cards dealt to each players is 7
    #[test]
    fn test_4() {
        deal_all();
    }
    #[test]
    fn test_4_and_5() {
        let state = deal_all();
        let state = sync(state);
        discard_card(state);
    }

    #[test]
    fn test_4_thru_7() {
        let state = deal_all();
        let state = sync(state);
        let state = discard_card(state);
        let state = pass_hand(state, Direction::Forward);
        let state = sync(state);
        let _state = show_card(state);
    }

    #[test]
    fn test_4_thru_8() {
        let state = deal_all();
        let state = sync(state);
        let mut state = discard_card(state);
        for _ in 0..5 {
            state = pass_hand(state, Direction::Forward);
            state = sync(state);
            state = show_card(state);
        }
    }

    #[test]
    fn test_4_thru_12() {
        let mut state = deal_all();
        for i in 0..4 {
            if i != 0 {
                state = deal_from_state(state);
            }
            state = sync(state);
            state = discard_card(state);
            for j in 0..6 {
                if j != 5 {
                    println!("Action {:?} round {:?}, pass_hand", j + 1, i);
                    state = pass_hand(state, Direction::Forward);
                    println!("Action {:?}, pass_hand forward Ok", j + 1);
                } else {
                    println!("Action {:?} round {:?}, pass_hand", j + 1, i);
                    state = pass_hand(state, Direction::Backward);
                    println!("Action {:?}, pass_hand forward Ok", j + 1);
                }
                println!("Action {:?} round {:?}, sync", j + 1, i);
                state = sync(state);
                println!("Action {:?}, sync Ok", j + 1);

                if j != 5 {
                    println!("Action {:?} round {:?}, show_card", j + 1, i);
                    state = show_card(state);
                    println!("Action {:?}, show_card Ok", j + 1);
                } else {
                    // Here we are going  to score things
                    println!("Action {:?} round {:?}, score_state", j + 1, i);
                    state = score_state(state);
                    if i != 3 {
                        state = sync(state);
                    }
                }
            }
        }

        println!("Action game_end");
        game_end(state);
        println!("Game end Ok");
    }

    // ==============================================================================
    //                              Helpers
    // ==============================================================================
    fn test_waiting_state_with_players(
        players: Vec<usize>,
        expect: (Vec<Action<New, Event>>, Option<Box<dyn GameState>>),
    ) {
        let mut waiting_state = WaitingForPlayers::<DealingCards>::new(None);

        let (_duration, actions, _next_state) = waiting_state.get_next_action(&players);
        assert_eq!(expect.0, actions);
    }
    fn hands(players: &Vec<AustraliaPlayer>) -> Vec<Vec<AustraliaCard>> {
        let mut ret = Vec::new();
        for player in players {
            ret.push(player.get_hand())
        }
        ret
    }
    fn deal_from_state(mut current_state: Box<dyn GameState>) -> Box<dyn GameState> {
        println!("State: {:?}", current_state);
        let players = vec![0, 1, 2, 3];
        // Create a mock DealingCards state with the game logic.
        let mut next_state = None;
        let mut card_counter = 0;
        // Simulate dealing the card
        while let None = next_state {
            let (_duration, actions, state) = current_state.get_next_action(&players);

            println!("Requested actions: {:?}", actions);
            match state {
                None => {
                    for action in actions.iter() {
                        match action.action() {
                            Event::Deal(_) => {
                                current_state
                                    .register_response((
                                        Event::Accept,
                                        &(action.clone()).transition().transition(),
                                    ))
                                    .unwrap();
                            }
                            _ => {
                                assert!(false);
                            }
                        }
                    }
                }
                _ => {
                    // Transition case, this should not increment the counter
                    next_state = state;
                    continue;
                }
            }
            next_state = state;

            let mut dealing = true;
            for action in actions {
                match action.action() {
                    Event::Deal(_) => {}
                    _ => dealing = false,
                }
            }
            if dealing {
                card_counter += 1;
            }
        }
        assert_eq!(card_counter, 7);
        next_state.unwrap()
    }
    fn deal_all() -> Box<dyn GameState> {
        let players = vec![0, 1, 2, 3];
        // Create a mock DealingCards state with the game logic.
        let mut dealing_cards_state = DealingCards::new(&players);
        let mut next_state = None;
        let mut card_counter = 0;
        // Simulate dealing the card
        while let None = next_state {
            let (_duration, actions, state) = dealing_cards_state.get_next_action(&players);
            match state {
                None => {
                    for action in actions.iter() {
                        match action.action() {
                            Event::Deal(_) => {
                                dealing_cards_state
                                    .register_response((
                                        Event::Accept,
                                        &(action.clone()).transition().transition(),
                                    ))
                                    .unwrap();
                            }
                            _ => {
                                assert!(false);
                            }
                        }
                    }
                }
                _ => {
                    // Transition case, this should not increment the counter
                    next_state = state;
                    continue;
                }
            }
            next_state = state;

            let mut dealing = true;
            for action in actions {
                match action.action() {
                    Event::Deal(_) => {}
                    _ => dealing = false,
                }
            }
            if dealing {
                card_counter += 1;
            }
        }
        assert_eq!(card_counter, 7);
        next_state.unwrap()
    }
    fn sync(mut current_state: Box<dyn GameState>) -> Box<dyn GameState> {
        let players = vec![0, 1, 2, 3];
        let mut next_state = None;

        while let None = next_state {
            let (_duration, actions, state) = current_state.get_next_action(&players);
            println!("Requested actions: {:?}", actions);

            for action in actions.iter() {
                match action.action() {
                    Event::Sync(_) => {
                        let response = Event::Accept;
                        current_state
                            .register_response((
                                response,
                                &action.clone().transition().transition(),
                            ))
                            .unwrap();
                    }
                    _ => {
                        assert!(false);
                    }
                }
            }

            next_state = state;
        }

        next_state.unwrap()
    }
    fn pass_hand(
        mut current_state: Box<dyn GameState>,
        direction: Direction,
    ) -> Box<dyn GameState> {
        let players = vec![0, 1, 2, 3];
        let mut next_state = None;

        // The hands before passing
        let mut initial_hands = hands(
            &current_state
                .metadata()
                .expect("Metadata is not available.")
                .hands(),
        );

        while let None = next_state {
            let (_duration, actions, state) = current_state.get_next_action(&players);
            println!("Requested actions: {:?}", actions);

            for action in actions.iter() {
                match action.action() {
                    Event::ReassignHand(_) => {
                        let response = Event::Accept;
                        current_state
                            .register_response((
                                response,
                                &action.clone().transition().transition(),
                            ))
                            .unwrap();
                    }
                    _ => {
                        assert!(false);
                    }
                }
            }

            next_state = state;
        }
        let mut state = next_state.unwrap();
        let mut final_hands = hands(
            &state
                .metadata()
                .expect("Metadata is not available.")
                .hands(),
        );
        println!(
            "Initial hands : {:?} \nfinal_hands : {:?}",
            initial_hands, final_hands
        );
        match direction {
            Direction::Forward => {
                assert_eq!(initial_hands.pop().unwrap(), final_hands.remove(0));
                assert_eq!(initial_hands, final_hands);
            }
            Direction::Backward => {
                assert_eq!(initial_hands.remove(0), final_hands.pop().unwrap());
                assert_eq!(initial_hands, final_hands);
            }
        }

        state
    }
    fn discard_card(mut current_state: Box<dyn GameState>) -> Box<dyn GameState> {
        let players = vec![0, 1, 2, 3];
        let mut next_state = None;
        let mut discard_counter = 0;
        // Simulate dealing the card
        while let None = next_state {
            let (_duration, actions, state) = current_state.get_next_action(&players);
            println!("Requested actions : {:?}", actions);
            match state {
                None => {
                    for action in actions.iter() {
                        if action.action() == Event::DiscardRequest {
                            discard_counter += 1;
                            current_state
                                .register_response((
                                    Event::Discard(0),
                                    &(action.clone()).transition().transition(),
                                ))
                                .unwrap();
                        } else {
                            assert!(false);
                        }
                    }
                }
                _ => {}
            }
            next_state = state;
        }
        assert_eq!(discard_counter, 4);
        next_state.unwrap()
    }
    fn show_card(mut current_state: Box<dyn GameState>) -> Box<dyn GameState> {
        let players = vec![0, 1, 2, 3];
        let mut next_state = None;
        let mut show_counter = 0;

        while let None = next_state {
            let (_duration, actions, state) = current_state.get_next_action(&players);
            println!("Requested actions: {:?}", actions);

            match state {
                None => {
                    for action in actions.iter() {
                        if action.action() == Event::ShowRequest {
                            show_counter += 1;
                            // Simulate player response
                            let response = Event::Show(0); // Simulate showing a card at index 0
                            current_state
                                .register_response((
                                    response,
                                    &(action.clone()).transition().transition(),
                                ))
                                .unwrap();
                        } else {
                            assert!(false);
                        }
                    }
                }
                _ => {}
            }

            next_state = state;
        }

        assert_eq!(show_counter, 4);

        next_state.unwrap()
    }

    fn score_state(mut current_state: Box<dyn GameState>) -> Box<dyn GameState> {
        let players: Vec<usize> = vec![0, 1, 2, 3];
        let mut next_state: Option<Box<dyn GameState>> = None;
        let mut score_counter = 0;
        while let None = next_state {
            let (_duration, actions, state) = current_state.get_next_action(&players);
            println!("Requested actions: {:?}", actions);

            match state {
                None => {
                    for action in actions.iter() {
                        match action.action() {
                            Event::ScoreActivityQuery(options) => {
                                let response = Event::ScoreActivity(options.get(0).copied());
                                current_state
                                    .register_response((
                                        response,
                                        &action.clone().transition().transition(),
                                    ))
                                    .unwrap();
                                score_counter += 1;
                            }
                            Event::NewRound => {}
                            _ => {
                                assert!(false);
                            }
                        }
                    }
                }
                Some(state) => {
                    next_state = Some(state);
                    for action in actions.iter() {
                        match action.action() {
                            Event::NewRound => {}
                            _ => {
                                assert!(false);
                            }
                        }
                    }
                }
            }
        }

        assert_eq!(score_counter, 4);

        next_state.unwrap()
    }

    fn game_end(mut current_state: Box<dyn GameState>) {
        let players = vec![0, 1, 2, 3];
        let mut score_recv_counter = 0;
        let (_duration, actions, state) = current_state.get_next_action(&players);
        println!("Requested actions: {:?}", actions);

        match state {
            None => {
                for action in actions.iter() {
                    match action.action() {
                        Event::FinalResult(_, _) => {
                            score_recv_counter += 1;
                        }
                        _ => {
                            assert!(false);
                        }
                    }
                }
            }
            _ => {}
        }

        assert_eq!(score_recv_counter, 4);
    }
}
