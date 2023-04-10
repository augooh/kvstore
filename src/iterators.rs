use serde::de::DeserializeOwned;
use std::collections::hash_map;
use std::slice;

use crate::serialization::Serializer;

// 一个迭代器结构体，用于遍历一个 HashMap 中的键值对，
// 'a 是生命周期参数，用于指定该迭代器的生命周期与其所遍历的 HashMap 的生命周期相同。
pub struct KeyValueDbIterator<'a> {
    // map_iter 是一个 hash_map::Iter 类型的迭代器，它用于遍历一个 HashMap 中的键值对，
    // 其中的键是一个 String 类型，值是一个 Vec<u8> 类型。
    // serializer 是一个对序列化器（Serializer）的引用，它用于反序列化 Vec<u8> 类型的值。
    pub(crate) map_iter: hash_map::Iter<'a, String, Vec<u8>>,
    pub(crate) serializer: &'a Serializer,
}

// 使其可以通过 for-in 循环进行迭代。每次迭代会返回一个 KeyValueIteratorItem 实例的引用，
// 其中包含了一个 key 和一个 value 值，并可以通过 serializer 字段来对 value 进行序列化和反序列化。
impl<'a> Iterator for KeyValueDbIterator<'a> {
    type Item = KeyValueDbIteratorItem<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.map_iter.next() {
            Some((key, value)) => Some(KeyValueDbIteratorItem {
                key,
                value,
                serializer: self.serializer,
            }),
            None => None,
        }
    }
}

pub struct KeyValueDbIteratorItem<'a> {
    key: &'a str,
    value: &'a Vec<u8>,
    serializer: &'a Serializer,
}

impl<'a> KeyValueDbIteratorItem<'a> {
    /// Get the key
    pub fn get_key(&self) -> &str {
        self.key
    }

    pub fn get_value<V>(&self) -> Option<V>
    where
        V: DeserializeOwned,
    {
        self.serializer.deserialize_data::<V>(self.value)
    }
}

pub struct KeyValueDbListIterator<'a> {
    pub(crate) list_iter: slice::Iter<'a, Vec<u8>>,
    pub(crate) serializer: &'a Serializer,
}

impl<'a> Iterator for KeyValueDbListIterator<'a> {
    type Item = KeyValueDbListIteratorItem<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.list_iter.next() {
            Some(value) => Some(KeyValueDbListIteratorItem {
                value,
                serializer: self.serializer,
            }),
            None => None,
        }
    }
}


pub struct KeyValueDbListIteratorItem<'a> {
    value: &'a Vec<u8>,
    serializer: &'a Serializer,
}

// 将字节数组反序列化为泛型类型 V，并使用 serializer 字段返回一个 Option<V>
impl<'a> KeyValueDbListIteratorItem<'a> {
    pub fn get_item<V>(&self) -> Option<V>
    where
        V: DeserializeOwned,
    {
        self.serializer.deserialize_data(self.value)
    }
}
