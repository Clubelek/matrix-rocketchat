use diesel::Connection;
use diesel::migrations::setup_database;
use diesel::sqlite::SqliteConnection;
use embedded_migrations::run as run_embedded_migrations;
use iron::{Chain, Iron, Listening};
use persistent::State;
use router::Router;
use slog::Logger;

use config::Config;
use errors::*;
use handlers::{Transactions, Welcome};
use log::IronLogger;

/// The application service server
pub struct Server<'a> {
    /// Application service configuration
    config: &'a Config,
    /// Logger passed to the server
    logger: Logger,
}

impl<'a> Server<'a> {
    /// Create a new server with a given configuration.
    pub fn new(config: &Config, logger: Logger) -> Server {
        Server {
            config: config,
            logger: logger,
        }
    }

    /// Runs the application service bridge.
    pub fn run(&self) -> Result<Listening> {
        self.prepare_database().chain_err(|| "Database setup failed")?;
        info!(self.logger, "Starting server"; "address" => format!("{:?}", self.config.as_address));
        let router = self.setup_routes();

        let mut chain = Chain::new(router);
        chain.link_before(State::<IronLogger>::one::<Logger>(self.logger.clone()));
        Iron::new(chain).http(self.config.as_address).chain_err(|| "Unable to start server")
    }

    fn setup_routes(&self) -> Router {
        debug!(self.logger, "Setting up routes");
        let mut router = Router::new();
        router.get("/", Welcome {});
        router.put("/transactions/:txn_id", Transactions::chain(self.config.clone()));

        router
    }

    fn prepare_database(&self) -> Result<()> {
        let connection =
            SqliteConnection::establish(&self.config.database_url).chain_err(|| "Could not establish database connection")?;
        setup_database(&connection).chain_err(|| "Could not setup database")?;
        run_embedded_migrations(&connection).chain_err(|| "Running migrations failed")
    }
}
