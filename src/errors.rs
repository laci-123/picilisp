#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    AmbiguousName(Vec<String>),
    GlobalNonExistentOrPrivate,
    ModuleNonExistent,
    NoSuchModule,
}
