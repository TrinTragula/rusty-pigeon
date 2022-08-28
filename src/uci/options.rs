use std::cmp;

use crate::board::models::Side;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GoOptions {
    pub movetime: Option<isize>,
    pub wtime: Option<isize>,
    pub btime: Option<isize>,
    pub winc: Option<isize>,
    pub binc: Option<isize>,
    pub movestogo: Option<isize>,
    pub depth: Option<usize>,
    pub infinite: bool,
}
impl GoOptions {
    pub fn empty() -> GoOptions {
        GoOptions {
            movetime: None,
            wtime: None,
            btime: None,
            winc: None,
            binc: None,
            movestogo: None,
            depth: None,
            infinite: false,
        }
    }

    #[allow(dead_code)]
    pub fn depth(depth: usize) -> GoOptions {
        GoOptions {
            depth: Some(depth),
            ..GoOptions::empty()
        }
    }

    pub fn movetime(movetime: isize) -> GoOptions {
        GoOptions {
            movetime: Some(movetime),
            ..GoOptions::empty()
        }
    }

    pub fn parse(command: &str) -> GoOptions {
        let mut result = GoOptions::empty();
        let mut splitted = command.split(' ');

        while let Some(s) = splitted.next() {
            match s {
                "movetime" => {
                    let value = splitted.next().unwrap().parse::<isize>().unwrap();
                    result.movetime = Some(value);
                }
                "wtime" => {
                    let value = splitted.next().unwrap().parse::<isize>().unwrap();
                    result.wtime = Some(value);
                }
                "btime" => {
                    let value = splitted.next().unwrap().parse::<isize>().unwrap();
                    result.btime = Some(value);
                }
                "winc" => {
                    let value = splitted.next().unwrap().parse::<isize>().unwrap();
                    result.winc = Some(value);
                }
                "binc" => {
                    let value = splitted.next().unwrap().parse::<isize>().unwrap();
                    result.binc = Some(value);
                }
                "movestogo" => {
                    let value = splitted.next().unwrap().parse::<isize>().unwrap();
                    result.movestogo = Some(value);
                }
                "depth" => {
                    let value = splitted.next().unwrap().parse::<usize>().unwrap();
                    result.depth = Some(value);
                }
                "infinite" => {
                    result.infinite = true;
                }
                _ => continue,
            }
        }
        result
    }

    pub fn parse_uci_options(
        options: GoOptions,
        side: usize,
        halfmovesnumbers: usize,
    ) -> (usize, Option<u128>) {
        let mut time_to_move_ms: Option<u128> = None;
        let mut max_depth: usize = 99;

        // todo: More advanced time management
        if options.infinite {
            max_depth = 99;
        } else if options.depth.is_some() {
            max_depth = options.depth.unwrap();
        } else if options.movetime.is_some() {
            time_to_move_ms = Some(options.movetime.unwrap() as u128);
        } else if options.wtime.is_some() || options.btime.is_some() {
            // Time left on the clock
            let time = match side {
                Side::WHITE => options.wtime.unwrap(),
                Side::BLACK => options.btime.unwrap(),
                _ => 0,
            };

            // Move before time increment/game ending
            let movestogo = if options.movestogo.is_some() {
                options.movestogo.unwrap()
            } else {
                cmp::max(40 - (halfmovesnumbers / 2), 10) as isize
            };

            // Time increment each move
            let inc = match side {
                Side::WHITE => options.winc.unwrap_or(0),
                Side::BLACK => options.binc.unwrap_or(0),
                _ => 0,
            };

            // Use 2.25% of the time + half of the increment
            // thanks to http://mediocrechess.blogspot.com/2007/01/guide-time-management.html
            let mut time_to_move = (time / movestogo) + (inc / 2);
            if time_to_move >= time {
                time_to_move = cmp::max(100, time - 500);
            }
            if time_to_move < 500 && time > 1000 {
                // If we can, never search for less than half a second
                time_to_move = 500;
            }
            time_to_move_ms = Some(time_to_move as u128);
        }
        (max_depth, time_to_move_ms)
    }
}
