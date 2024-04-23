mod base64;
mod csv;
mod genpass;
mod http;
mod jwt;
mod text;

use std::path::{Path, PathBuf};

pub use self::{base64::*, csv::*, genpass::*, http::*, jwt::*, text::*};

use clap::Parser;
use enum_dispatch::enum_dispatch;

#[derive(Debug, Parser)]
#[command(name = "rcli", version, author, about, long_about = None)]
pub struct Opts {
    #[command(subcommand)]
    pub cmd: Subcommand,
}

#[derive(Debug, Parser)]
#[enum_dispatch(CmdExecutor)]
pub enum Subcommand {
    #[command(name = "csv", about = "Convert CSV to JSON")]
    Csv(CsvOpts),

    #[command(name = "genpass", about = "Generate a random password")]
    GenPass(GenPassOpts),

    #[command(subcommand, about = "Base64 encode/decode")]
    Base64(Base64SubCommand),

    #[command(subcommand, about = "Text sign/verify")]
    Text(TextSubCommand),

    #[command(subcommand, about = "HTTP Server")]
    Http(HttpSubCommand),

    #[command(subcommand, about = "Json web token")]
    Jwt(JwtSubCommand),
}

// 使用 enum_dispatch 实现CmdExecutor, 等价于下面的代码
// impl CmdExecutor for Subcommand {
//     async fn execute(self) -> anyhow::Result<()> {
//         // match self {
//         //     Subcommand::Csv(opts) => opts.execute().await,
//         //     Subcommand::GenPass(opts) => opts.execute().await,
//         //     Subcommand::Base64(subcmd) => subcmd.execute().await,
//         //     Subcommand::Text(subcmd) => subcmd.execute().await,
//         //     Subcommand::Http(subcmd) => subcmd.execute().await,
//         // }
//     }
// }

fn verify_file(filename: &str) -> Result<String, &'static str> {
    if filename == "-" || Path::new(filename).exists() {
        return Ok(filename.into());
    }

    Err("File does not exist")
}

fn verify_path(path: &str) -> Result<PathBuf, &'static str> {
    let p = Path::new(path);
    if !p.exists() || !p.is_dir() {
        return Err("Path does not exist or is not a directory");
    }
    Ok(path.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_input_file() {
        assert_eq!(verify_file("-"), Ok("-".to_string()));
        assert_eq!(verify_file("*"), Err("File does not exist"));
        assert_eq!(verify_file("Cargo.toml"), Ok("Cargo.toml".to_string()));
        assert_eq!(verify_file("not-exist"), Err("File does not exist"));
    }
}
