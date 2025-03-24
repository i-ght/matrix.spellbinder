use core::sync;
use std::{error::Error, fs, io, str::FromStr, time::Duration};

use danse_macabre::{DanseMacabreCard, DanseMacabreCardKey};
use ruma::{
    api::client::{
        media::{self, create_content},
        message::send_message_event::{self},
        sync::sync_events,
    },
    events::{
        policy::rule::user,
        room::{
            message::{
                ImageMessageEventContent, MessageType, RoomMessageEventContent,
                TextMessageEventContent,
            },
            MediaSource,
        },
        OriginalSyncMessageLikeEvent, SyncMessageLikeEvent,
    },
    presence::PresenceState,
    OwnedRoomId, OwnedUserId, RoomId, ServerName, TransactionId,
};

use ruma::events::room::message::MessageType::Text;
use ruma::events::AnySyncMessageLikeEvent::RoomMessage;
use ruma::events::AnySyncTimelineEvent::MessageLike;
use ruma::events::SyncMessageLikeEvent::Original;
use ruma_client::Client;

use rand::Rng;

type MatrixClient = ruma_client::Client<ruma_client::http_client::HyperNativeTls>;

use tao_te_ching::load_tao_te_ching;
use tokio_stream::StreamExt;

const SYNC_TOKEN_PATH: &str = "./sync_token.json";

mod danse_macabre;
mod tao_te_ching;


struct MatrixCfg {
    local_user_id: OwnedUserId,
    client: MatrixClient,
    tao_deck: Vec<String>,
    danse_deck: Vec<DanseMacabreCard>,
}

fn load_sync_token() -> io::Result<Option<String>> {
    match fs::read_to_string(SYNC_TOKEN_PATH) {
        Ok(access_token) => Ok(Some(access_token)),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e),
    }
}

fn save_sync_token(sync_token: &str) -> io::Result<()> {
    fs::write(SYNC_TOKEN_PATH, sync_token)
}

async fn send_text_msg(
    client: &MatrixClient,
    room_id: &OwnedRoomId,
    content: &str,
) -> Result<send_message_event::v3::Response, Box<dyn Error>> {
    let request = send_message_event::v3::Request::new(
        room_id.clone(),
        TransactionId::new(),
        &RoomMessageEventContent::text_plain(content),
    )?;

    let response = client.send_request(request).await?;
    return Ok(response);
}

async fn upload_content(
    client: &MatrixClient,
    data: Vec<u8>,
) -> Result<create_content::v3::Response, Box<dyn Error>> {
    let request = create_content::v3::Request::new(data);
    let response = client.send_request(request).await?;
    Ok(response)
}

async fn send_image_msg(
    client: &MatrixClient,
    room_id: &OwnedRoomId,
    content: &str,
    data: Vec<u8>,
) -> Result<send_message_event::v3::Response, Box<dyn Error>> {
    let response = upload_content(client, data).await?;
    let media_source = MediaSource::Plain(response.content_uri);

    let content = RoomMessageEventContent::new(MessageType::Image(ImageMessageEventContent::new(
        String::from(content),
        media_source,
    )));

    let request =
        send_message_event::v3::Request::new(room_id.clone(), TransactionId::new(), &content)?;

    let response = client.send_request(request).await?;
    return Ok(response);
}

async fn init_sync(client: &MatrixClient) -> Result<String, Box<dyn Error>> {
    let request = sync_events::v3::Request::new();
    let response = client.send_request(request).await?;

    for (room_id, room) in response.rooms.join.iter() {
        for event in &room.timeline.events {
            println!("{}: {}", room_id, event.json());
        }
    }

    return Ok(response.next_batch);
}

fn compute_random_tao_chapter() -> usize {
    let mut rng = rand::rng();
    loop {
        let random = rng.random::<u8>();
        match random {
            chapter @ 1..=81 => return chapter as usize,
            _ => continue,
        }
    }
}

fn compute_tao_chapter(user_input: &str) -> usize {
    match user_input.parse::<usize>() {
        Ok(chapter @ 1..=81) => chapter,
        _ => compute_random_tao_chapter(),
    }
}

async fn send_tao(
    client: &MatrixClient,
    room_id: &OwnedRoomId,
    tao: &[String],
    chapter: usize,
) -> Result<(), Box<dyn Error>> {
    let wisdom = &tao[chapter - 1];
    let _response = send_text_msg(&client, room_id, &wisdom[..]).await?;
    Ok(())
}

async fn send_danse_macabre(
    client: &MatrixClient,
    room_id: &OwnedRoomId,
    card: DanseMacabreCardKey,
    deck: &Vec<DanseMacabreCard>,
) -> Result<(), Box<dyn Error>> {
    let card = &deck[card as usize];

    let msg = vec![
        card.key.to_string(),
        card.german_name.clone(),
        card.description.clone(),
        String::from(""),
        card.bible_verse.clone(),
        String::from(""),
        card.quatrain.clone(),
        String::from(""),
        card.bible_verse_eng.clone(),
        String::from(""),
        card.quatrain_eng.clone(),
    ]
    .join("\n");

    let _response = send_text_msg(client, room_id, &msg[..]).await?;

    let name = format!("{}.jpg", &card.key.to_string());
    let _response = send_image_msg(client, room_id, &name[..], card.card_image.clone()).await?;

    Ok(())
}

