#![allow(clippy::struct_excessive_bools)]
use arrayvec::ArrayString;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use redb::{TypeName, Value};

// ==================================================================================
// Object Type & Schema
// ==================================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
#[serde(rename_all = "camelCase")]
pub enum ObjectType {
    Image,
    Video,
    Album,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
#[serde(rename_all = "camelCase")]
pub struct ObjectSchema {
    pub id: ArrayString<64>,
    pub obj_type: ObjectType,
    pub pending: bool,
    pub thumbhash: Option<Vec<u8>>,
    pub description: Option<String>,
    pub tags: HashSet<String>,
    pub is_favorite: bool,
    pub is_archived: bool,
    pub is_trashed: bool,
}

// ==================================================================================
// File & Share Structures
// ==================================================================================

#[derive(Debug, Default, Clone, Deserialize, Serialize, Decode, Encode, Hash, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FileModify {
    pub file: String,
    pub modified: u128,
    pub scan_time: u128,
}

#[derive(Debug, Clone, Deserialize, Default, Serialize, Decode, Encode, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct Share {
    pub url: ArrayString<64>,
    pub description: String,
    pub password: Option<String>,
    pub show_metadata: bool,
    pub show_download: bool,
    pub show_upload: bool,
    pub exp: u64,
}

// ==================================================================================
// Metadata Structures
// ==================================================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
#[serde(rename_all = "camelCase")]
pub struct ImageMetadata {
    pub id: ArrayString<64>,
    pub size: u64,
    pub width: u32,
    pub height: u32,
    pub ext: String,
    pub phash: Option<Vec<u8>>,
    pub albums: HashSet<ArrayString<64>>,
    pub exif_vec: BTreeMap<String, String>,
    pub alias: Vec<FileModify>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Encode, Decode)]
#[serde(rename_all = "camelCase")]
pub struct VideoMetadata {
    pub id: ArrayString<64>,
    pub size: u64,
    pub width: u32,
    pub height: u32,
    pub ext: String,
    pub duration: f64,
    pub albums: HashSet<ArrayString<64>>,
    pub exif_vec: BTreeMap<String, String>,
    pub alias: Vec<FileModify>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
#[serde(rename_all = "camelCase")]
pub struct AlbumMetadata {
    pub id: ArrayString<64>,
    pub title: Option<String>,
    pub created_time: i64,
    pub start_time: Option<i64>,
    pub end_time: Option<i64>,
    pub last_modified_time: i64,
    pub cover: Option<ArrayString<64>>,
    pub item_count: usize,
    pub item_size: u64,
    pub share_list: HashMap<ArrayString<64>, Share>,
}

// ==================================================================================
// Combined Structures
// ==================================================================================

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
#[serde(rename_all = "camelCase")]
pub struct ImageCombined {
    #[serde(flatten)]
    pub object: ObjectSchema,
    #[serde(flatten)]
    pub metadata: ImageMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
#[serde(rename_all = "camelCase")]
pub struct VideoCombined {
    #[serde(flatten)]
    pub object: ObjectSchema,
    #[serde(flatten)]
    pub metadata: VideoMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
#[serde(rename_all = "camelCase")]
pub struct AlbumCombined {
    #[serde(flatten)]
    pub object: ObjectSchema,
    #[serde(flatten)]
    pub metadata: AlbumMetadata,
}

// ==================================================================================
// AbstractData Enum
// ==================================================================================

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum AbstractData {
    Image(ImageCombined),
    Video(VideoCombined),
    Album(AlbumCombined),
}

// Implement Value trait for Redb compatibility
impl Value for AbstractData {
    type SelfType<'a> = Self;
    type AsBytes<'a> = Vec<u8>;

    fn fixed_width() -> Option<usize> {
        None
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        bitcode::decode(data)
            .expect("Corrupt Data: Failed to decode Legacy AbstractData via bitcode")
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a> {
        bitcode::encode(value)
    }

    fn type_name() -> TypeName {
        TypeName::new("AbstractData")
    }
}
