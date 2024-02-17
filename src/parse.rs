use std::{
    fs::File,
    io::{BufRead, BufReader},
    str::FromStr,
};

fn remove_wrap<'a>(s: &'a str, left: &'a str, right: &'a str) -> Option<&'a str> {
    (s.strip_prefix(left).and_then(|s| s.strip_suffix(right))).map(|s| s.trim())
}

pub enum ParseError {
    InvalidIdent(String),
    IoErr(std::io::Error),
    InvalidInt(std::num::ParseIntError),
    BadArgs(String, String), // fn name, args
    InvalidCall(String),
}

impl From<std::io::Error> for ParseError {
    fn from(value: std::io::Error) -> Self {
        Self::IoErr(value)
    }
}

impl From<std::num::ParseIntError> for ParseError {
    fn from(value: std::num::ParseIntError) -> Self {
        Self::InvalidInt(value)
    }
}

type Res<T> = Result<T, ParseError>;

#[derive(Clone)]
pub enum MacroToken {
    Key(String),
    Combination(Box<[String]>), // Type 2 macros
    Text(String),
    Hold(Box<[String]>),
    Timeout(u16),
}

impl MacroToken {
    fn try_into_combination(s: &str) -> Option<Self> {
        // using `while` for `break`
        let [lefts @ .., last] = &s.split('-').collect::<Vec<_>>()[..] else {
            return None;
        };
        if !KeydExpr::is_valid_ident(last) {
            return None;
        }
        // if let Some(x) = lefts.iter().find(|s| s.len() != 1 || !"ACMS".contains(s.chars().next().unwrap()))
        if lefts
            .iter()
            .find(|s| s.len() != 1 || !"ACMS".contains(s.chars().next().unwrap()))
            .is_some()
        {
            return None;
        }
        let mut all = lefts.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        all.push(last.to_string());
        Some(Self::Combination(all.into_boxed_slice()))
    }
}

impl std::str::FromStr for MacroToken {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if KeydExpr::is_valid_ident(s) {
            return Ok(Self::Key(s.to_string()));
        }
        // TODO: try_into_combination
        // using `while` for `break`
        while let [lefts @ .., last] = &s.split('-').collect::<Vec<_>>()[..] {
            if !KeydExpr::is_valid_ident(last) {
                // return Err(ParseError::InvalidIdent(last.to_string()));
                break;
            }
            // if let Some(x) = lefts.iter().find(|s| s.len() != 1 || !"ACMS".contains(s.chars().next().unwrap()))
            if lefts
                .iter()
                .find(|s| s.len() != 1 || !"ACMS".contains(s.chars().next().unwrap()))
                .is_some()
            {
                // return Err(ParseError::InvalidIdent(x.to_string()));
                break;
            }
            let mut all = lefts.iter().map(|s| s.to_string()).collect::<Vec<_>>();
            all.push(last.to_string());
            return Ok(Self::Combination(all.into_boxed_slice()));
        }
        {
            let mut keys = s.split('+');
            if keys.all(|k| KeydExpr::is_valid_ident(k)) {
                return Ok(Self::Hold(keys.map(|s| s.to_string()).collect()));
            }
        }
        if let Ok(t) = s.parse() {
            return Ok(Self::Timeout(t));
        }
        Ok(Self::Text(s.to_string()))
    }
}

type Macro = Vec<MacroToken>;

// NOTE: Not sure if this is 100% compatible.
pub fn parse_macro(arg: &str) -> Res<Macro> {
    let mut iter = arg.chars();
    let mut macros = vec![];
    let mut wordbuf = String::new();
    let mut depth: u8 = 0;
    while let Some(c) = iter.next() {
        if c == ')' {
            if depth == 1 {
                depth = 0;
                macros.append(&mut parse_macro(&wordbuf)?);
                wordbuf.clear();
                continue;
            }
            if depth > 0 {
                depth -= 1;
            }
            wordbuf.push(c);
            continue;
        }
        if c == ' ' {
            if wordbuf.is_empty() {
                continue;
            }
            macros.push(MacroToken::from_str(&wordbuf)?);
            wordbuf.clear();
            continue;
        }
        if c == '(' && wordbuf == "macro" {
            wordbuf.clear();
            depth += 1;
            continue;
        }
        wordbuf.push(c);
    }
    if !wordbuf.is_empty() {
        macros.push(MacroToken::from_str(&wordbuf)?);
    }
    Ok(macros)
}

