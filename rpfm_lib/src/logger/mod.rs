//---------------------------------------------------------------------------//
// Copyright (c) 2017-2020 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module to log CTDs and messages within RPFM.

This module is a custom CTD logging module, heavely inspired in the `human-panic` crate.
The reason to not use that crate is because it's not configurable. At all. But otherwise,
feel free to check it out if you need an easy-to-use simple error logger.

Note that these loggers need to be initialized on start by calling `Logger::init()`.
Otherwise, none of them will work.
!*/

use backtrace::Backtrace;
use log::{error, info};
use sentry::ClientInitGuard;
use simplelog::{ColorChoice, CombinedLogger, LevelFilter, TerminalMode, TermLogger, WriteLogger};

use sentry::integrations::log::SentryLogger;

use serde_derive::Serialize;
use uuid::Uuid;

use std::fs::File;
use std::io::{BufWriter, Write};
use std::panic::PanicInfo;
use std::path::Path;
use std::panic;

use rpfm_error::Result;

use crate::settings::get_config_path;

/// Log file to log execution steps and other messages.
const LOG_FILE: &str = "rpfm.log";

/// Current version of the crate.
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// This is the DSN needed for Sentry reports to work. Don't change it.
const SENTRY_DSN: &str = "https://a8bf0a98ed43467d841ec433fb3d75a8@sentry.io/1205298";

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains all the info to write into a `CrashReport` file.
#[derive(Debug, Serialize)]
pub struct Logger {

    /// Name of the Program. To know what of the programs crashed.
    name: String,

    /// Version of the Program/Lib.
    crate_version: String,

    /// If it happened in a `Debug` or `Release` build.
    build_type: String,

    /// The OS in which the crash happened.
    operating_system: String,

    /// The reason why the crash happened.
    explanation: String,

    /// A backtrace generated when the crash happened.
    backtrace: String,
}

//-------------------------------------------------------------------------------//
//                              Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `Logger`.
impl Logger {

    /// This function initialize the `Logger` to log crashes.
    ///
    /// There are three loggers active:
    /// - Log CTD to files.
    /// - Log CTD to sentry (release only)
    /// - Log execution steps to file/sentry.
    pub fn init() -> Result<ClientInitGuard> {
        info!("Initializing Logger.");

        // Make sure the config folder actually exists before we try to dump crashes into it.
        let config_path = get_config_path()?;

        // Initialize the combined logger, with a term logger (for runtime logging) and a write logger (for storing on a log file).
        let combined_logger = CombinedLogger::new(vec![
            TermLogger::new(LevelFilter::Info, simplelog::Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
            WriteLogger::new(LevelFilter::Info, simplelog::Config::default(), File::create(config_path.join(LOG_FILE))?),
        ]);

        // Initialize Sentry's logger, so anything logged goes to the breadcrumbs too.
        let logger = SentryLogger::with_dest(combined_logger);
        log::set_max_level(log::LevelFilter::Info);
        log::set_boxed_logger(Box::new(logger))?;

        // Initialize Sentry's guard, for remote reporting. Only for release mode.
        let dsn = if cfg!(debug_assertions) { "" } else { SENTRY_DSN };
        let sentry_guard = sentry::init((dsn, sentry::ClientOptions {
            release: sentry::release_name!(),
            sample_rate: 1.0,
            ..Default::default()
        }));

        // Setup the panic hooks to catch panics on all threads, not only the main one.
        let orig_hook = panic::take_hook();
        panic::set_hook(Box::new(move |info: &panic::PanicInfo| {
            info!("Panic detected. Generating backtraces and crash logs...");
            if Self::new(info, VERSION).save(&config_path).is_err() {
                error!("Failed to generate crash log.");
            }
            orig_hook(info);
            std::process::exit(1);
        }));

        // Return Sentry's guard, so we can keep it alive until everything explodes, or the user closes the program.
        info!("Logger initialized.");
        Ok(sentry_guard)
    }

    /// Create a new local Crash Report from a `Panic`.
    ///
    /// Remember that this creates the Crash Report in memory. If you want to save it to disk, you've to do it later.
    pub fn new(panic_info: &PanicInfo, version: &str) -> Self {

        let info = os_info::get();
        let operating_system = format!("OS: {}\nVersion: {}", info.os_type(), info.version());

        let mut explanation = String::new();
        if let Some(payload) = panic_info.payload().downcast_ref::<&str>() {
            explanation.push_str(&format!("Cause: {}\n", &payload));
        }

        match panic_info.location() {
            Some(location) => explanation.push_str(&format!("Panic occurred in file '{}' at line {}\n", location.file(), location.line())),
            None => explanation.push_str("Panic location unknown.\n"),
        }

        Self {
            name: env!("CARGO_PKG_NAME").to_owned(),
            crate_version: version.to_owned(),
            build_type: if cfg!(debug_assertions) { "Debug" } else { "Release" }.to_owned(),
            operating_system,
            explanation,
            backtrace: format!("{:#?}", Backtrace::new()),
        }
    }

    /// This function tries to save a generated Crash Report to the provided folder.
    pub fn save(&self, path: &Path) -> Result<()> {
        let uuid = Uuid::new_v4().to_hyphenated().to_string();
        let file_path = path.join(format!("error/error-report-{}.toml", &uuid));
        let mut file = BufWriter::new(File::create(&file_path)?);
        file.write_all(toml::to_string_pretty(&self)?.as_bytes())?;
        Ok(())
    }
}