use arrayvec::ArrayString;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Default, Serialize, Decode, Encode, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct Share {
    pub url: ArrayString<64>,
    pub description: String,
    pub password: Option<String>,
    pub show_metadata: bool,
    pub show_download: bool,
    pub show_upload: bool,
    pub exp: i64,
}

#[derive(Debug, Clone, Deserialize, Default, Serialize, Decode, Encode, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedShare {
    pub share: Share,
    pub album_id: ArrayString<64>,
    pub album_title: Option<String>,
}

impl ResolvedShare {
    pub fn new(album_id: ArrayString<64>, album_title: Option<String>, share: Share) -> Self {
        Self {
            share,
            album_id,
            album_title,
        }
    }
}

pub fn resolve_show_download_and_metadata(
    resolved_share_opt: Option<ResolvedShare>,
) -> (bool, bool) {
    resolved_share_opt.map_or((true, true), |resolved_share| {
        (
            resolved_share.share.show_download,
            resolved_share.share.show_metadata,
        )
    })
}
