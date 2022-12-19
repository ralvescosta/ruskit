use env::{Config, Environment};
use errors::logging::LoggingError;
use tracing_bunyan_formatter::BunyanFormattingLayer;
use tracing_log::LogTracer;
use tracing_subscriber::{
    filter::{LevelFilter, Targets},
    fmt::{
        format::{Format, Pretty},
        Layer,
    },
    layer::SubscriberExt,
};

pub fn setup(cfg: &Config) -> Result<(), LoggingError> {
    LogTracer::init().map_err(|_| LoggingError::InternalError {})?;

    let level_filter = get_log_level_filter(cfg);

    let mut target_filters = Targets::new().with_default(level_filter);
    if !cfg.app.enable_external_creates_logging {
        target_filters = Targets::new()
            .with_default(level_filter)
            .with_target("lapin::channels", LevelFilter::WARN)
            .with_target("tower::buffer::worker", LevelFilter::WARN)
            .with_target("h2::client", LevelFilter::WARN)
            .with_target("h2::codec::framed_read", LevelFilter::WARN)
            .with_target("h2::codec::framed_write", LevelFilter::WARN)
            .with_target("h2::proto::settings", LevelFilter::WARN)
            .with_target("hyper::client::connect::dns", LevelFilter::WARN)
            .with_target("hyper::client::connect::http", LevelFilter::WARN)
            .with_target("rustls::client::hs", LevelFilter::WARN)
            .with_target("rustls::anchors", LevelFilter::WARN)
            .with_target("rustls::client::tls13", LevelFilter::WARN)
            .with_target("paho_mqtt::async_client", LevelFilter::WARN);
    }

    let mut fmt_pretty: Option<Layer<_, Pretty, Format<Pretty>>> = None;
    let mut fmt_json = None;

    if cfg.app.env == Environment::Local {
        fmt_pretty = Some(Layer::new().pretty());
    } else {
        fmt_json = Some(BunyanFormattingLayer::new(
            cfg.app.name.to_owned(),
            std::io::stdout,
        ));
    }

    tracing::subscriber::set_global_default(
        tracing_subscriber::registry()
            .with(fmt_json)
            .with(fmt_pretty)
            .with(target_filters),
    )
    .map_err(|_| LoggingError::InternalError {})?;

    Ok(())
}

fn get_log_level_filter(cfg: &Config) -> LevelFilter {
    match cfg.app.log_level.as_str() {
        "debug" | "Debug" | "DEBUG" => LevelFilter::DEBUG,
        "info" | "Info" | "INFO" => LevelFilter::INFO,
        "warn" | "Warn" | "WARN" => LevelFilter::WARN,
        "error" | "Error" | "ERROR" => LevelFilter::ERROR,
        "trace" | "Trace" | "TRACE" => LevelFilter::TRACE,
        _ => LevelFilter::OFF,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn setup_successfully() {
        let res = setup(&Config::default());
        assert!(res.is_ok());
    }

    #[test]
    fn get_log_level_successfully() {
        let mut cfg = Config::default();

        cfg.app.log_level = "debug".to_owned();
        assert_eq!(get_log_level_filter(&cfg), LevelFilter::DEBUG);
        cfg.app.log_level = "Debug".to_owned();
        assert_eq!(get_log_level_filter(&cfg), LevelFilter::DEBUG);
        cfg.app.log_level = "DEBUG".to_owned();
        assert_eq!(get_log_level_filter(&cfg), LevelFilter::DEBUG);

        cfg.app.log_level = "info".to_owned();
        assert_eq!(get_log_level_filter(&cfg), LevelFilter::INFO);
        cfg.app.log_level = "Info".to_owned();
        assert_eq!(get_log_level_filter(&cfg), LevelFilter::INFO);
        cfg.app.log_level = "INFO".to_owned();
        assert_eq!(get_log_level_filter(&cfg), LevelFilter::INFO);

        cfg.app.log_level = "warn".to_owned();
        assert_eq!(get_log_level_filter(&cfg), LevelFilter::WARN);
        cfg.app.log_level = "Warn".to_owned();
        assert_eq!(get_log_level_filter(&cfg), LevelFilter::WARN);
        cfg.app.log_level = "WARN".to_owned();
        assert_eq!(get_log_level_filter(&cfg), LevelFilter::WARN);

        cfg.app.log_level = "error".to_owned();
        assert_eq!(get_log_level_filter(&cfg), LevelFilter::ERROR);
        cfg.app.log_level = "Error".to_owned();
        assert_eq!(get_log_level_filter(&cfg), LevelFilter::ERROR);
        cfg.app.log_level = "ERROR".to_owned();
        assert_eq!(get_log_level_filter(&cfg), LevelFilter::ERROR);

        cfg.app.log_level = "trace".to_owned();
        assert_eq!(get_log_level_filter(&cfg), LevelFilter::TRACE);
        cfg.app.log_level = "Trace".to_owned();
        assert_eq!(get_log_level_filter(&cfg), LevelFilter::TRACE);
        cfg.app.log_level = "TRACE".to_owned();
        assert_eq!(get_log_level_filter(&cfg), LevelFilter::TRACE);

        cfg.app.log_level = "UNKNOWN".to_owned();
        assert_eq!(get_log_level_filter(&cfg), LevelFilter::OFF);
    }
}
