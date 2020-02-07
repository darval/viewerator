use pancurses::{initscr, init_pair, start_color, endwin, cbreak, noecho, ACS_HLINE, Input, Window};
use std::process::{exit, ExitCode, Termination};
use std::convert::TryInto;
use log::*;

use crate::webdata;

pub struct Screen {
    window: Window,
    wd: webdata::WebData,
    x: i32,
    y: i32,
    current_worker: usize,
}

const MIN_X: i32 = 80;
const MIN_Y: i32 = 25;
const HEALTH_COLOR_RAMPUP: i16 = 1;
const HEALTH_COLOR_SLOWINCREASE: i16 = 2;
const HEALTH_COLOR_HOLD: i16 = 3;
const HEALTH_COLOR_SLOWDECREASE: i16 = 4;
const HEALTH_COLOR_CRITICAL: i16 = 5;

impl Screen {
    //
    // Basically just creates a screen object (ala window in curses)
    //
    pub fn new(wd: webdata::WebData) -> Screen {
        Screen { 
            window: initscr(), 
            x: 0,
            y: 0,
            current_worker: 0,
            wd,
        }
    }

    // 
    // Create initial screen and ensure we have a large enough window to support
    // what we want to accomplish.
    //
    pub fn init(&mut self) {
        self.x = self.window.get_max_x();
        self.y = self.window.get_max_y();
        if self.x < MIN_X {
            endwin();
            println!("Console screen must be at least {} columns in X, current X is {}", MIN_X, self.x);
            error!("Console screen must be at least {} columns in X, current X is {}", MIN_X, self.x);
            exit(ExitCode::SUCCESS.report())
        }
        if self.y < MIN_Y {
            endwin();
            println!("Console screen must be at least {} rows in Y, current Y is {}", MIN_Y, self.y);
            error!("Console screen must be at least {} rows in Y, current Y is {}", MIN_Y, self.y);
            exit(ExitCode::SUCCESS.report())
        }
        debug!("Screen is {} X x {} Y", self.x, self.y);

        start_color();
        init_pair(HEALTH_COLOR_RAMPUP, pancurses::COLOR_BLACK, pancurses::COLOR_GREEN);
        init_pair(HEALTH_COLOR_SLOWINCREASE, pancurses::COLOR_BLACK, pancurses::COLOR_CYAN);
        init_pair(HEALTH_COLOR_HOLD, pancurses::COLOR_BLACK, pancurses::COLOR_YELLOW);

        self.window.printw(format!("Viewerator v{}, press delete to exit\n", clap::crate_version!()));
        self.window.hline(ACS_HLINE(), self.x);

        self.window.keypad(true);
        self.window.timeout(1000);
        cbreak();
        noecho();     

        self.window.refresh();
    }

