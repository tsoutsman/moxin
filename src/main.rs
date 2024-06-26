use std::fs::File;

fn main() {
    println!("Moxin was invoked with args: {:#?}", std::env::args().collect::<Vec<_>>());

    use std::io::Write;
    std::fs::write("/Users/klim/Projects/robius/shared-directory-utm/moxin/log", "a").unwrap();

    robius_url_handler::register_handler(handler);
    let mut file = File::options().append(true).open("/Users/klim/Projects/robius/shared-directory-utm/moxin/log").unwrap();
    write!(file, "c").unwrap();
    moxin::app::app_main()
}

fn handler(s: &str) {
    use std::io::Write;
    let mut file = File::options().append(true).open("/Users/klim/Projects/robius/shared-directory-utm/moxin/log").unwrap();
    write!(file, "\n").unwrap();
    write!(file, "ad: {}", s).unwrap();
}