fn compute_random_danse_card_key() -> usize {
    let mut rng = rand::rng();
    loop {
        let random = rng.random::<u8>();
        match random {
            card @ 0..=48 => return card as usize,
            _ => continue,
        }
    }
}

fn compute_random_danse_card() -> DanseMacabreCardKey {
    let key = compute_random_danse_card_key();
    DanseMacabreCardKey::try_from(key).unwrap()
}

fn compute_danse_card(user_input: &str) -> DanseMacabreCardKey {
    match user_input.parse::<usize>() {
        Ok(key @ 1..=49) => {
            DanseMacabreCardKey::try_from(key - 1).unwrap()
        },
        _ => match DanseMacabreCardKey::from_str(user_input) {
            Ok(card) => card,
            Err(_) => compute_random_danse_card(),
        },
    }
}

enum Cmd {
    Tao(usize),
    DanseMacabre(DanseMacabreCardKey),
}

fn construct_tao_cmd(user_input: &[&str]) -> Cmd {
    let chapter = match user_input {
        [head, _tail @ ..] => compute_tao_chapter(head),
        _ => compute_random_tao_chapter(),
    };

    Cmd::Tao(chapter)
}

fn construct_danse_cmd(user_input: &[&str]) -> Cmd {
    let card = match user_input {
        [head, _tail @ ..] => compute_danse_card(head),
        _ => compute_random_danse_card(),
    };
    Cmd::DanseMacabre(card)
}

fn try_parse_cmd(cmd: &[&str]) -> Option<Cmd> {
    match cmd {
        ["./tao", user_input @ ..] => Some(construct_tao_cmd(user_input)),
        ["./dansemacabre", user_input @ ..]
        | ["./danse_macabre", user_input @ ..]
        | ["./danse", user_input @ ..]
        | ["./dance", user_input @ ..]
        | ["./death", user_input @ ..]
        | ["./die", user_input @ ..] => Some(construct_danse_cmd(user_input)),

        _ => None,
    }
}

async fn exec_matrix_cmd(cmd: Cmd, room_id: &OwnedRoomId, cfg: &MatrixCfg) -> Result<(), Box<dyn Error>> {
    match cmd {
        Cmd::Tao(chapter) => send_tao(&cfg.client, room_id, &cfg.tao_deck[..], chapter).await,
        Cmd::DanseMacabre(card) => {
            send_danse_macabre(&cfg.client, room_id, card, &cfg.danse_deck).await
        }
    }
}

fn compute_output(cmd: Cmd) {

}

fn compute_user_input(
    user_input: &str
) {
    
    let cmd: Vec<&str> = user_input.split(' ').collect();
    let cmd = &cmd[..];

    let cmd = try_parse_cmd(cmd);
    if let Some(cmd) = cmd {
        compute_output(cmd);
    }


}

async fn compute_matrix_text_msg(
    text_msg: TextMessageEventContent,
    room_id: &OwnedRoomId,
    cfg: &MatrixCfg,
) -> Result<(), Box<dyn Error>> {
    dbg!("received text message: {:#?}", &text_msg);

    let content = text_msg.body.to_ascii_lowercase();
    let cmd: Vec<&str> = content.split(' ').collect();
    let cmd = &cmd[..];

    let cmd = try_parse_cmd(cmd);

    if let Some(cmd) = cmd {
        exec_matrix_cmd(cmd, room_id, cfg).await?
    }

    Ok(())
}

async fn head(cfg: MatrixCfg) -> Result<(), Box<dyn Error>> {
    let since_token = match load_sync_token() {
        Ok(Some(since_token)) => since_token,
        Ok(None) => {
            let sync_token = init_sync(&cfg.client).await?;
            save_sync_token(&sync_token[..])?;
            sync_token
        }
        Err(e) => return Err(Box::new(e)),
    };

    let timeout = Duration::from_secs(30);

    let mut sync_stream =
        Box::pin(
            cfg.client
                .sync(None, since_token, PresenceState::Online, Some(timeout)),
        );

    while let Some(response) = sync_stream.try_next().await? {
        save_sync_token(&response.next_batch)?;

        for (room_id, room) in response.rooms.join.iter() {
            for event in &room.timeline.events {
                match event.deserialize() {
                    Ok(MessageLike(RoomMessage(Original(msg))))
                        if msg.sender != cfg.local_user_id =>
                    {
                        if let Text(text_msg) = msg.content.msgtype {
                            compute_matrix_text_msg(text_msg, room_id, &cfg).await?
                        }
                    }
                    Ok(_) => continue,
                    Err(err) => return Err(Box::new(err)),
                }
            }
        }
    }

    Ok(())
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let tao_deck = load_tao_te_ching();
    let danse_deck = danse_macabre::load_danse_deck();

    let homeserver_url = "https://matrix.org".to_owned();

    let client = Client::builder()
        .homeserver_url(homeserver_url)
        .build::<ruma_client::http_client::HyperNativeTls>()
        .await?;

    let session = client
        .log_in("@spell.binder:matrix.org", "joyToTheWorld99", None, None)
        .await?;

    let local_user_id = session.user_id;

    dbg!(
        "Logged in as {} with access token: {}",
        &local_user_id,
        session.access_token
    );

    let cfg = MatrixCfg {
        local_user_id,
        client,
        tao_deck,
        danse_deck
    };
    head(cfg).await?;

    Ok(())
}
