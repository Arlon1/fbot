use std::io::{stdout, Write};

extern crate curl;
use curl::easy::Easy;


fn main() {
    print!("testKF\n");

    let mut easy = Easy::new();
    easy.url("https://chat.qed-verein.de/").unwrap();
    easy.write_function(|data| {
        stdout().write_all(data).unwrap();
        Ok(data.len())
    }).unwrap();
    easy.perform().unwrap();

    println!("{}", easy.response_code().unwrap());
}
