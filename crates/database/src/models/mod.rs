mod downloads;
mod oidc_mapping;
mod user;
mod user_session;
mod videos;

pub use downloads::{Download, DownloadStatus};
pub use oidc_mapping::OidcMapping;
pub use user::User;
pub use user_session::UserSession;
pub use videos::Video;

type Result<T> = std::result::Result<T, diesel::result::Error>;
