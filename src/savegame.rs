use base64;
use bincode;
use serde::{Deserialize, Serialize};
use stdweb::{js, unstable::TryInto};

pub struct SaveGameError;

impl From<bincode::Error> for SaveGameError {
    fn from(_: bincode::Error) -> Self {
        Self
    }
}

impl From<base64::DecodeError> for SaveGameError {
    fn from(_: base64::DecodeError) -> Self {
        Self
    }
}

fn encode<T>(game: T) -> Result<String, SaveGameError>
where
    T: Serialize,
{
    let bin = bincode::serialize(&game)?;
    Ok(base64::encode(&bin))
}

fn decode<T>(save: String) -> Result<T, SaveGameError>
where
    T: for<'a> Deserialize<'a>,
{
    let bin = base64::decode(&save)?;
    Ok(bincode::deserialize::<T>(&bin)?)
}

pub fn save<T>(key: &str, game: T)
where
    T: Serialize,
{
    if let Ok(encoded) = encode(game) {
        js! { localStorage.setItem(@{key}, @{encoded}) };
    }
}

pub fn load<T>(key: &str) -> Option<T>
where
    T: for<'a> Deserialize<'a>,
{
    let encoded = js! { return localStorage.getItem(@{key}); }.try_into().ok();
    encoded.and_then(|encoded| decode(encoded).ok())
}
