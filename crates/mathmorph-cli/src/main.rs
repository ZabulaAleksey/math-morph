use std::env;
use std::process;

fn main() {
    let result = mathmorph_cli::parse_args(env::args_os().skip(1)).and_then(mathmorph_cli::execute);
    match result {
        Ok(summary) => {
            println!("{summary}");
            process::exit(0);
        }
        Err(error) => {
            eprintln!("{}", mathmorph_cli::render_error(&error));
            process::exit(error.exit_code().value());
        }
    }
}
