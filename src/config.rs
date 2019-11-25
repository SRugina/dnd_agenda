/// js toISOString() in test suit can't handle chrono's default precision
pub const DATE_FORMAT: &str = "%FT%H:%M:%S%.3f%:z";

pub static JWT_SECRET: &str = dotenv!("JWT_SECRET");

pub const TOKEN_PREFIX: &str = "Token ";

pub const DEFAULT_LIMIT: i64 = 20;

pub const DATABASE_URL: &str = dotenv!("DATABASE_URL");
