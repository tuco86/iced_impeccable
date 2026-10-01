//! `iced-impeccable ctl ...`, `iced-impeccable sheet ...`, `iced-impeccable help`.

const USAGE: &str =
    "usage: iced-impeccable ctl [--wait MS] [--keep-going] <socket> <command> [args...] | -
       iced-impeccable sheet [--cols N] [--labels a,b,c] [--max-width PX] OUT.png IN.png...
       iced-impeccable help";

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let code = match args.split_first() {
        Some((command, rest)) if command == "ctl" => iced_impeccable::cli::ctl(rest),
        Some((command, rest)) if command == "sheet" => iced_impeccable::cli::sheet(rest),
        Some((command, [])) if command == "help" => {
            println!("{}", iced_impeccable::cli::help());
            0
        }
        _ => {
            eprintln!("{USAGE}");
            2
        }
    };
    std::process::exit(code);
}
