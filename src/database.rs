use rocket_contrib::databases::diesel::PgConnection;
#[database("dnd_agenda")]
pub struct DnDAgendaDB(PgConnection);

use diesel::prelude::*;
use crate::config;

pub fn establish_connection() -> PgConnection {

    PgConnection::establish(config::DATABASE_URL)
        .unwrap_or_else(|_| panic!("Error connecting to {}", config::DATABASE_URL))
}