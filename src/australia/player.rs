use async_recursion::async_recursion;
use log::{error, info, warn};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::tcp::{OwnedReadHalf, OwnedWriteHalf},
    sync::broadcast::{self, Receiver},
};

use super::protocol::{Event, Message};

#[async_recursion]
/// Manages incoming messages
///
/// This function manages incoming messages and passes them to the event manager for further interpretation
pub async fn read_event(mut read_part: OwnedReadHalf, channel: broadcast::Sender<Event>) {
    loop {
        info!("Waiting for events");
        let mut buff = vec![0; 2048];
        let recv = read_part.read(&mut buff).await;
        match recv {
            Ok(_) => {}
            Err(_) => {
                continue;
            }
        }
        while let Some(0) = buff.last() {
            buff.pop();
        }

        let recv = String::from_utf8_lossy(&buff).to_string();
        info!("Server sent {:?}", recv);
        channel
            .send(match serde_json::from_str::<Event>(recv.as_str()) {
                Ok(val) => {
                    info!("returning {:?}", val);
                    val
                }
                _ => continue,
            })
            .unwrap();
    }
}

pub async fn manage_event(
    writer: tokio::sync::broadcast::Sender<Message>,
    mut feedback_reader: tokio::sync::broadcast::Receiver<Message>,
    mut reader: Receiver<Event>,
    mut write_part: OwnedWriteHalf,
) {
    info!("Monitoring TCP");

    loop {
        let event: Event = match reader.recv().await {
            Ok(event) => event,
            _ => continue,
        };
        info!("Server sent {:?}", event);

        let to_send: Event = match event {
            Event::ReadyCheck => {
                writer.send(Message::ReadyCheck).unwrap();
                #[allow(unused_assignments)]
                let mut ret = Event::Accept;
                loop {
                    warn!("Waiting for message from frontend");
                    match feedback_reader.recv().await {
                        Ok(Message::Ready) => {
                            info!("Message received from frontend");
                            ret = Event::Accept;
                            break;
                        }
                        Ok(Message::NotReady) => {
                            info!("Message received from frontend");
                            ret = Event::Deny;
                            break;
                        }
                        Err(_) => {
                            writer.send(Message::Exit).unwrap();
                            return;
                        }
                        _ => {}
                    }
                }
                ret
            }
            Event::Deal(card) => {
                writer.send(Message::Deal(card)).unwrap();
                Event::Accept
            }
            Event::UnexpectedMessage => continue,
            Event::DiscardRequest => {
                writer.send(Message::DiscardQuery).unwrap();
                #[allow(unused_assignments)]
                let mut ret = 0;
                loop {
                    warn!("Waiting for message from frontend");
                    match feedback_reader.recv().await {
                        Ok(Message::Discard(_, idx)) => {
                            info!("Message received from frontend");
                            ret = idx;
                            break;
                        }
                        _ => {
                            writer.send(Message::Exit).unwrap();
                            return;
                        }
                    }
                }
                Event::Discard(ret)
            }
            Event::ShowRequest => {
                writer.send(Message::ShowQuery).unwrap();
                #[allow(unused_assignments)]
                let mut ret = 0;
                loop {
                    warn!("Waiting for message from frontend");
                    match feedback_reader.recv().await {
                        Ok(Message::Show(_, idx)) => {
                            info!("Message received from frontend");
                            ret = idx;
                            break;
                        }
                        _ => return,
                    }
                }
                Event::Show(ret)
            }
            Event::ShowPile(idx, cards, visited) => {
                info!("Server sent ShowPile({:?},{:?})", idx, cards);
                writer
                    .send(Message::ShowOtherHand(idx.into(), cards, visited))
                    .unwrap();
                continue;
            }
            Event::ReassignHand(new_hand) => {
                info!("Replacing hand with with {:?}", new_hand);
                writer.send(Message::ReassignHand(new_hand)).unwrap();
                info!("Replaced");
                Event::Accept
            }
            Event::WaitingForPlayers => {
                writer.send(Message::WaitingForPlayers).unwrap();
                continue;
            }
            Event::Sync(player) => {
                loop {
                    match writer.send(Message::Sync(player.clone())) {
                        Ok(_) => break,
                        Err(_) => {
                            error!(
                                "Frontend managed must have crashed silently the channel is closed"
                            );
                            return;
                        }
                    }
                }
                loop {
                    warn!("Waiting for message from frontend");
                    match feedback_reader.recv().await {
                        Ok(Message::Ok) => {
                            break;
                        }
                        other => {
                            error!("Received unexpected {:?} from frontend", other);
                            return;
                        }
                    }
                }
                Event::Accept
            }
            Event::ScoreActivityQuery(options) => {
                writer.send(Message::ScoreActivityQuery(options)).unwrap();
                #[allow(unused_assignments)]
                let mut ret = None;
                loop {
                    warn!("Waiting for message from frontend");
                    match feedback_reader.recv().await {
                        Ok(Message::ScoreActivity(x)) => {
                            info!("Message received from frontend");
                            ret = x;
                            break;
                        }
                        Err(_) => {
                            writer.send(Message::Exit).unwrap();
                            return;
                        }
                        _ => {
                            continue;
                        }
                    }
                }
                Event::ScoreActivity(ret)
            }
            Event::NewRound => {
                writer.send(Message::NewRound).unwrap();
                continue;
            }
            Event::FinalResult(uid, scores) => {
                // At this point we should disconnect
                writer.send(Message::FinalResult(uid, scores)).unwrap();
                return;
            }
            unexpected => {
                error!("Got unhandled message: {:?}", unexpected);
                continue;
            }
        }
        .into();

        send_event(&mut write_part, to_send).await;
    }
}

async fn send_event(write_part: &mut OwnedWriteHalf, event: Event) {
    let to_send: Vec<u8> = event.into();
    write_part.write_all(&to_send).await.unwrap();
}
