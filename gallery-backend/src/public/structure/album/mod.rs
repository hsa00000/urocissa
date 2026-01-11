#[allow(clippy::module_inception)]
pub mod album;
pub mod combined;
pub mod metadata;
pub mod share;

pub use album::Album;
pub use combined::AlbumCombined;
pub use share::{ResolvedShare, Share};
