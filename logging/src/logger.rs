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

    let (non_blocking_writer, _guard) = tracing_appender::non_blocking(std::io::stdout());

    let level_filter = get_log_level_filter(cfg);

    let mut target_filters = Targets::new().with_default(level_filter);
    if !cfg.enable_external_creates_logging {
        target_filters = Targets::new()
            .with_default(level_filter)
            .with_target("rumqttc::state", LevelFilter::WARN)
            .with_target("lapin::channels", LevelFilter::WARN)
            .with_target("tower::buffer::worker", LevelFilter::WARN)
            .with_target("h2::codec::framed_read", LevelFilter::WARN)
            .with_target("h2::codec::framed_write", LevelFilter::WARN)
            .with_target("h2::proto::settings", LevelFilter::WARN)
            .with_target("hyper::client::connect::dns", LevelFilter::WARN)
            .with_target("hyper::client::connect::http", LevelFilter::WARN)
            .with_target("rustls::client::hs", LevelFilter::WARN)
            .with_target("rustls::anchors", LevelFilter::WARN)
            .with_target("rustls::client::tls13", LevelFilter::WARN);
    }

    let mut fmt_pretty: Option<Layer<_, Pretty, Format<Pretty>>> = None;
    let mut fmt_json = None;

    if cfg.env == Environment::Local {
        fmt_pretty = Some(Layer::new().pretty());
    } else {
        fmt_json = Some(BunyanFormattingLayer::new(
            cfg.app_name.to_owned(),
            non_blocking_writer,
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
    match cfg.log_level {
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
        let res = setup(&Config::mock());
        assert!(res.is_ok());
    }

    #[test]
    fn get_log_level_successfully() {
        let mut cfg = Config::mock();

        cfg.log_level = "debug";
        assert_eq!(get_log_level_filter(&cfg), LevelFilter::DEBUG);
        cfg.log_level = "Debug";
        assert_eq!(get_log_level_filter(&cfg), LevelFilter::DEBUG);
        cfg.log_level = "DEBUG";
        assert_eq!(get_log_level_filter(&cfg), LevelFilter::DEBUG);

        cfg.log_level = "info";
        assert_eq!(get_log_level_filter(&cfg), LevelFilter::INFO);
        cfg.log_level = "Info";
        assert_eq!(get_log_level_filter(&cfg), LevelFilter::INFO);
        cfg.log_level = "INFO";
        assert_eq!(get_log_level_filter(&cfg), LevelFilter::INFO);

        cfg.log_level = "warn";
        assert_eq!(get_log_level_filter(&cfg), LevelFilter::WARN);
        cfg.log_level = "Warn";
        assert_eq!(get_log_level_filter(&cfg), LevelFilter::WARN);
        cfg.log_level = "WARN";
        assert_eq!(get_log_level_filter(&cfg), LevelFilter::WARN);

        cfg.log_level = "error";
        assert_eq!(get_log_level_filter(&cfg), LevelFilter::ERROR);
        cfg.log_level = "Error";
        assert_eq!(get_log_level_filter(&cfg), LevelFilter::ERROR);
        cfg.log_level = "ERROR";
        assert_eq!(get_log_level_filter(&cfg), LevelFilter::ERROR);

        cfg.log_level = "trace";
        assert_eq!(get_log_level_filter(&cfg), LevelFilter::TRACE);
        cfg.log_level = "Trace";
        assert_eq!(get_log_level_filter(&cfg), LevelFilter::TRACE);
        cfg.log_level = "TRACE";
        assert_eq!(get_log_level_filter(&cfg), LevelFilter::TRACE);

        cfg.log_level = "UNKNOWN";
        assert_eq!(get_log_level_filter(&cfg), LevelFilter::OFF);
    }
}
