/// js toISOString() in test suit can't handle chrono's default precision
pub const DATE_FORMAT: &str = "%Y-%m-%dT%H:%M:%S%.3fZ";

pub const SECRET: &str = "secret123";
pub const TOKEN_PREFIX: &str = "Token ";

pub const DATABASE_URL: &str = "postgres://dnd_agenda:dnd_agenda@localhost/dnd_agenda";