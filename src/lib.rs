// 该模块导出了 KeyValueDb crate 中的所有公共接口，
// 包括了对数据库的读写、数据迭代器、序列化方法、错误等。

pub use self::extenders::KeyValueDbListExtender;
pub use self::iterators::{
    KeyValueDbIterator, KeyValueDbIteratorItem, KeyValueDbListIterator, KeyValueDbListIteratorItem,
};
pub use self::keyvaluedb::{KeyValueDb, KeyValueDbDumpPolicy};
pub use self::serialization::SerializationMethod;

mod extenders;
mod iterators;
mod keyvaluedb;
mod serialization;

pub mod error;
