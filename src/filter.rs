const PROHIBITED_NAMES: &[&str] = &[
    "nigga",
    "nigger",
    "niggo",
    "hitler",
    "dick",
    "pussy",
    "gay",
    "faggot",
    "niga",
    "niger",
    "porn",
    "ass",
    "bastard",
    "bitch",
    "tits",
    "tity",
    "fuck",
    "penis",
    "anal",
    "sex",
    "balls",
    "piss",
    "orgasm",
    "masturbate",
    "cum",
    "pussies",
    "boner",
    "retard",
    "puta",
    "vagina",
    "pene",
];

pub const DEFAULT_USERNAME: &str = "Gasmoxian";

pub fn contains_prohibited_name(username: &str) -> bool {
    let bytes = username.as_bytes();
    PROHIBITED_NAMES.iter().any(|bad| {
        bytes.windows(bad.len()).any(|window| {
            window
                .iter()
                .zip(bad.bytes())
                .all(|(a, b)| a.eq_ignore_ascii_case(&b))
        })
    })
}
