const PROHIBITED_NAMES: &[&[u8]] = &[
    b"nigga",
    b"nigger",
    b"niggo",
    b"hitler",
    b"dick",
    b"pussy",
    b"gay",
    b"faggot",
    b"niga",
    b"niger",
    b"porn",
    b"ass",
    b"bastard",
    b"bitch",
    b"tits",
    b"tity",
    b"fuck",
    b"penis",
    b"anal",
    b"sex",
    b"balls",
    b"piss",
    b"orgasm",
    b"masturbate",
    b"cum",
    b"pussies",
    b"boner",
    b"retard",
    b"puta",
    b"vagina",
    b"pene",
];

/// Checks if a null-terminated byte slice contains any prohibited word (case-insensitive).
pub fn contains_prohibited_name(name: &[u8]) -> bool {
    let trimmed = match name.iter().position(|&b| b == b'\0') {
        Some(pos) => &name[..pos],
        None => name,
    };

    PROHIBITED_NAMES.iter().any(|&bad| {
        trimmed
            .windows(bad.len())
            .any(|window| window.eq_ignore_ascii_case(bad))
    })
}
