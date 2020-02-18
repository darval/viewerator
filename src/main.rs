use log::*;
use mylib::screen;
use mylib::webdata;
use pancurses::endwin;
use simplelog::*;
use std::fs;
use std::fs::OpenOptions;
use std::panic;
use std::path::Path;

fn main() {
    let matches = clap::App::new(clap::crate_name!())
        .version(clap::crate_version!())
        .author(clap::crate_authors!())
        .about(clap::crate_description!())
        .arg(
            clap::Arg::with_name("config_dir")
                .short("c")
                .long("config_dir")
                .value_name("DIR")
                .help("Sets a custom config directory"),
        )
        .arg(
            clap::Arg::with_name("log_level")
                .short("l")
                .long("log_level")
                .value_name("debug|info|warn|error")
                .help("Sets the log level (default info) for the viewerator.log in the config directory"),
        )
        .arg(
            clap::Arg::with_name("input_file")
                .short("f")
                .long("input_file")
                .value_name("FILE")
                .help("Read JSON from file rather than http://localhost/api/status"),
        )
        .arg(
            clap::Arg::with_name("debug")
                .short("d")
                .long("debug")
                .help("Runs in debug mode, which allows normal panics with trace"),
        )
        .get_matches();
    init_logging(&matches);

    let mut scr = screen::Screen::new(webdata::WebData::new());
    if !matches.is_present("debug") {
        panic::set_hook(Box::new(|panic_info| {
            endwin();
            if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
                eprintln!("{:?}", s);
            }
            if let Some(location) = panic_info.location() {
                eprintln!("Unexpected termination occurred in file '{}' at line {}", location.file(),
                    location.line());
            } else {
                eprintln!("Unexpected termination occurred but can't get location information...");
            }
            eprintln!(
                "Please report what happened at https://github.com/darval/viewerator/issues"
            );
        }));
    }
    scr.init();
    scr.mainloop(&matches);
    endwin();
}

fn init_logging<'a>(matches: &clap::ArgMatches<'a>) {
    let appname = clap::crate_name!();
    let version = clap::crate_version!();
    let default_config = format!("{}/.{}", env!("HOME"), appname);
    let mut created_dir = false;
    let config_dir = matches.value_of("config_dir").unwrap_or(&default_config);
    if !(Path::new(&config_dir).exists()) {
        fs::create_dir_all(&config_dir).unwrap();
        created_dir = true;
    }
    let default_log_level = "info";
    let log_level = match matches.value_of("log_level").unwrap_or(&default_log_level) {
        "info" => LevelFilter::Info,
        "debug" => LevelFilter::Debug,
        "warn" => LevelFilter::Warn,
        "error" => LevelFilter::Error,
        _ => LevelFilter::Debug,
    };

    CombinedLogger::init(vec![
        TermLogger::new(LevelFilter::Warn, Config::default(), TerminalMode::Mixed).unwrap(),
        WriteLogger::new(
            log_level,
            Config::default(),
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(format!("{}/{}.log", config_dir, appname))
                .unwrap(),
        ),
    ])
    .unwrap();
    info!(
        "Logging started for v{} of {}, log level: {}",
        version, appname, log_level
    );
    if created_dir {
        info!("Created new config directory: {}", config_dir);
    }
}
