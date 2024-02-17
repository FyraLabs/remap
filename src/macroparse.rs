#[macro_export]
macro_rules! generate_expr_parser {
    ($KeydExpr:ident; $(
        $act:ident($($n:literal: $param:ident:$desc:literal),*) => $enumid:ident : $actdesc:literal
    )+) => {
        pub enum $KeydExpr {
            Key(String),
            Noop,
            $($enumid($($crate::generate_expr_parser!(@type $param)),*)),+
        }
        fn is_valid_ident(s: &str) -> bool {
            !s.contains(|c: char| !c.is_ascii_alphanumeric())
                || s.len() == 1 && r#"`~!@#$%^&*()-_=+[]{}\|;:'",./<>?"#.contains(s)
        }
        impl $KeydExpr {
            fn parse(s: &str) -> Res<Self> {
                if is_valid_ident(s) {
                    return Ok(Self::Key(s.to_string()));
                }
                let r = s.strip_suffix(')').ok_or_else(|| ParseError::InvalidIdent(s.to_string()))?;
                let (call, sargs) = r.split_once('(').ok_or_else(|| ParseError::InvalidIdent(s.to_string()))?;
                match call {
                    $(
                        stringify!($act) => {::paste::paste! {
                            let [$([<arg_ $n>]),*] = &sargs.split(',').collect::<Vec<_>>()[..] else {
                                return Err(ParseError::BadArgs(stringify!($act).to_string(), s.to_string()));
                            };
                            return Ok(Self::$enumid($($crate::generate_expr_parser!(@genparse $param [<arg_ $n>])),*));
                        }},
                    )+
                    _ => return Err(ParseError::InvalidCall(call.to_string())),
                }
            }
        }
    };
    (@type layer) => { String };
    (@type layout) => { String };
    (@type macro) => { Macro };
    (@type timeout) => { u16 };
    (@type cmd) => { String };
    (@type act) => { KeydExpr };
    (@type $any:ident) => { compile_error!("Unknown $param {}", stringify!($any))};
    (@genparse layer $arg:ident) => {{
        if $arg.contains(|c: char| !c.is_ascii_alphanumeric()) {
            return Err(ParseError::InvalidIdent($arg.to_string()))
        }
        $arg.to_string()
    }};
    (@genparse layout $arg:ident) => {{
        $arg.to_string()
    }};
    (@genparse macro $arg:ident) => {{
        if $arg.len() == 1 {
            vec![MacroToken::Text($arg.to_string())]
        } else if let Some(x) = MacroToken::try_into_combination($arg) {
            vec![x]
        } else if let Some(x) = remove_wrap($arg, "macro(", ")"){
            parse_macro(x)?
        } else {
            return Err(ParseError::InvalidIdent($arg.to_string()))
        }
    }};
    (@genparse timeout $arg:ident) => {{
        $arg.parse::<u16>()?
    }};
    (@genparse cmd $arg:ident) => {{
        $arg.to_string()
    }};
    (@genparse act $arg:ident) => {{
        KeydExpr::parse($arg)?
    }};
}
