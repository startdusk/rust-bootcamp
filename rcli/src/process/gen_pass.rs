use anyhow::Result;
use rand::seq::SliceRandom;

// 大写的I，O去掉，容易引发视觉上的误差导致的错误
const UPPER: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ";
// 小写的l，o去掉，容易引发视觉上的误差导致的错误
const LOWER: &[u8] = b"abcdefghijkmnpqrstuvwxyz";
// 数字0去掉，容易引发视觉上的误差导致的错误
const NUMBER: &[u8] = b"0123456789";
const SYMBOL: &[u8] = b"!@#$%^&*()_";

pub struct GenPassOpt {
    pub uppercase: bool,
    pub lowercase: bool,
    pub number: bool,
    pub symbol: bool,
    pub length: u8,
}

pub fn process_genpass(opts: GenPassOpt) -> Result<String> {
    let mut rng = rand::thread_rng();
    let mut password = Vec::new();
    let mut chars = Vec::new();

    // 保证password尽量都包含所有类型的字符
    if opts.uppercase {
        chars.extend_from_slice(UPPER);
        password.push(*UPPER.choose(&mut rng).expect("upper won't be empty"));
    }
    if opts.lowercase {
        chars.extend_from_slice(LOWER);
        password.push(*LOWER.choose(&mut rng).expect("lower won't be empty"));
    }
    if opts.number {
        chars.extend_from_slice(NUMBER);
        password.push(*NUMBER.choose(&mut rng).expect("number won't be empty"));
    }
    if opts.symbol {
        chars.extend_from_slice(SYMBOL);
        password.push(*SYMBOL.choose(&mut rng).expect("symbol won't be empty"));
    }
    for _ in 0..(opts.length - password.len() as u8) {
        let c = chars
            .choose(&mut rng)
            .expect("chars won't be empty in this context");
        password.push(*c)
    }

    // 重新打乱字符排序，避免有规律的生成(生成的密码应该是无规律)
    password.shuffle(&mut rng);
    let password = String::from_utf8(password)?;
    Ok(password)
}
