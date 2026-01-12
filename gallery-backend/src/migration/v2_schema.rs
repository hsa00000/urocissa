#![allow(clippy::too_many_lines)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]

use arrayvec::ArrayString;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use redb_old::{TypeName, Value};

#[derive(Debug, Clone, Deserialize, Default, Serialize, Decode, Encode, PartialEq, Eq)]
pub struct Database {
    pub hash: ArrayString<64>,
    pub size: u64,
    pub width: u32,
    pub height: u32,
    pub thumbhash: Vec<u8>,
    pub phash: Vec<u8>,
    pub ext: String,
    pub exif_vec: BTreeMap<String, String>,
    pub tag: HashSet<String>,
    pub album: HashSet<ArrayString<64>>,
    pub alias: Vec<OldFileModify>,
    pub ext_type: String,
    pub pending: bool,
}

impl Value for Database {
    type SelfType<'a> = Self;
    type AsBytes<'a> = Vec<u8>;

    fn fixed_width() -> Option<usize> {
        None
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        bitcode::decode(data).expect("Corrupt Data: Failed to decode OldDatabase via bitcode")
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a> {
        bitcode::encode(value)
    }

    fn type_name() -> TypeName {
        TypeName::new("Database")
    }
}

#[derive(
    Debug,
    Default,
    Clone,
    Deserialize,
    Serialize,
    Decode,
    Encode,
    Hash,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
)]
#[serde(rename_all = "camelCase")]
pub struct OldFileModify {
    pub file: String,
    pub modified: u128,
    pub scan_time: u128,
}

#[derive(Debug, Clone, Deserialize, Default, Serialize, Decode, Encode, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Album {
    pub id: ArrayString<64>,
    pub title: Option<String>,
    pub created_time: u128,
    pub start_time: Option<u128>,
    pub end_time: Option<u128>,
    pub last_modified_time: u128,
    pub cover: Option<ArrayString<64>>,
    pub thumbhash: Option<Vec<u8>>,
    pub user_defined_metadata: HashMap<String, Vec<String>>,
    pub share_list: HashMap<ArrayString<64>, OldShare>,
    pub tag: HashSet<String>,
    pub width: u32,
    pub height: u32,
    pub item_count: usize,
    pub item_size: u64,
    pub pending: bool,
}

impl Value for Album {
    type SelfType<'a> = Self;
    type AsBytes<'a> = Vec<u8>;

    fn fixed_width() -> Option<usize> {
        None
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        bitcode::decode(data).expect("Corrupt Data: Failed to decode OldAlbum via bitcode")
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a> {
        bitcode::encode(value)
    }

    fn type_name() -> TypeName {
        TypeName::new("Album")
    }
}

#[derive(
    Debug, Clone, Deserialize, Default, Serialize, Decode, Encode, PartialEq, Eq, Hash,
)]
#[serde(rename_all = "camelCase")]
pub struct OldShare {
    pub url: ArrayString<64>,
    pub description: String,
    pub password: Option<String>,
    pub show_metadata: bool,
    pub show_download: bool,
    pub show_upload: bool,
    pub exp: u64,
}
