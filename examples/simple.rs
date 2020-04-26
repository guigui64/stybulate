use stybulate::*;

fn main() {
    // A simple table with strings and numbers and the default alignments
    let mut table = Table::new(
        Style::Fancy,
        vec![
            vec![Cell::from("spam"), Cell::Float(41.9999)],
            vec![Cell::from("eggs"), Cell::Int(451)],
        ],
        Some(Headers::from(vec!["strings", "numbers"])),
    );
    println!("{}", table.tabulate());
    /* Will print:
    ╒═══════════╤═══════════╕
    │ strings   │   numbers │
    ╞═══════════╪═══════════╡
    │ spam      │   41.9999 │
    ├───────────┼───────────┤
    │ eggs      │  451      │
    ╘═══════════╧═══════════╛
    */

    // Alignment can be changed like so:
    table.set_align(Align::Right, Align::Center);
    println!("{}", table.tabulate());
    /* Will print:
    ╒═══════════╤═══════════╕
    │   strings │  numbers  │
    ╞═══════════╪═══════════╡
    │      spam │  41.9999  │
    ├───────────┼───────────┤
    │      eggs │ 451.0000  │
    ╘═══════════╧═══════════╛
    */

    // With the feature "ansi_term_style", the borders style can be changed like so:
    table.set_border_style(ansi_term::Color::Cyan.bold());
    println!("{}", table.tabulate());
    // Will print same as above but with bold and colored borders
}
