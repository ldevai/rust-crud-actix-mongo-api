use log::info;
use mongodb::{options::ClientOptions, sync::Client, sync::Database};
use mongodb::options::Credential;

#[derive(Clone)]
pub struct Environment {
    db_pool: Client,
    config: Config,
}

#[derive(Clone, Debug)]
pub struct Config {
    db_name: String,
    pub host: String,
    // ... other properties to be used throughout the app
}

impl Environment {
    pub async fn new() -> Self {
        info!("[environment] Initializing environment");
        dotenv::dotenv().ok();
        log4rs::init_file("logconfig.yml", Default::default()).expect("Log config file not found.");

        let db_url = std::env::var("DB_URL").unwrap();
        let db_name = std::env::var("DB_NAME").unwrap();
        let db_username = std::env::var("DB_USER").unwrap();
        let db_password = std::env::var("DB_PASSWORD").unwrap();
        let host = std::env::var("HOST").unwrap();

        let config = Config {
            db_name: db_name.clone(),
            host,
        };

        info!("[environment] Connecting to MongoDB at db_url={:?}, db_name={:?}", &db_url, &db_name);

        let mut db_config = ClientOptions::parse(db_url).unwrap();
        db_config.app_name = Some(String::from(&db_name));
        db_config.server_selection_timeout = Some(std::time::Duration::new(5, 0));
        db_config.credential = Some(Credential::builder().username(db_username).password(db_password).build());
        let db_pool = Client::with_options(db_config).unwrap();

        Self {
            db_pool,
            config,
        }
    }

    pub fn db(&self) -> Database {
        let db = self.db_pool.clone().database(&self.config.db_name);
        return db;
    }

    pub fn config(&self) -> &Config { &self.config }
}
