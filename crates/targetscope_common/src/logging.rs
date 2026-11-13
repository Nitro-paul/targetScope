// doc/shared_libraries.md#logging

use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

use crate::clock::{local, unix_s, Clock, LocalDate, SystemClock};
use crate::config::BootstrapConfig;

pub struct LogGuard;

pub fn init(program: &str, config: &BootstrapConfig) -> LogGuard {
    init_with_clock(program, config, Arc::new(SystemClock))
}

pub fn init_with_clock(program: &str, config: &BootstrapConfig, clock: Arc<dyn Clock>) -> LogGuard {
    let _ = subscriber(program, config, clock).try_init();
    LogGuard
}

// doc/shared_libraries.md#logging
pub fn subscriber(program: &str, config: &BootstrapConfig, clock: Arc<dyn Clock>) -> impl tracing::Subscriber + Send + Sync {
    let filter = EnvFilter::try_new(&config.log_level).unwrap_or_else(|_| EnvFilter::new("info"));
    let file = DailyFile::new(config.log_dir.clone(), program, clock);
    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer().with_writer(io::stderr))
        .with(tracing_subscriber::fmt::layer().with_ansi(false).with_writer(file))
}

// doc/shared_libraries.md#logging
#[derive(Clone)]
pub struct DailyFile {
    inner: Arc<Mutex<DailyState>>,
}

struct DailyState {
    dir: PathBuf,
    program: String,
    clock: Arc<dyn Clock>,
    open: Option<(LocalDate, File)>,
}

impl DailyFile {
    pub fn new(dir: PathBuf, program: &str, clock: Arc<dyn Clock>) -> Self {
        DailyFile {
            inner: Arc::new(Mutex::new(DailyState { dir, program: program.to_string(), clock, open: None })),
        }
    }
}

impl DailyState {
    fn current(&self) -> PathBuf {
        self.dir.join(format!("{}.log", self.program))
    }

    fn file(&mut self) -> io::Result<&mut File> {
        let today = local(unix_s(self.clock.now())).date;
        let stale = match &self.open {
            Some((date, _)) => (*date != today).then_some(*date),
            None => fs::metadata(self.current())
                .and_then(|m| m.modified())
                .ok()
                .map(|t| local(unix_s(t)).date)
                .filter(|d| *d != today),
        };
        if let Some(date) = stale {
            self.open = None;
            let rotated = self.dir.join(format!("{}.log.{date}", self.program));
            let _ = fs::rename(self.current(), rotated);
        }
        if self.open.is_none() {
            fs::create_dir_all(&self.dir)?;
            let f = OpenOptions::new().create(true).append(true).open(self.current())?;
            self.open = Some((today, f));
        }
        Ok(&mut self.open.as_mut().unwrap().1)
    }
}

pub struct DailyWriter<'a> {
    file: &'a DailyFile,
}

impl Write for DailyWriter<'_> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut state = self.file.inner.lock().unwrap();
        state.file()?.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        let mut state = self.file.inner.lock().unwrap();
        state.file()?.flush()
    }
}

impl<'a> MakeWriter<'a> for DailyFile {
    type Writer = DailyWriter<'a>;

    fn make_writer(&'a self) -> Self::Writer {
        DailyWriter { file: self }
    }
}
