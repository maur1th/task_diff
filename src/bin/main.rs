use std::char;
use std::io;

extern crate ncurses;
use ncurses::*;

extern crate colored;
use colored::*;

extern crate task_diff;
use task_diff::{parser, util};

fn get_input() -> String {
    /* Setup ncurses. */
    initscr();
    raw();

    /* Allow for extended keyboard (like F1). */
    keypad(stdscr(), true);
    noecho();

    /* Prompt for a character. */
    printw("Please enter your diff (ENTER / ESC once done):\n");
    let mut input = String::new();
    loop {
        let ch = getch();
        // Return if ENTER or ESC
        if ch == 10 || ch == 27 {
            refresh();
            break;
        } else {
            let c = char::from_u32(ch as u32).expect("Invalid char");
            printw(&format!("{}", c));
            input.push(c);
        }
        refresh();
    }

    endwin();
    input
}

fn run_app() -> io::Result<()> {
    let input = get_input();
    let pair = util::Pair::new(&input)?;
    let result = parser::diff(&pair.a, &pair.b)?;
    for line in result {
        let color = match line.diff {
            '+' => "cyan",
            '~' => "yellow",
            '-' => "red",
            _ => "white",
        };
        let output = format!("{}", line);
        println!("{}", output.color(color));
    }
    Ok(())
}

fn main() {
    ::std::process::exit(match run_app() {
        Ok(_) => 0,
        Err(err) => {
            eprintln!("error: {}", err);
            1
        }
    });
}
