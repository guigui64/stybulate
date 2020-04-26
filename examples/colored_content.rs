use stybulate::*;

fn main() {
    // Colored content can be inserted either with AsciiEscapedString
    let table = Table::new(
        Style::Presto,
        vec![
            vec![Cell::from("Normal"), Cell::from("Stybulate")],
            vec![
                Cell::from("Red"),
                Cell::Text(Box::new(AsciiEscapedString::from(
                    "\x1b[31mStybulate\x1b[0m",
                ))),
            ],
            vec![
                Cell::from("Green"),
                Cell::Text(Box::new(AsciiEscapedString::from(
                    "\x1b[32mStybulate\x1b[0m",
                ))),
            ],
            vec![
                Cell::from("Yellow"),
                Cell::Text(Box::new(AsciiEscapedString::from(
                    "\x1b[33mStybulate\x1b[0m",
                ))),
            ],
            vec![
                Cell::from("Blue"),
                Cell::Text(Box::new(AsciiEscapedString::from(
                    "\x1b[34mStybulate\x1b[0m",
                ))),
            ],
            vec![
                Cell::from("Mutlicolor and blinking (because why not?)"),
                Cell::Text(Box::new(AsciiEscapedString::from(
                    "\x1b[5;30mS\x1b[31mt\x1b[32my\x1b[33mb\x1b[34mu\x1b[35ml\x1b[36ma\x1b[37mt\x1b[30me\x1b[0m",
                ))),
            ],
        ],
        Some(Headers::from(vec!["Style", "Text"])),
    );
    println!("{}", table.tabulate());

    // or with ansi_term ANSIStrings (with the feature "ansi_term_style")
    use ansi_term::ANSIStrings;
    use ansi_term::Color::*;
    let red_stybulate = &[Red.paint("Stybulate")];
    let green_stybulate = &[Green.paint("Stybulate")];
    let yellow_stybulate = &[Yellow.paint("Stybulate")];
    let blue_stybulate = &[Blue.paint("Stybulate")];
    let multicolorandblinking_stybulate = &[
        Black.blink().paint("S"),
        Red.blink().paint("t"),
        Green.blink().paint("y"),
        Yellow.blink().paint("b"),
        Blue.blink().paint("u"),
        Purple.blink().paint("l"),
        Cyan.blink().paint("a"),
        White.blink().paint("t"),
        Black.blink().paint("e"),
    ];
    let table2 = Table::new(
        Style::FancyGithub,
        vec![
            vec![Cell::from("Normal"), Cell::from("Stybulate")],
            vec![
                Cell::from("Red"),
                Cell::Text(Box::new(ANSIStrings(red_stybulate))),
            ],
            vec![
                Cell::from("Green"),
                Cell::Text(Box::new(ANSIStrings(green_stybulate))),
            ],
            vec![
                Cell::from("Yellow"),
                Cell::Text(Box::new(ANSIStrings(yellow_stybulate))),
            ],
            vec![
                Cell::from("Blue"),
                Cell::Text(Box::new(ANSIStrings(blue_stybulate))),
            ],
            vec![
                Cell::from("Mutlicolor and blinking (because why not?)"),
                Cell::Text(Box::new(ANSIStrings(multicolorandblinking_stybulate))),
            ],
        ],
        Some(Headers::from(vec!["Style", "Text"])),
    );
    println!("{}", table2.tabulate());
}
