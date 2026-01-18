use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "ixchel", version)]
#[command(about = "Ixchel (ik-SHEL) — git-first knowledge weaving", long_about = None)]
struct Args {}

fn main() {
    let _args = Args::parse();
    println!("ixchel (skeleton) — run with --help");
}
