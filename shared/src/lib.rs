use byteorder::{LittleEndian, ReadBytesExt};
use log::LevelFilter;
use log4rs::append::rolling_file::policy::compound::roll::delete::DeleteRoller;
use log4rs::append::rolling_file::policy::compound::trigger::size::SizeTrigger;
use log4rs::append::rolling_file::policy::compound::CompoundPolicy;
use log4rs::append::rolling_file::RollingFileAppender;
use log4rs::config::{Appender, Root};
use log4rs::encode::pattern::PatternEncoder;
use log4rs::Config;
use ring::digest::{Context, SHA256};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::io::Cursor;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ClientToServerMessage {
    Join { channel_id: i32 },
    NewMessage(String),
    ChatsFromTodayRequest,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ServerToClientMessage {
    Joined { chatroom_id: i32 },
    NewUser { user_id: i32 },
    UserDisconnected { user_id: i32 },
    NewMessage(String),
    ChatsFromTodayResponse { messages: Vec<String> },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Chatroom {
    pub chatroom_id: i32,
    pub num_users: u32,
    pub online: bool,
    pub term: String,
    pub url: String,
}

pub fn get_channel_id(term: &str) -> i32 {
    let mut context = Context::new(&SHA256);
    context.update(term.as_bytes());
    let hash = context.finish();
    let bytes = hash.as_ref();
    let mut cursor = Cursor::new(bytes);
    return cursor.read_i32::<LittleEndian>().unwrap();
}

pub fn initialize_logger() -> Result<(), Box<dyn Error + Send + Sync>> {
    let roller = Box::new(DeleteRoller::new());
    let trigger = Box::new(SizeTrigger::new(1_000_000));
    let policy = Box::new(CompoundPolicy::new(trigger, roller));

    let logfile = RollingFileAppender::builder()
        .append(true)
        .encoder(Box::new(PatternEncoder::new("{l} - {m}\n")))
        .build("log/output.log", policy)?;

    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(Root::builder().appender("logfile").build(LevelFilter::Info))?;

    log4rs::init_config(config)?;

    Ok(())
}
