use serde::{Serialize};
use serde_with::skip_serializing_none;

use crate::text::{Text, TextColor};

#[derive(Serialize, Debug)]
pub struct Version {
    pub name: String,
    pub protocol: i64,
}

#[derive(Serialize, Debug)]
pub struct Player {
    pub name: String,
    pub id: String,
}

#[skip_serializing_none]
#[derive(Serialize, Debug)]
pub struct Players {
    pub max: i64,
    pub online: i64,
    pub sample: Option<Vec<Player>>,
}

#[derive(Serialize,  Debug)]
#[serde(untagged)]
pub enum Description {
    String(String),
    Text(Text),
}

#[skip_serializing_none]
#[derive(Serialize,  Debug)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub version: Version,
    pub players: Players,
    pub description: Description,
    pub favicion: Option<String>,
    pub enforce_secure_chat: Option<bool>,
}

pub fn get_offline_status(version: i64) -> Status {
    let version = version.max(766);
    let description = Description::Text(
        Text::literal("HII ")
            .with_color(TextColor::Hex("#048ac7".into()))
            .append(Text::literal("Server is currently ").with_color(TextColor::Gray))
            .append(Text::literal("offline.").with_color(TextColor::Red))
            .append(Text::literal(" Join to start it!").with_color(TextColor::White)),
    );
    Status {
        version: Version {
            name: "1.20.5+".into(),
            protocol: version,
        },
        players: Players {
            max: -1,
            online: 0,
            sample: None,
        },
        description,
        favicion: None,
        enforce_secure_chat: None,
    }
}
