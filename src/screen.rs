use pancurses::{initscr, endwin, cbreak, noecho, ACS_HLINE, Input, Window};
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
            wd: wd,
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
                None => { self.update_screen(&matches); () }
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
                let column_offset: i32 = (i * 20).try_into().unwrap();
                self.window.mvprintw(2, column_offset, format!("DNA: {}", w.dna));
                self.window.mvprintw(3, column_offset, format!("Name: {}", w.name));
            }
        }

    }
}