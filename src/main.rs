mod app;

use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Arguments {
    filename: String,
    #[arg(short, long, default_value_t = false)]
    debug: bool,
}

fn main() {
    let args = Arguments::parse();
    app::app(&args.filename, args.debug);
}
