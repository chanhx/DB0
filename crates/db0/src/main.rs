use {
    clap::{arg, Command},
    db0::cmd::{self, Error as ExecutionError},
    snafu::prelude::*,
    std::{env, path::PathBuf, process, str::FromStr},
};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("the `DB0_DATADIR` environment variable is unset, you can pass a argument with `-d` to config"))]
    NoDataDirectory,

    ExecuteCommand {
        #[snafu(backtrace)]
        source: ExecutionError,
    },
}

pub type Result<T> = std::result::Result<T, Error>;

const DB0_DATADIR: &str = "DB0_DATADIR";

const INIT_DATABASE: &str = "initdb";

fn cli() -> Command {
    let pkg_name = env!("CARGO_PKG_NAME");

    Command::new(pkg_name)
        .bin_name(pkg_name)
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .subcommand_required(true)
        .subcommand(
            Command::new(INIT_DATABASE)
                .about("initialize the data directory")
                .arg(arg!(-d --data_dir <PATH> "data directory")),
        )
}

fn main() {
    if let Err(err) = try_main() {
        eprintln!("{}", err);
        process::exit(2);
    }
}

fn try_main() -> Result<()> {
    let matches = cli().get_matches();
    match matches.subcommand() {
        Some((INIT_DATABASE, sub_matches)) => {
            let data_dir = sub_matches.get_one::<String>("data_dir");
            let data_dir = match data_dir {
                Some(dir) => dir.into(),
                None => env::var(DB0_DATADIR).map_err(|_| Error::NoDataDirectory)?,
            };

            cmd::create_meta_tables(PathBuf::from_str(&data_dir).unwrap().as_path())
                .context(ExecuteCommandSnafu)?;
        }
        _ => unreachable!(),
    }

    Ok(())
}
