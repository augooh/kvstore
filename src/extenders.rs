use crate::keyvaluedb::KeyValueDb;
use serde::Serialize;

// ?
pub struct KeyValueDbListExtender<'a> {
    pub(crate) db: &'a mut KeyValueDb,
    pub(crate) list_name: String,
}

impl<'a> KeyValueDbListExtender<'a> {
    // 向列表末尾添加一个新元素。
    pub fn ladd<V>(&mut self, value: &V) -> KeyValueDbListExtender
    where
        V: Serialize,
    {
        self.db.ladd(&self.list_name, value).unwrap()
    }

    // 向列表末尾批量添加新元素。
    pub fn lextend<'i, V, I>(&mut self, seq: I) -> KeyValueDbListExtender
    where
        V: 'i + Serialize,
        I: IntoIterator<Item = &'i V>,
    {
        self.db.lextend(&self.list_name, seq).unwrap()
    }
}

// 在上述示例中，我们首先创建了一个 KeyValueDb 实例，
// 然后创建了一个 KeyValueDbListExtender 实例，并将其 db 字段设置为指向上述 KeyValueDb 实例的可变引用，将 list_name 字段设置为 "example_list"。
// 接着，我们使用 ladd 方法向列表中添加了一个新元素，使用 lextend 方法批量添加了多个新元素。
// 由于 ladd 和 lextend 方法都会返回修改后的 KeyValueDbListExtender 实例，因此我们可以将其存储到变量中以便后续操作。