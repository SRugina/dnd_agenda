use rocket_contrib::databases::diesel::PgConnection;
#[database("dnd_agenda")]
pub struct DnDAgendaDB(PgConnection);

use crate::config;
use diesel::prelude::*;

pub fn establish_connection() -> PgConnection {
    PgConnection::establish(config::DATABASE_URL)
        .unwrap_or_else(|_| panic!("Error connecting to {}", config::DATABASE_URL))
}
