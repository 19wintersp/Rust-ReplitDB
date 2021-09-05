//! Simple Replit Database clients.
//!
//! Provides two clients to interact with Replit DB, [`AsyncClient`] and [`SyncClient`]. The API is modeled after [the NodeJS client].
//!
//! [the NodeJS client]: https://www.npmjs.com/package/@replit/database

mod async_client;
mod sync_client;

pub use async_client::Client as AsyncClient;
pub use sync_client::Client as SyncClient;

const URL_VAR: &str = "REPLIT_DB_URL";

fn get_url_from_env() -> String {
	std::env::var(URL_VAR).unwrap()
}
