use std::fs::File;

const LOG_FILE: &str = "/Users/klim/Projects/robius/shared-directory-utm/moxin/log.txt";

fn main() {
    println!("Moxin was invoked with args: {:#?}", std::env::args().collect::<Vec<_>>());

    use std::io::Write;
    robius_url_handler::register_handler(handler);
    moxin::app::app_main()
}

fn handler(s: &str) {
    use std::io::Write;
    let mut file = File::options().append(true).open(LOG_FILE).unwrap();
    write!(file, "handled: {}\n", s).unwrap();
}
