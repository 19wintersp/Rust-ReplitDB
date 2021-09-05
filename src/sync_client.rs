use crate::get_url_from_env;

use std::collections::HashMap;

use reqwest::blocking::Client as HttpClient;

use urlencoding::{ encode, decode };

/// A blocking client.
/// 
/// Provides synchronous functions for interacting with the database.
/// For an asynchronous, non-blocking client, use [`AsyncClient`](super::AsyncClient).
/// 
/// ```
/// let client = replitdb::SyncClient::new();
/// client.set("greeting", "hello world")?;
/// println!("{}", client.get("greeting")?);
/// ```
#[derive(Clone, Debug)]
pub struct Client {
	url: String,
	client: HttpClient,
}

impl Client {
	/// Create a new synchronous client, fetching the URL from an environment variable.
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

	/// Create a new synchronous client, specifying a custom database URL.
	pub fn new_url(url: impl Into<String>) -> Self {
		Client {
			url: url.into(),
			client: HttpClient::new(),
		}
	}

	/// Get the value of the specified key. Returns `Ok(None)` if the key does not exist.
	pub fn get(
		&self,
		key: impl Into<String>,
	) -> Result<Option<String>, String> {
		let encoded_key = encode(key.into().as_str()).into_owned();

		let response = self.client.get(format!("{}/{}", self.url, encoded_key))
			.send()
			.map_err(|err| err.to_string())?;

		if response.status().is_success() {
			Ok(Some(
				response.text()
					.map_err(|err| err.to_string())?
			))
		} else if response.status().as_u16() == 404 {
			Ok(None)
		} else {
			Err(
				response.text()
					.map_err(|err| err.to_string())?
			)
		}
	}

	/// Set the value of the specified key to the provided value.
	pub fn set(
		&self,
		key: impl Into<String>,
		value: impl Into<String>,
	) -> Result<(), String> {
		let encoded_key = encode(key.into().as_str()).into_owned();
		let encoded_value = encode(value.into().as_str()).into_owned();

		let response = self.client.post(self.url.clone())
			.body(format!("{}={}", encoded_key, encoded_value))
			.header("Content-Type", "application/x-www-form-urlencoded")
			.send()
			.map_err(|err| err.to_string())?;
		
		if response.status().is_success() {
			Ok(())
		} else {
			Err(
				response.text()
					.map_err(|err| err.to_string())?
			)
		}
	}

	/// Delete the specified key from the database.
	pub fn delete(&self, key: impl Into<String>) -> Result<(), String> {
		let encoded_key = encode(key.into().as_str()).into_owned();

		let response = self.client.delete(format!("{}/{}", self.url, encoded_key))
			.send()
			.map_err(|err| err.to_string())?;
		
		if response.status().is_success() || response.status().as_u16() == 404 {
			Ok(())
		} else {
			Err(
				response.text()
					.map_err(|err| err.to_string())?
			)
		}
	}

	/// List all keys in the database.
	pub fn list(&self) -> Result<Vec<String>, String> {
		self.list_prefix("")
	}

	/// List all keys in the database that start with the specified prefix.
	pub fn list_prefix(
		&self,
		prefix: impl Into<String>,
	) -> Result<Vec<String>, String> {
		let response = self.client.get(self.url.clone())
			.query(&[ ("encode", "true"), ("prefix", prefix.into().as_str()) ])
			.send()
			.map_err(|err| err.to_string())?;
		
		if response.status().is_success() {
			let text = response.text()
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
					.map_err(|err| err.to_string())?
			)
		}
	}

	/// Delete all keys in the database.
	pub fn empty(&self) -> Result<(), String> {
		// this could probably be improved

		let keys = self.list()?;
		for key in keys {
			self.delete(key)?;
		}

		Ok(())
	}

	/// Get all key-value pairs and return them as a [`HashMap`](std::collections::HashMap).
	pub fn get_all(&self) -> Result<HashMap<String, String>, String> {
		// this could probably be improved

		let mut out = HashMap::new();

		let keys = self.list()?;
		for key in keys {
			let value = self.get(key.clone())?;
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
