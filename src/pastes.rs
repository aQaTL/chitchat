use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Paste {
	pub author: String,
	pub title: String,
	pub content: String,
}
