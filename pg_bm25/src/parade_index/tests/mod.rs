use pgrx::Spi;

pub fn dblink(query: &str) -> String {
    // Retrieve the current database name from the PostgreSQL environment.
    let current_db_name: String = Spi::get_one("SELECT current_database()::text")
        .expect("couldn't get current database for postgres connection")
        .unwrap();

    // Retrieve the current port number on which the PostgreSQL server is listening.
    let current_port: u32 =
        Spi::get_one::<String>("SELECT setting FROM pg_settings WHERE name = 'port'")
            .expect("couldn't get current port for postgres connection")
            .unwrap()
            .parse()
            .expect("couldn't parse current port into u32");

    // Prepare the connection string for dblink. This string contains the host (assumed to be
    // localhost in this function), the port number, and the database name to connect to.
    let connection_string = format!(
        "host=localhost port={} dbname={}",
        current_port, current_db_name
    );

    // Escape single quotes in the SQL query since it will be nested inside another SQL string
    // in the dblink function call. Single quotes in SQL strings are escaped by doubling them.
    let escaped_query_string = query.replace('\'', "''");

    // Construct the dblink function call with the connection string and the escaped query.
    // This function call is what can be executed within a PostgreSQL environment.
    format!("dblink('{connection_string}', '{escaped_query_string}')")
}

#[pgrx::pg_schema]
mod tests {
    use pgrx::*;

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
        let setup_query = format!("SELECT * FROM {} AS (_ text)", &super::dblink(SETUP_SQL));
        Spi::run(&setup_query).expect("error running dblink setup query");
    }
}
