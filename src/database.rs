use rocket_contrib::databases::diesel::PgConnection;
#[database("dnd_agenda")]
pub struct DnDAgendaDB(PgConnection);
