pub struct FlagInfo {
    pub short: &'static [char],
    pub long: &'static [&'static str],
    pub description: &'static str,
}
pub struct CommandInfo {
    pub name: &'static str,
    pub description: &'static str,
}
