#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    GlobalNameInMultipleModules,
    GlobalNonExistentOrPrivate,
}
