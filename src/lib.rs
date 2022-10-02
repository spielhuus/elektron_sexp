pub mod error;
pub mod library;
pub mod pcb;
pub mod model;
pub mod parser;
pub mod schema;
pub mod shape;
pub mod write;

macro_rules! uuid {
    () => {
        Uuid::new_v4().to_string()
    };
}
pub(crate) use uuid;
