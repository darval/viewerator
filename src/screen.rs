use log::*;
use pancurses::{cbreak, endwin, init_pair, initscr, noecho, start_color, Input, Window, ACS_HLINE};
use std::convert::TryInto;
use std::process::{exit, ExitCode, Termination};

use crate::log_display;
use crate::webdata;

pub struct Screen {
    window: Window,
    wd: webdata::WebData,
    x: i32,
    y: i32,
    current_worker: usize,
}

const MIN_X: i32 = 126;
const MIN_Y: i32 = 26;
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
            println!(
                "Console screen must be at least {} columns in X, current X is {}",
                MIN_X, self.x
            );
            error!(
                "Console screen must be at least {} columns in X, current X is {}",
                MIN_X, self.x
            );
            exit(ExitCode::SUCCESS.report())
        }
        if self.y < MIN_Y {
            endwin();
            println!(
                "Console screen must be at least {} rows in Y, current Y is {}",
                MIN_Y, self.y
            );
            error!(
                "Console screen must be at least {} rows in Y, current Y is {}",
                MIN_Y, self.y
            );
            exit(ExitCode::SUCCESS.report())
        }
        debug!("Screen is {} X x {} Y", self.x, self.y);

        start_color();
        init_pair(HEALTH_COLOR_RAMPUP, pancurses::COLOR_BLACK, pancurses::COLOR_GREEN);
        init_pair(HEALTH_COLOR_SLOWINCREASE, pancurses::COLOR_BLACK, pancurses::COLOR_CYAN);
        init_pair(HEALTH_COLOR_HOLD, pancurses::COLOR_BLACK, pancurses::COLOR_YELLOW);
        init_pair(
            HEALTH_COLOR_SLOWDECREASE,
            pancurses::COLOR_WHITE,
            pancurses::COLOR_MAGENTA,
        );
        init_pair(HEALTH_COLOR_CRITICAL, pancurses::COLOR_WHITE, pancurses::COLOR_RED);

        pancurses::set_title(&*format!("Viewerator v{}", clap::crate_version!()));
        self.window.keypad(true);
        self.window.timeout(1000);
        cbreak();
        noecho();

        self.window.refresh();
    }

    pub fn mainloop<'a>(&mut self, matches: &clap::ArgMatches<'a>) {
        let mut ld = log_display::LogDisplay::new();
        loop {
            match self.window.getch() {
                Some(Input::Character(c)) => {
                    if c.is_ascii_digit() {
                        let w: usize = c.to_digit(10).unwrap().try_into().unwrap();
                        if w <= self.wd.workers.len() {
                            if self.wd.workers[self.current_worker].hw_type != self.wd.workers[w - 1].hw_type {
                                self.window.erase();
                            }
                            self.current_worker = w - 1;
                            info!("Showing device {}", w - 1);
                            self.update_screen(&matches, &mut ld);
                        }
                    }
                }
                Some(Input::KeyDC) => break,
                Some(_input) => {} // ignore
                None => {
                    self.update_screen(&matches, &mut ld);
                }
            }
            self.window.refresh();
        }
        info!("Exiting..");
    }

    pub fn update_screen<'a>(&mut self, matches: &clap::ArgMatches<'a>, ld: &mut log_display::LogDisplay) {
        debug!("Getting data");
        self.wd.getdata(matches);
        let loginfo = ld.read_raw();
        debug!("Updating screen");
        self.window.printw(format!(
            "Viewerator v{}, press delete to exit    ",
            clap::crate_version!()
        ));
        self.window.mv(0, self.x - 47);
        let attr = self.set_text_colors(&"critical".to_string());
        self.window.printw(" --- ");
        self.window.attroff(attr);
        let attr = self.set_text_colors(&"slowDecrease".to_string());
        self.window.printw("  -  ");
        self.window.attroff(attr);
        let attr = self.set_text_colors(&"hold".to_string());
        self.window.printw("     ");
        self.window.attroff(attr);
        let attr = self.set_text_colors(&"slowIncrease".to_string());
        self.window.printw("  +  ");
        self.window.attroff(attr);
        let attr = self.set_text_colors(&"rampUp".to_string());
        self.window.printw(" +++ \n");
        self.window.attroff(attr);

        self.window.hline(ACS_HLINE(), self.x);

        self.window
            .mvprintw(0, self.x - 20, format!("Minerator: {}", self.wd.minerator));
        let num_devices = self.wd.workers.len();
        debug!("Numer of devices = {}", num_devices);
        if num_devices != 1 {
            self.draw_devices(1, 30);
        }
        for (i, w) in self.wd.workers.iter().enumerate() {
            if i == self.current_worker {
                self.window.mvprintw(2, 0, format!("HWUID: {}", w.dna));
                self.window.mvprintw(3, 0, format!("Name:  {}", w.name));

                self.window.mv(4, 0);
                self.window.hline(ACS_HLINE(), 24);
                self.window.mvprintw(5, 0, "Input Power    ");
                let attr = self.set_text_colors(&w.input_power_health);
                self.window.mvprintw(5, 16, Screen::float_to_string3(w.input_power));
                self.window.attroff(attr);

                self.window.mvprintw(6, 0, "AUX 12V");
                let attr = self.set_text_colors(&w.aux_12v_health);
                self.window.mvprintw(6, 16, Screen::float_to_string3(w.aux_12v));
                self.window.attroff(attr);

                self.window.mvprintw(7, 0, "AUX Current");
                let attr = self.set_text_colors(&w.aux_current_health);
                self.window.mvprintw(7, 16, Screen::float_to_string3(w.aux_current));
                self.window.attroff(attr);

                self.window.mvprintw(8, 0, "PEX 12V");
                let attr = self.set_text_colors(&w.pex_12v_health);
                self.window.mvprintw(8, 16, Screen::float_to_string3(w.pex_12v));
                self.window.attroff(attr);

                self.window.mvprintw(9, 0, "PEX Current");
                let attr = self.set_text_colors(&w.pex_current_health);
                self.window.mvprintw(9, 16, Screen::float_to_string3(w.pex_current));
                self.window.attroff(attr);

                self.window.mvprintw(10, 0, "VCCINT");
                self.window.mvprintw(10, 16, Screen::float_to_string3(w.vccint));

                self.window.mvprintw(11, 0, "VCCINT Current");
                let attr = self.set_text_colors(&w.vccint_current_health);
                self.window.mvprintw(11, 16, Screen::float_to_string3(w.vccint_current));
                self.window.attroff(attr);

                self.window.mvprintw(12, 0, "VRCTRL Temp");
                let attr = self.set_text_colors(&w.vrctrl_temp_health);
                self.window.mvprintw(12, 16, Screen::float_to_string3(w.vrctrl_temp));
                self.window.attroff(attr);
                if w.hw_type == webdata::HWTYPE_BCU {
                    self.draw_phases(4, 26, &w);
                }
                self.draw_sysmons(9, 50, &w.sysmons);
                for (_num, core) in w.cores.cores.iter().enumerate() {
                    self.draw_clock(4, 50, &core.clock);
                    self.draw_stats(14, 0, &w, &core);
                }
            }
        }
        if !matches.is_present("input_file") {
            self.window.mv(21, 0);
            let lines_available: usize = (self.y - 21).try_into().unwrap();
            let name = format!("{}: ", &self.wd.workers[self.current_worker].cores.cores[0].stats.name);
            let match_this_board: Vec<&String> = loginfo
                .iter()
                .filter(|s| s.contains(&name) || s.contains("Fee"))
                .collect();
            let starting_index = match_this_board.len() - lines_available;
            let display_lines = &match_this_board[starting_index..];
            for line in display_lines {
                let mut attr = pancurses::A_NORMAL;
                if line.contains("WRN") {
                    attr = pancurses::COLOR_PAIR(HEALTH_COLOR_HOLD.try_into().unwrap());
                } else if line.contains("ERR") {
                    attr = pancurses::COLOR_PAIR(HEALTH_COLOR_CRITICAL.try_into().unwrap());
                }
                self.window.attron(attr);
                self.window.printw(format!("{}\n", line));
                self.window.attroff(attr);
            }
        }
        self.window.mv(self.y - 1, self.x - 1);
    }

    fn calc_total(stat: webdata::StatDetail, val: f32) -> f32 {
        let div: f32 = ((stat.endTime - stat.startTime) / 1_000_000_000.0) as f32;
        val / div
    }

    fn float_to_string1(f: f32) -> String {
        if f >= 0.0 && f < 1000.0 {
            format!("{:>7.1}", f)
        } else if f >= 1000.0 && f < 1_000_000.0 {
            format!("{:>6.1}K", f / 1000.0)
        } else if f >= 1_000_000.0 && f < 1_000_000_000.0 {
            format!("{:>6.1}M", f / 1_000_000.0)
        } else if f >= 1_000_000_000.0 && f < 1_000_000_000_000.0 {
            format!("{:>6.1}G", f / 1_000_000_000.0)
        } else {
            "*******".to_string()
        }
    }

    fn float_to_string3(f: f32) -> String {
        if f >= 0.0 && f < 1000.0 {
            format!("{arg:>8.3}", arg = f)
        } else {
            f.to_string()
        }
    }

    fn set_text_colors(&self, health: &str) -> pancurses::chtype {
        let mut attr = pancurses::A_COLOR;
        match &health[..] {
            "rampUp" => {
                attr = pancurses::COLOR_PAIR(HEALTH_COLOR_RAMPUP.try_into().unwrap());
            }
            "slowIncrease" => {
                attr = pancurses::COLOR_PAIR(HEALTH_COLOR_SLOWINCREASE.try_into().unwrap());
            }
            "hold" => {
                attr = pancurses::COLOR_PAIR(HEALTH_COLOR_HOLD.try_into().unwrap());
            }
            "slowDecrease" => {
                attr = pancurses::COLOR_PAIR(HEALTH_COLOR_SLOWDECREASE.try_into().unwrap());
            }
            "critical" => {
                attr = pancurses::COLOR_PAIR(HEALTH_COLOR_CRITICAL.try_into().unwrap());
            }
            _ => {}
        }
        self.window.attron(attr);
        attr
    }

    fn draw_devices(&self, y: i32, x: i32) {
        self.window.mvprintw(y, x, " Current device is highlighted: ");
        for (i, w) in self.wd.workers.iter().enumerate() {
            let mut attr = pancurses::A_NORMAL;
            if self.current_worker == i {
                attr |= pancurses::A_BOLD | pancurses::A_UNDERLINE;
            } else {
                attr = self.set_text_colors(&w.worse_health);
            }
            self.window.attron(attr);
            self.window.printw(format!("{}", i + 1));
            self.window.attroff(attr);
            self.window.printw("  ");
        }
    }

    fn draw_sysmons(&self, y: i32, x: i32, sysmons: &webdata::SysMons) {
        let line_length: i32 = (16 * sysmons.sysmon.len()).try_into().unwrap();
        self.window.mv(y, x);
        self.window.hline(ACS_HLINE(), line_length);
        for (num, sysmon) in sysmons.sysmon.iter().enumerate() {
            let mut column_offset: i32 = (num * 16).try_into().unwrap();
            column_offset += x;
            self.window.mvprintw(y + 1, column_offset, format!("Sysmon {}", num));
            self.window.mvprintw(y + 2, column_offset, "temp");
            self.window.mvprintw(y + 3, column_offset, "vccint");
            let attr = self.set_text_colors(&sysmon.health);
            self.window
                .mvprintw(y + 2, column_offset + 7, Screen::float_to_string3(sysmon.temperature));
            self.window.attroff(attr);
            self.window
                .mvprintw(y + 3, column_offset + 7, Screen::float_to_string3(sysmon.vccint));
        }
    }

    fn draw_phases(&self, y: i32, x: i32, w: &webdata::Worker) {
        self.window.mv(y, x);
        self.window.hline(ACS_HLINE(), 22);

        self.window.mvprintw(y + 1, x, "LTC3884 Phase 0");
        self.window.mvprintw(y + 2, x, "Global status");
        self.window
            .mvprintw(y + 2, x + 14, format!("{:#08x}", w.phase0_status_global));
        self.window.mvprintw(y + 3, x, "temperature");
        let attr = self.set_text_colors(&w.phase0_temperature_health);
        self.window
            .mvprintw(y + 3, x + 14, Screen::float_to_string3(w.phase0_temperature));
        self.window.attroff(attr);
        self.window.mvprintw(y + 4, x, "vout");
        self.window
            .mvprintw(y + 4, x + 14, Screen::float_to_string3(w.phase0_vout));

        self.window.mvprintw(y + 5, x, "LTC3884 Phase 1");
        self.window.mvprintw(y + 6, x, "Global status");
        self.window
            .mvprintw(y + 6, x + 14, format!("{:#08x}", w.phase1_status_global));
        self.window.mvprintw(y + 7, x, "temperature");
        let attr = self.set_text_colors(&w.phase1_temperature_health);
        self.window
            .mvprintw(y + 7, x + 14, Screen::float_to_string3(w.phase1_temperature));
        self.window.attroff(attr);
        self.window.mvprintw(y + 8, x, "vout");
        self.window
            .mvprintw(y + 8, x + 14, Screen::float_to_string3(w.phase1_vout));
    }

    fn draw_clock(&self, y: i32, x: i32, clock: &webdata::Clock) {
        self.window.mv(y, x);
        self.window.hline(ACS_HLINE(), 28);

        self.window.mvprintw(y + 1, x, "Clock Multiplier");
        self.window.mvprintw(y + 2, x, "Bad Nonces");
        self.window.mvprintw(y + 3, x, "Total Nonces");
        let attr = self.set_text_colors(&clock.health);
        self.window
            .mvprintw(y + 1, x + 20, Screen::float_to_string3(clock.multiplier));
        self.window
            .mvprintw(y + 2, x + 20, Screen::float_to_string3(clock.badNonces));
        self.window
            .mvprintw(y + 3, x + 20, Screen::float_to_string3(clock.totalNonces));
        self.window.attroff(attr);
    }

    fn draw_stats(&self, y: i32, x: i32, w: &webdata::Worker, core: &webdata::Core) {
        self.window.mvprintw(y, x, "Worker/Pool Name");
        self.window.mv(y + 2, x);
        self.window.hline(ACS_HLINE(), 27);
        self.window.mvprintw(y, x + 28, "Since start [MH/s]");
        self.window
            .mvprintw(y + 1, x + 28, "WrkReq |Calcul |Found  |Valid  |Submit |Accept");
        self.window.mv(y + 2, x + 28);
        self.window.hline(ACS_HLINE(), 48);
        self.window.mvprintw(y, x + 78, "Last Minute [MH/s]");
        self.window
            .mvprintw(y + 1, x + 78, "WrkReq |Calcul |Found  |Valid  |Submit |Accept ");
        self.window.mv(y + 2, x + 78);
        self.window.hline(ACS_HLINE(), 48);

        // output totals
        // worksource
        self.draw_stat_line(y + 3, x, &w.worksource.stats);
        // fee
        self.draw_stat_line(y + 4, x, &w.fee.stats);
        // total
        self.draw_stat_line(y + 5, x, &core.stats);
    }

    fn draw_stat_line(&self, y: i32, x: i32, stats: &webdata::Stats) {
        self.window.mvprintw(y, x, format!("{:29}", stats.name));
        self.window.mvprintw(
            y,
            x + 28,
            Screen::float_to_string1(Screen::calc_total(stats.total, stats.total.requested)),
        );
        self.window.mvprintw(
            y,
            x + 36,
            Screen::float_to_string1(Screen::calc_total(stats.total, stats.total.calculated)),
        );
        self.window.mvprintw(
            y,
            x + 44,
            Screen::float_to_string1(Screen::calc_total(stats.total, stats.total.found)),
        );
        self.window.mvprintw(
            y,
            x + 52,
            Screen::float_to_string1(Screen::calc_total(stats.total, stats.total.valid)),
        );
        self.window.mvprintw(
            y,
            x + 60,
            Screen::float_to_string1(Screen::calc_total(stats.total, stats.total.submitted)),
        );
        self.window.mvprintw(
            y,
            x + 68,
            Screen::float_to_string1(Screen::calc_total(stats.total, stats.total.accepted)),
        );

        self.window
            .mvprintw(y, x + 78, Screen::float_to_string1(stats.minute.requested / 60.0));
        self.window
            .mvprintw(y, x + 86, Screen::float_to_string1(stats.minute.calculated / 60.0));
        self.window
            .mvprintw(y, x + 94, Screen::float_to_string1(stats.minute.found / 60.0));
        self.window
            .mvprintw(y, x + 102, Screen::float_to_string1(stats.minute.valid / 60.0));
        self.window
            .mvprintw(y, x + 110, Screen::float_to_string1(stats.minute.submitted / 60.0));
        self.window
            .mvprintw(y, x + 118, Screen::float_to_string1(stats.minute.accepted / 60.0));
    }
}
