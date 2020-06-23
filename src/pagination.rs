use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::query_builder::*;
use diesel::query_dsl::methods::LoadQuery;
use diesel::sql_types::BigInt;
use serde::Serialize;

pub trait Paginate: Sized {
	fn paginate(self, page: i64, per_page: i64) -> PaginatedQuery<Self>;
}

impl<T> Paginate for T {
	fn paginate(self, page: i64, per_page: i64) -> PaginatedQuery<Self> {
		PaginatedQuery {
			query: self,
			page,
			per_page,
		}
	}
}

#[derive(Clone, Copy, QueryId)]
pub struct PaginatedQuery<T> {
	query: T,
	page: i64,
	per_page: i64,
}

#[derive(Serialize, Debug)]
pub struct Paginated<T> {
	pub page: i64,
	pub total_pages: i64,
	pub results: Vec<T>,
}

impl<T> PaginatedQuery<T> {
	pub fn load_and_count_pages<U>(self, conn: &PgConnection) -> QueryResult<Paginated<U>>
	where
		Self: LoadQuery<PgConnection, (U, i64)>,
	{
		let per_page = self.per_page;
		let page = self.page;
		let results = self.load::<(U, i64)>(conn)?;
		let total = results.get(0).map(|x| x.1).unwrap_or(0);
		let results = results.into_iter().map(|x| x.0).collect();
		let total_pages = (total as f64 / per_page as f64).ceil() as i64;
		Ok(Paginated {
			page,
			total_pages,
			results,
		})
	}
}

impl<T: Query> Query for PaginatedQuery<T> {
	type SqlType = (T::SqlType, BigInt);
}

impl<T> RunQueryDsl<PgConnection> for PaginatedQuery<T> {}

impl<T> QueryFragment<Pg> for PaginatedQuery<T>
where
	T: QueryFragment<Pg>,
{
	fn walk_ast(&self, mut out: AstPass<Pg>) -> QueryResult<()> {
		out.push_sql("SELECT *, COUNT(*) OVER () FROM(");
		self.query.walk_ast(out.reborrow())?;
		out.push_sql(") t LIMIT ");
		out.push_bind_param::<BigInt, _>(&self.per_page)?;
		out.push_sql(" OFFSET ");
		let offset = (self.page - 1) * self.per_page;
		out.push_bind_param::<BigInt, _>(&offset)?;
		Ok(())
	}
}