crate::generate_expr_parser!(KeydExpr2;
    layer(1: layer: "") => Layer : ""
    oneshot(1: layer: "") => Oneshot : ""
    swap(1: layer: "") => Swap : ""
    setlayout(1: layout: "") => SetLayout : ""
    clear() => Clear : ""
    toggle(1: layout: "") => Toggle : ""
    layerm(1: layer: "", 2: macro: "") => Layerm : ""
    oneshotm(1: layer: "", 2: macro: "") => Oneshotm : ""
    swapm(1: layer: "", 2: macro: "") => Swapm : ""
    togglem(1: layer: "", 2: macro: "") => Togglem : ""
    clearm(1: macro: "") => Clearm : ""
    overload(1: layer: "", 2: act: "") => Overload : ""
    overloadt(1: layer: "", 2: act: "", 3: timeout: "") => Overloadt : ""
    overloadt2(1: layer: "", 2: act: "", 3: timeout: "") => Overloadt2 : ""
    timeout(1: act: "", 2: timeout: "", 3: act: "") => Timeout : ""
    macro2(1: timeout: "", 2: timeout: "", 3: macro: "") => Macro2 : ""
    command(1: cmd: "") => Command : ""
);

pub enum KeydExpr {
    Key(String),
    Layer(String),
    Oneshot(String),
    Swap(String),
    SetLayout(String),
    Clear,
    Toggle(String),
    Timeout(Box<Self>, u16, Box<Self>),
}

impl KeydExpr {
    // FIXME: actually check if the keys actually exist
    fn is_valid_ident(s: &str) -> bool {
        !s.contains(|c: char| !c.is_ascii_alphanumeric())
            || s.len() == 1 && r#"`~!@#$%^&*()-_=+[]{}\|;:'",./<>?"#.contains(s)
    }
    fn parse(s: &str) -> Res<Self> {
        if Self::is_valid_ident(s) {
            return Ok(Self::Key(s.to_string()));
        }
        macro_rules! action {
            ($name:ident => $exprtype:ident) => {
                if let Some($name) = remove_wrap(s, concat!(stringify!($name), "("), ")") {
                    return if Self::is_valid_ident($name) {
                        Ok(Self::$exprtype($name.to_string()))
                    } else {
                        Err(ParseError::InvalidIdent($name.to_string()))
                    };
                }
            };
        }
        action!(layer => Layer);
        action!(oneshot => Oneshot);
        action!(swap => Swap);
        action!(setlayout => SetLayout);
        if remove_wrap(s, "clear(", ")").map_or(false, |s| s.trim().len() == 0) {
            return Ok(Self::Clear);
        }
        action!(toggle => Toggle);
        if let Some(stimeout) = remove_wrap(s, "timeout(", ")") {
            return if let [act1, time, act2] = &*stimeout.split(',').collect::<Vec<_>>() {
                let (act1, act2) = (Self::parse(act1.trim())?, Self::parse(act2.trim())?);
                let time = time.trim();
                Ok(Self::Timeout(Box::new(act1), time.parse()?, Box::new(act2)))
            } else {
                Err(ParseError::BadArgs(
                    "timeout".to_string(),
                    stimeout.to_string(),
                ))
            };
        }

        todo!()
    }
}

pub enum KeydStatement {
    Star,
    SetId(String, String),
    UnsetId(String, String),
    Define(String, KeydExpr),
}

impl KeydStatement {
    fn parse(left: &str, right: &str) -> Res<Self> {
        if left.contains(|c: char| !c.is_ascii_lowercase()) {
            return Err(ParseError::InvalidIdent(left.to_string()));
        }
        let left = left.to_string();
        let right = KeydExpr::parse(right)?;
        Ok(Self::Define(left, right))
    }
}

pub struct KeydSection {
    name: String,
    content: Vec<KeydStatement>,
}

impl KeydSection {
    pub fn parse(name: &str, file: &mut BufReader<File>) -> Res<(Self, Option<String>)> {
        let mut content = vec![];
        loop {
            let mut buf = String::new();
            if file.read_line(&mut buf)? == 0 {
                let name = name.to_string();
                return Ok((Self { name, content }, None));
            }
            // remove comments
            let Some((buf, _)) = buf.split_once('#') else {
                continue; // line is empty
            };
            if buf.is_empty() {
                continue;
            }

            if let Some((left, right)) = buf.split_once('=') {
                content.push(KeydStatement::parse(left.trim(), right.trim())?);
            }
        }
    }
}

pub struct KeydParser {
    file: BufReader<File>,
    sections: Vec<KeydSection>,
}

impl KeydParser {
    pub fn new() -> std::io::Result<Self> {
        Ok(Self {
            file: BufReader::new(File::open("/etc/keyd/default.conf")?),
            sections: vec![],
        })
    }

    pub fn parse(&mut self) -> Res<()> {
        loop {
            let mut buf = String::new();
            if self.file.read_line(&mut buf)? == 0 {
                return Ok(()); // EOF
            }
            // remove comments
            let Some((buf, _)) = buf.split_once('#') else {
                continue; // line is empty
            };
            if buf.is_empty() {
                continue;
            }
            if let Some(name) = remove_wrap(buf, "[", "]") {
                let (section, next) = KeydSection::parse(name, &mut self.file)?;
            }
        }
    }
}
