use crate::upstream::Upstream;
use std::net::SocketAddr;

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum)]
pub enum ArgVerbosity {
    Off = 0,
    Error,
    Warn,
    #[default]
    Info,
    Debug,
    Trace,
}

#[cfg(target_os = "android")]
impl TryFrom<jni::sys::jint> for ArgVerbosity {
    type Error = std::io::Error;
    fn try_from(value: jni::sys::jint) -> Result<Self, <Self as TryFrom<jni::sys::jint>>::Error> {
        match value {
            0 => Ok(ArgVerbosity::Off),
            1 => Ok(ArgVerbosity::Error),
            2 => Ok(ArgVerbosity::Warn),
            3 => Ok(ArgVerbosity::Info),
            4 => Ok(ArgVerbosity::Debug),
            5 => Ok(ArgVerbosity::Trace),
            _ => Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid verbosity level")),
        }
    }
}

impl From<ArgVerbosity> for log::LevelFilter {
    fn from(verbosity: ArgVerbosity) -> Self {
        match verbosity {
            ArgVerbosity::Off => log::LevelFilter::Off,
            ArgVerbosity::Error => log::LevelFilter::Error,
            ArgVerbosity::Warn => log::LevelFilter::Warn,
            ArgVerbosity::Info => log::LevelFilter::Info,
            ArgVerbosity::Debug => log::LevelFilter::Debug,
            ArgVerbosity::Trace => log::LevelFilter::Trace,
        }
    }
}

impl From<log::Level> for ArgVerbosity {
    fn from(level: log::Level) -> Self {
        match level {
            log::Level::Error => ArgVerbosity::Error,
            log::Level::Warn => ArgVerbosity::Warn,
            log::Level::Info => ArgVerbosity::Info,
            log::Level::Debug => ArgVerbosity::Debug,
            log::Level::Trace => ArgVerbosity::Trace,
        }
    }
}

impl std::fmt::Display for ArgVerbosity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArgVerbosity::Off => write!(f, "off"),
            ArgVerbosity::Error => write!(f, "error"),
            ArgVerbosity::Warn => write!(f, "warn"),
            ArgVerbosity::Info => write!(f, "info"),
            ArgVerbosity::Debug => write!(f, "debug"),
            ArgVerbosity::Trace => write!(f, "trace"),
        }
    }
}

/// A lightweight DNS-over-HTTPS proxy
#[derive(clap::Parser, Debug, Clone, PartialEq, Eq, Default)]
#[command(author, version, about = "A lightweight DNS-over-HTTPS proxy", long_about = None)]
pub struct Args {
    /// Listen for DNS requests on the addresses and ports
    #[arg(short, long, value_name = "IP:port", default_value = "127.0.0.1:53")]
    pub bind: Vec<SocketAddr>,

    /// URL(s) of upstream DNS-over-HTTPS service
    #[arg(short, long, value_name = "URL", default_value = "https://1.1.1.1/dns-query")]
    pub upstream_urls: Vec<String>,

    /// Verbosity level
    #[arg(short, long, value_name = "level", value_enum, default_value = "info")]
    pub verbosity: ArgVerbosity,

    /// Windows only: Run as a service
    #[cfg(target_os = "windows")]
    #[arg(long)]
    pub service: bool,
}

impl Args {
    pub fn bind<T: Into<SocketAddr>>(&mut self, bind: T) -> &mut Self {
        self.bind.push(bind.into());
        self
    }

    pub fn upstream_url<T: Into<String>>(&mut self, url: T) -> &mut Self {
        self.upstream_urls.push(url.into());
        self
    }

    pub fn verbosity(&mut self, verbosity: ArgVerbosity) -> &mut Self {
        self.verbosity = verbosity;
        self
    }

    /// Returns the Args for the current run.
    pub fn parse() -> Args {
        <Args as clap::Parser>::parse()
    }

    /// Return a vector of Upstreams with the given Client.
    pub fn upstreams<'a>(&'a self, client: &'a reqwest::Client) -> Vec<Upstream<'a>> {
        self.upstream_urls.iter().map(|url| Upstream::new(client, url)).collect()
    }
}
