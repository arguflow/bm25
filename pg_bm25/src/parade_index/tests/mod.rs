#[pgrx::pg_schema]
mod tests {
    use pgrx::*;
    use shared::testing::dblink;

    const SETUP_SQL: &str = include_str!("tokenizer_chinese_compatible_setup.sql");

    #[pgrx::pg_test]
    fn test_chinese_compatible_tokenizer_in_new_connection() {
        // In this test, the index is created and the tokenizer is used in separate connections.
        // Because we retrieve the index from disk for new connections, we want to make
        // sure that the tokenizers are set up properly. We're going to make use of a
        // Postgres extension that lets us create 'sub-connections' to the database.

        // Create the dblink extension if it doesn't already exist.
        // dblink allows us to establish a 'sub-connection' to the current database
        // and execute queries. This is necessary, because the test context otherwise
        // is executed within a single Postgres transaction.
        Spi::run("CREATE EXTENSION IF NOT EXISTS dblink").expect("error creating dblink extension");

        // Set up the test environment using dblink to run the setup SQL in a separate connection.
        // The setup SQL is expected to prepare the database with the necessary configuration for the tokenizer.
        let setup_query = format!("SELECT * FROM {} AS (_ text)", &dblink(SETUP_SQL));
        Spi::run(&setup_query).expect("error running dblink setup query");
    }
}
