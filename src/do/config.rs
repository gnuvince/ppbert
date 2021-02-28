#[derive(Debug)]
pub struct Config {
    pub transform_proplists: bool,
    pub indent: usize,
    pub short_collection: usize,
    pub terminator: &'static str,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            transform_proplists: false,
            indent: 2,
            short_collection: 8,
            terminator: ".",
        }
    }
}
