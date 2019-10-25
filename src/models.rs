use crate::schema::{images, pastes};

use chrono::NaiveDateTime;
use diesel::{
	dsl::Eq, insertable::Insertable, pg::Pg, prelude::*, query_builder::UndecoratedInsertRecord,
	sql_types::*, Queryable,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug)]
pub struct Paste {
	pub id: i64,
	pub filename: Option<String>,
	pub content: Option<String>,
	pub creation_date: NaiveDateTime,
}

impl Queryable<pastes::SqlType, Pg> for Paste {
	type Row = (i64, Option<String>, Option<String>, NaiveDateTime);

	fn build(row: Self::Row) -> Self {
		let (id, filename, content, creation_date) = row;
		Paste {
			id,
			filename,
			content,
			creation_date,
		}
	}
}

impl Queryable<(BigInt, Nullable<Text>, Timestamp), Pg> for Paste {
	type Row = (i64, Option<String>, NaiveDateTime);

	fn build(row: Self::Row) -> Self {
		let (id, filename, creation_date) = row;
		Paste {
			id,
			filename,
			content: None,
			creation_date,
		}
	}
}

impl<'a> Insertable<pastes::table> for Paste {
	type Values = <(
		Option<Eq<pastes::filename, String>>,
		Option<Eq<pastes::content, String>>,
		Option<Eq<pastes::creation_date, NaiveDateTime>>,
	) as Insertable<pastes::table>>::Values;

	fn values(self) -> Self::Values {
		(
			self.filename.map(|x| pastes::filename.eq(x)),
			self.content.map(|x| pastes::content.eq(x)),
			Some(pastes::creation_date.eq(self.creation_date)),
		)
			.values()
	}
}

impl<'a> Insertable<pastes::table> for &'a Paste {
	type Values = <(
		Option<Eq<pastes::filename, &'a String>>,
		Option<Eq<pastes::content, &'a String>>,
		Option<Eq<pastes::creation_date, &'a NaiveDateTime>>,
	) as Insertable<pastes::table>>::Values;

	fn values(self) -> Self::Values {
		(
			self.filename.as_ref().map(|x| pastes::filename.eq(x)),
			self.content.as_ref().map(|x| pastes::content.eq(x)),
			Some(pastes::creation_date.eq(&self.creation_date)),
		)
			.values()
	}
}

impl<'a> UndecoratedInsertRecord<pastes::table> for Paste {}

#[derive(Deserialize, Debug)]
pub struct PasteForm {
	pub filename: Option<String>,
	pub content: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct Image {
	pub id: i64,
	pub filename: String,
	pub creation_date: NaiveDateTime,
	pub content: Vec<u8>,
}

impl Queryable<images::SqlType, Pg> for Image {
	type Row = (i64, String, NaiveDateTime, Vec<u8>);

	fn build(row: Self::Row) -> Self {
		let (id, filename, creation_date, content) = row;
		Image {
			id,
			filename,
			creation_date,
			content,
		}
	}
}

impl Queryable<(BigInt, Text, Timestamp), Pg> for Image {
	type Row = (i64, String, NaiveDateTime);

	fn build(row: Self::Row) -> Self {
		let (id, filename, creation_date) = row;
		Image {
			id,
			filename,
			creation_date,
			content: vec![],
		}
	}
}

#[derive(Insertable, Debug)]
#[table_name = "images"]
pub struct NewImage {
	pub filename: String,
	pub creation_date: NaiveDateTime,
	pub content: Vec<u8>,
}