    pub fn mainloop<'a>(&mut self, matches: &clap::ArgMatches<'a>) {
        loop {
            match self.window.getch() {
                Some(Input::Character(c)) => { 
                    if c.is_ascii_digit() {
                        let w = c.to_digit(10).unwrap()
                                            .try_into().unwrap();
                        if w < self.wd.workers.len() {
                            self.current_worker = w;
                        }
                    }
                },
                Some(Input::KeyDC) => break,
                Some(_input) => {}, // ignore
                None => { self.update_screen(&matches); }
            }
        }
     
    }

    pub fn update_screen<'a>(&mut self, matches: &clap::ArgMatches<'a>)  {
        debug!("Getting data");
        self.wd.getdata(matches);
        debug!("Updating screen");
        self.window.mvprintw(0, self.x - 20, format!("Minerator: {}", self.wd.minerator));
        debug!("Numer of devices = {}", self.wd.workers.len());
        for (i, w) in self.wd.workers.iter().enumerate() {
            if i == self.current_worker {
                self.window.mvprintw(2, 0, format!("DNA: {}", w.dna));
                self.window.mvprintw(3, 0, format!("Name: {}", w.name));
                self.window.mv(4,0);
                let line_length: i32 =  (20 * w.sysmons.sysmon.len()).try_into().unwrap();
                self.window.hline(ACS_HLINE(), line_length);
                for (num, sysmon) in w.sysmons.sysmon.iter().enumerate() {
                    let column_offset: i32 = (num * 20).try_into().unwrap();
                    let attr = self.set_text_colors(&sysmon.health);
                    self.window.mvprintw(5, column_offset, format!("Sysmon {}", num));
                    self.window.attroff(attr);
                    self.window.mvprintw(7,column_offset, format!("{} degrees", Screen::float_to_string3(sysmon.temperature)));
                    self.window.mvprintw(8,column_offset, format!("{} volts", Screen::float_to_string3(sysmon.vccint)));
                }
                for (_num, core) in w.cores.cores.iter().enumerate() {
                    self.window.mvprintw(13, 0, "Clock Multiplier");
                    self.window.mvprintw(14, 0, "Bad Nonces");
                    self.window.mvprintw(15, 0, "Total Nonces");
                    let attr = self.set_text_colors(&core.clock.health);
                    self.window.mvprintw(13, 20, Screen::float_to_string3(core.clock.multiplier));
                    self.window.mvprintw(14, 20, Screen::float_to_string3(core.clock.badNonces));
                    self.window.mvprintw(15, 20, Screen::float_to_string3(core.clock.totalNonces));
                    self.window.attroff(attr);

                    self.window.mvprintw(10, 30, "Worker/Pool Name");
                    self.window.mv(12, 30);
                    self.window.hline(ACS_HLINE(), 27);
                    self.window.mvprintw(10, 58, "Since start [MH/s]");
                    self.window.mvprintw(11, 58, "WrkReq |Calcul |Found  |Valid  |Submit |Accept");         
                    self.window.mv(12, 58);
                    self.window.hline(ACS_HLINE(), 46);
                    self.window.mvprintw(10, 105, "Last Minute [MH/s]");
                    self.window.mvprintw(11, 105, "WrkReq |Calcul |Found  |Valid  |Submit |Accept");         
                    self.window.mv(12, 105);
                    self.window.hline(ACS_HLINE(), 46);

                    // output totals
                    self.window.mvprintw(15, 30, format!("{:29}", w.name));
                    self.window.mvprintw(15, 58, Screen::float_to_string1(core.stats.total.requested));
                    self.window.mvprintw(15, 66, Screen::float_to_string1(core.stats.total.calculated));
                    self.window.mvprintw(15, 74, Screen::float_to_string1(core.stats.total.found));
                    self.window.mvprintw(15, 82, Screen::float_to_string1(core.stats.total.valid));
                    self.window.mvprintw(15, 90, Screen::float_to_string1(core.stats.total.submitted));
                    self.window.mvprintw(15, 97, Screen::float_to_string1(core.stats.total.accepted));
                    self.window.mvprintw(15, 105, Screen::float_to_string1(core.stats.minute.requested));
                    self.window.mvprintw(15, 113, Screen::float_to_string1(core.stats.minute.calculated));
                    self.window.mvprintw(15, 121, Screen::float_to_string1(core.stats.minute.found));
                    self.window.mvprintw(15, 129, Screen::float_to_string1(core.stats.minute.valid));
                    self.window.mvprintw(15, 137, Screen::float_to_string1(core.stats.minute.submitted));
                    self.window.mvprintw(15, 145, Screen::float_to_string1(core.stats.minute.accepted));

                }
            }
        }
        self.window.mv(self.y-1, self.x-1);

    }

    fn float_to_string1(f: f32) -> String {
        if f >= 0.0 && f < 1000.0 {
            format!("{:>7.1}", f)
        } else if f >= 1000.0 && f < 1000000.0 {
            format!("{:>6.1}K", f/1000.0)
        } else if f >= 1000000.0 && f < 1000000000.0 {
            format!("{:>6.1}M", f/1000000.0)
        } else if f >= 1000000000.0 && f < 1000000000000.0 {
            format!("{:>6.1}G", f/1000000000.0)
        } else {
            "*******".to_string()
        }
    }

    fn float_to_string3(f: f32) -> String {
        if f >= 0.0 && f < 1000.0 {
            format!("{arg:>8.3}", arg=f)
        } else {
            f.to_string()
        }
    }

    fn set_text_colors(&self, health: &String) -> pancurses::chtype {
        let mut attr = pancurses::A_COLOR;
        match &health[..] {
            "rampUp" => { attr = pancurses::COLOR_PAIR(HEALTH_COLOR_RAMPUP.try_into().unwrap());}, 
            "slowIncrease" => { attr = pancurses::COLOR_PAIR(HEALTH_COLOR_SLOWINCREASE.try_into().unwrap()); }, 
            "hold" => { attr = pancurses::COLOR_PAIR(HEALTH_COLOR_HOLD.try_into().unwrap()); }, 
            _ => {},
        }
        self.window.attron(attr);
        attr
    }
}