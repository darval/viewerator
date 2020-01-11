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
                Some(Input::KeyF1) => {
                    self.window.printw("\n"); 
                    self.window.attron(pancurses::A_BOLD);
                    self.window.printw("F1 ");
                    self.window.attroff(pancurses::A_BOLD);
                    self.window.printw("key pressed\n");
                },
                Some(input) => { self.window.addstr(&format!("{:?}", input)); },
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
                for (num, sysmon) in w.sysmons.sysmon.iter().enumerate() {
                    let column_offset: i32 = (num * 20).try_into().unwrap();
                    let attr = self.set_text_colors(&sysmon.health);
                    self.window.mvprintw(5, column_offset, format!("Sysmon {}", num));
                    self.window.attroff(attr);
                    self.window.mvprintw(7,column_offset, Screen::float_to_string(sysmon.temperature));
                    self.window.mvprintw(8,column_offset, Screen::float_to_string(sysmon.vccint));
                }
            }
        }

    }

    fn float_to_string(f: f32) -> String {
        if f > 0.0 && f < 1000.0 {
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