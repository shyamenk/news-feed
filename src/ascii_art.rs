#[allow(dead_code)]
pub const QUOTES: &[&str] = &[
    "\"Stay curious, keep reading.\"",
    "\"Knowledge is the new currency.\"",
    "\"Every article is a window to the world.\"",
    "\"Read widely, think deeply.\"",
    "\"Information is the oxygen of the modern age.\"",
    "\"The more you read, the more you know.\"",
    "\"Feed your mind, one article at a time.\"",
    "\"Stay informed, stay ahead.\"",
    "\"Reading is to the mind what exercise is to the body.\"",
    "\"A reader lives a thousand lives.\"",
    "\"Today a reader, tomorrow a leader.\"",
    "\"Books are a uniquely portable magic.\"",
];

#[allow(dead_code)]
pub fn get_random_quote() -> &'static str {
    use std::time::{SystemTime, UNIX_EPOCH};
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let index = (seed % QUOTES.len() as u64) as usize;
    QUOTES[index]
}
