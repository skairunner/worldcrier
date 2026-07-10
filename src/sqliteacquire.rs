use sqlx::Sqlite;

// Little bit of magic to make a shorthand for sqlx::Acquire
pub trait SqliteAcquire<'a>: sqlx::Acquire<'a, Database = Sqlite> {}
impl<'a, T: sqlx::Acquire<'a, Database = Sqlite>> SqliteAcquire<'a> for T {}
