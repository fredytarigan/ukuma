use tracing::{Level, info};
use tracing_subscriber::{Registry, fmt::writer::MakeWriterExt, layer::SubscriberExt};

use crate::Logger;

/* logger implementation */
impl Logger {   
    pub fn setup(&self) -> Result<(), std::io::Error> {
        color_eyre::install().expect("Failed to install color_eyre");

        info!("Configuring logger with level {}", self.level);

        let stdout = tracing_subscriber::fmt::layer()
            .with_writer(std::io::stdout.with_max_level(self.level))
            .with_file(true)
            .with_line_number(true)
            .with_target(true)
            .json();

        let subscriber = Registry::default().with(stdout);

        tracing::subscriber::set_global_default(subscriber)
            .expect("Unable to set default subscriber");

        Ok(())
    }
}
