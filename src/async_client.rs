use crate::get_url_from_env;

use std::collections::HashMap;

use reqwest::Client as HttpClient;
use reqwest::multipart::Form;

use urlencoding::{ encode, decode };

/// An asynchronous client.
/// 
/// Provides asynchronous functions for interacting with the database.
/// For a synchronous, blocking client, use [`SyncClient`](super::SyncClient).
/// 
/// ```
/// let client = replitdb::AsyncClient::new();
/// client.set("greeting", "hello world").await?;
/// println!("{}", client.get("greeting").await?);
/// ```
#[derive(Clone, Debug)]
pub struct Client {
	url: String,
	client: HttpClient,
}

impl Client {
	/// Create a new asynchronous client, fetching the URL from an environment variable.
	/// 
	/// # Panics
	/// 
	/// Panics if the database URL environment variable (`REPLIT_DB_URL`) is not set.
	pub fn new() -> Self {
		Client {
			url: get_url_from_env(),
			client: HttpClient::new(),
		}
	}

	/// Create a new asynchronous client, specifying a custom database URL.
	pub fn new_url(url: impl Into<String>) -> Self {
		Client {
			url: url.into(),
			client: HttpClient::new(),
		}
	}

	/// Get the value of the specified key. Returns `Ok(None)` if the key does not exist.
	pub async fn get(
		self,
		key: impl Into<String>,
	) -> Result<Option<String>, String> {
		let encoded_key = encode(key.into().as_str()).into_owned();

		let response = self.client.get(format!("{}/{}", self.url, encoded_key))
			.send()
			.await
			.map_err(|err| err.to_string())?;

		if response.status().is_success() {
			Ok(Some(
				response.text()
					.await
					.map_err(|err| err.to_string())?
			))
		} else if response.status().as_u16() == 404 {
			Ok(None)
		} else {
			Err(
				response.text()
					.await
					.map_err(|err| err.to_string())?
			)
		}
	}

	/// Set the value of the specified key to the provided value.
	pub async fn set(
		self,
		key: impl Into<String>,
		value: impl Into<String>,
	) -> Result<(), String> {
		let form = Form::new()
			.text(key.into(), value.into());

		let response = self.client.post(self.url)
			.multipart(form)
			.send()
			.await
			.map_err(|err| err.to_string())?;
		
		if response.status().is_success() {
			Ok(())
		} else {
			Err(
				response.text()
					.await
					.map_err(|err| err.to_string())?
			)
		}
	}

	/// Delete the specified key from the database.
	pub async fn delete(self, key: impl Into<String>) -> Result<(), String> {
		let encoded_key = encode(key.into().as_str()).into_owned();

		let response = self.client.delete(format!("{}/{}", self.url, encoded_key))
			.send()
			.await
			.map_err(|err| err.to_string())?;
		
		if response.status().is_success() || response.status().as_u16() == 404 {
			Ok(())
		} else {
			Err(
				response.text()
					.await
					.map_err(|err| err.to_string())?
			)
		}
	}

	/// List all keys in the database.
	pub async fn list(self) -> Result<Vec<String>, String> {
		self.list_prefix("").await
	}

	/// List all keys in the database that start with the specified prefix.
	pub async fn list_prefix(
		self,
		prefix: impl Into<String>,
	) -> Result<Vec<String>, String> {
		let response = self.client.get(self.url)
			.query(&[ ("encode", "true"), ("prefix", prefix.into().as_str()) ])
			.send()
			.await
			.map_err(|err| err.to_string())?;
		
		if response.status().is_success() {
			let text = response.text()
				.await
				.map_err(|err| err.to_string())?;

			Ok(
				text.split("\n")
					.map(|key| Ok(
						decode(key.into())
							.map_err(|err| err.to_string())?
							.into_owned()
					))
					.collect::<Result<Vec<String>, String>>()?
			)
		} else {
			Err(
				response.text()
					.await
					.map_err(|err| err.to_string())?
			)
		}
	}

	/// Delete all keys in the database.
	pub async fn empty(self) -> Result<(), String> {
		// this could probably be improved

		let keys = self.clone().list().await?;
		for key in keys {
			self.clone().delete(key).await?;
		}

		Ok(())
	}

	/// Get all key-value pairs and return them as a [`HashMap`](std::collections::HashMap).
	pub async fn get_all(self) -> Result<HashMap<String, String>, String> {
		// this could probably be improved

		let mut out = HashMap::new();

		let keys = self.clone().list().await?;
		for key in keys {
			let value = self.clone().get(key.clone()).await?;
			out.insert(key, value.unwrap());
		}

		Ok(out)
	}
}

impl Default for Client {
	fn default() -> Self {
		Self::new()
	}
}
