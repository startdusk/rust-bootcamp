use clap::Parser;

use crate::{process_genpass, CmdExecutor, GenPassOpt};
use zxcvbn::zxcvbn;

#[derive(Debug, Parser)]
pub struct GenPassOpts {
    #[arg(short, long, default_value_t = 16)]
    pub length: u8,

    #[arg(long, default_value_t = true)]
    pub uppercase: bool,

    #[arg(long, default_value_t = true)]
    pub lowercase: bool,

    #[arg(long, default_value_t = true)]
    pub number: bool,

    #[arg(long, default_value_t = true)]
    pub symbol: bool,
}

impl CmdExecutor for GenPassOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let password = process_genpass(GenPassOpt {
            uppercase: self.uppercase,
            lowercase: self.lowercase,
            length: self.length,
            symbol: self.symbol,
            number: self.number,
        })?;
        println!("{}", &password);
        // output password strength in stderr(输入到stderr，是为了避免不和stdout的输出混在一起)
        let estimate = zxcvbn(&password, &[])?;
        eprintln!("Password strength: {}", estimate.score());
        Ok(())
    }
}
