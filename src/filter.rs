// mods/Windows/Gasmoxian/Network_PC/GClient/GASMOX_CLIENT.cpp:1704-1746

const PROHIBITED_NAMES: [&str; 32] = [
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
    "fuck",
    "pene",
];

// Checks if the name contains any prohibited words. The name is expected to be a null-terminated string, and the check is case-insensitive.
pub fn contains_prohibited_name(name: &[u8]) -> bool {
    let s = String::from_utf8_lossy(name)
        .trim_end_matches('\0')
        .to_ascii_lowercase();
    PROHIBITED_NAMES.iter().any(|bad| s.contains(bad))
}
