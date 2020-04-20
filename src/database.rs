// TODO: remove once clippy allows disabling single_component_path_import within #[derive(...)]
#![allow(clippy::single_component_path_imports)]

use rocket_contrib::databases::diesel::PgConnection;
#[database("dnd_agenda")]
pub struct DnDAgendaDB(PgConnection);

use crate::config;
use crate::config::DEFAULT_LIMIT;
use diesel::prelude::*;

pub fn establish_connection() -> PgConnection {
    PgConnection::establish(config::DATABASE_URL)
        .unwrap_or_else(|_| panic!("Error connecting to {}", config::DATABASE_URL))
}

pub mod functions {
    use diesel::sql_types::*;

    sql_function! {
        /// Represents the `SIMILARITY` SQL function from pg_trgm
        #[sql_name = "SIMILARITY"]
        fn similarity(a: Text, b: Text) -> Float;
    }
}

pub mod helper_types {
    #[allow(dead_code)]
    /// The return type of `similarity(expr, expr)`
    pub type Similarity<Expr1, Expr2> = super::functions::similarity::HelperType<Expr1, Expr2>;
}

pub mod operators {
    use diesel::pg::Pg;

    
    diesel_infix_operator!(SimilarTo, " %> ", backend: Pg);

    use diesel::expression::AsExpression;
    use diesel::prelude::*;

    // Normally you would put this on a trait instead.

    /// usage: `.filter(similar_to(username, "bob"))`
    pub fn similar_to<T, U>(left: T, right: U) -> SimilarTo<T, U::Expression>
    where
        T: Expression,
        U: AsExpression<T::SqlType>,
    {
        SimilarTo::new(left, right.as_expression())
    }
}

pub mod dsl {
    pub use super::functions::*;
    pub use super::helper_types::*;
    pub use super::operators::*;
}

use diesel::pg::Pg;
use diesel::query_builder::*;
use diesel::query_dsl::methods::LoadQuery;
use diesel::sql_types::BigInt;

pub trait Paginate: Sized {
    fn paginate(self, page: i64) -> Paginated<Self>;
}

impl<T> Paginate for T {
    fn paginate(self, page: i64) -> Paginated<Self> {
        Paginated {
            query: self,
            per_page: DEFAULT_LIMIT,
            page,
        }
    }
}


#[derive(Debug, Clone, Copy, QueryId)]
pub struct Paginated<T> {
    query: T,
    page: i64,
    per_page: i64,
}

impl<T> Paginated<T> {
    pub fn per_page(self, per_page: i64) -> Self {
        Paginated { per_page, ..self }
    }

    pub fn load_and_count_pages<U>(self, conn: &PgConnection) -> QueryResult<(Vec<U>, i64)>
    where
        Self: LoadQuery<PgConnection, (U, i64)>,
    {
        let per_page = self.per_page;
        let results = self.load::<(U, i64)>(conn)?;
        let total = results.get(0).map(|x| x.1).unwrap_or(0);
        let records = results.into_iter().map(|x| x.0).collect();
        let total_pages = (total as f64 / per_page as f64).ceil() as i64;
        Ok((records, total_pages))
    }
}

impl<T: Query> Query for Paginated<T> {
    type SqlType = (T::SqlType, BigInt);
}

impl<T> RunQueryDsl<PgConnection> for Paginated<T> {}

impl<T> QueryFragment<Pg> for Paginated<T>
where
    T: QueryFragment<Pg>,
{
    fn walk_ast(&self, mut out: AstPass<Pg>) -> QueryResult<()> {
        out.push_sql("SELECT *, COUNT(*) OVER () FROM (");
        self.query.walk_ast(out.reborrow())?;
        out.push_sql(") t LIMIT ");
        out.push_bind_param::<BigInt, _>(&self.per_page)?;
        out.push_sql(" OFFSET ");
        let offset = (self.page - 1) * self.per_page;
        out.push_bind_param::<BigInt, _>(&offset)?;
        Ok(())
    }
}
