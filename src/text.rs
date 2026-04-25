use serde::{Serialize};
use serde_with::skip_serializing_none;

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum TextType {
    Text,
    Translatable,
    Score,
    Selector,
    Keybind,
    Nbt,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum TextColor {
    Black,
    DarkBlue,
    DarkGreen,
    DarkAqua,
    DarkRed,
    DarkPurple,
    Gold,
    Gray,
    DarkGray,
    Blue,
    Green,
    Aqua,
    Red,
    LightPurple,
    Yellow,
    White,
    #[serde(untagged)]
    Hex(String),
}

#[skip_serializing_none]
#[derive(Serialize, Debug,Default)]
pub struct Text {
    pub text: Option<String>,
    pub atlas: Option<String>,
    pub sprite: Option<String>,
    pub player: Option<String>,
    pub extra: Option<Vec<Text>>,
    pub color: Option<TextColor>,
}

impl Text {
    pub fn new() -> Self {
        Self {
            text: None,
            atlas: None,
            sprite: None,
            player: None,
            extra: None,
            color: None,
        }
    }

    pub fn literal(text: &str) -> Self {
        let mut result = Self::new();
        result.text = Some(text.into());
        result
    }

    pub fn player(player: &str) -> Self {
        let mut result = Self::new();
        result.player = Some(player.into());
        result
    }

    pub fn sprite(atlas: &str, sprite: &str) -> Self {
        let mut result = Self::new();
        result.atlas = Some(atlas.into());
        result.sprite = Some(sprite.into());
        result
    }

    pub fn with_color(mut self, color: TextColor) -> Self {
        self.color = Some(color);
        self
    }

    pub fn append(mut self, text: Self) -> Self {
        if let Some(ref mut extra) = self.extra {
            extra.push(text);
        } else {
            self.extra = Some(vec![text]);
        }

        self
    }
}
