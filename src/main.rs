#![feature(process_exitcode_placeholder,termination_trait_lib)]
use std::fs::OpenOptions;
use std::fs;
use std::path::Path;
use simplelog::*;
use log::*;
use pancurses::endwin;

mod screen;
mod webdata;

fn main() {
    let matches = clap::App::new(clap::crate_name!())
        .version(clap::crate_version!())
        .author(clap::crate_authors!())
        .about(clap::crate_description!())
        .arg(clap::Arg::with_name("config_dir")
            .short("c")
            .long("config_dir")
            .value_name("DIR")
            .help("Sets a custom config directory")
            )
        .get_matches();
    init_logging(&matches);

    let mut scr = screen::Screen::new(webdata::WebData::new());
    scr.init();
    scr.mainloop(&matches);
    endwin();
  }

fn init_logging<'a>(matches: &clap::ArgMatches<'a> ) {
    let appname = clap::crate_name!();
    let version = clap::crate_version!();
    let default_config = format!("{}/.{}", env!("HOME"), appname);
    let mut created_dir = false;
    let config_dir = matches.value_of("config_dir")
            .unwrap_or(&default_config);
    if !(Path::new(&config_dir).exists()) { 
        fs::create_dir_all(&config_dir).unwrap();
        created_dir = true;
    }
    CombinedLogger::init(vec![
        TermLogger::new(LevelFilter::Warn, Config::default(), TerminalMode::Mixed).unwrap(),
        WriteLogger::new(
            LevelFilter::Debug,
            Config::default(),
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(format!("{}/{}.log", config_dir, appname))
                .unwrap(),
        ),
    ])
    .unwrap();
    info!("Logging started for v{} of {}", version, appname);
    if created_dir {
        info!("Created new config directory: {}", config_dir);
    }
}
