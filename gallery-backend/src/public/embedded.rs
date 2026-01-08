use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "../gallery-frontend/dist/"]
pub struct FrontendAssets;
