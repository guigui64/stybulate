use stybulate::*;

fn main() {
    // Wide characters are handled correctly when computing columns length:
    let table = Table::new(
        Style::Grid,
        vec![
            vec![Cell::from("Tabulate with style!")],
            vec![Cell::from("табулировать со стилем!")],
            vec![Cell::from("スタイルで集計！")],
            vec![Cell::from("用样式表！")],
        ],
        None,
    );
    println!("{}", table.tabulate());
    /* Will print:
    +-------------------------+
    | Tabulate with style!    |
    +-------------------------+
    | табулировать со стилем! |
    +-------------------------+
    | スタイルで集計！        |
    +-------------------------+
    | 用样式表！              |
    +-------------------------+
    */
}
