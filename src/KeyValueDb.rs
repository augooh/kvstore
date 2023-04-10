use serde::{de::DeserializeOwned, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::error::{Error, ErrorCode, Result};
use crate::extenders::KeyValueDbListExtender;
use crate::iterators::{KeyValueDbIterator, KeyValueDbListIterator};
use crate::serialization::SerializationMethod;
use crate::serialization::Serializer;

// 将键值对数据库中的更改自动存储到磁盘的四种策略
pub enum KeyValueDbDumpPolicy {
    // 永远不会将任何更改存储到文件中，文件始终保持只读。
    NeverDump,
    // 每次更改都会立即自动存储到文件中。
    AutoDump,
    // 除非用户主动请求将数据存储到文件中，否则数据不会自动存储。
    DumpUponRequest,
    // 更改将定期存储到文件中。
    // 每次有数据库更改时，如果自上次存储以来经过的时间比 Duration 更长，则会存储更改；
    // 否则，将不会存储更改。
    // Duration 是表示时间长度的 Rust 标准库结构体。
    PeriodicDump(Duration),
}

// 表示一个键值对数据库对象
pub struct KeyValueDb {
    map: HashMap<String, Vec<u8>>,
    list_map: HashMap<String, Vec<Vec<u8>>>,
    serializer: Serializer,
    db_file_path: PathBuf,
    dump_policy: KeyValueDbDumpPolicy,
    last_dump: Instant,
}

impl KeyValueDb {
    // 创建 KeyValueDb 实例的方法，参数为：
    // db_path：类型是 P，需要可以转换为 Path 类型，它指定了数据库存储的路径。

    // dump_policy：是一个枚举值，决定了数据库更改被转储到文件中的策略。选项在 KeyValueDbDumpPolicy 枚举中定义。

    // serialization_method：指定了用于将数据存储到内存和文件中的序列化方法，类型为 SerializationMethod。
    // 返回一个 KeyValueDb 。
    pub fn new<P: AsRef<Path>>(
        db_path: P,
        dump_policy: KeyValueDbDumpPolicy,
        serialization_method: SerializationMethod,
    ) -> KeyValueDb {
        let mut db_path_buf = PathBuf::new();
        db_path_buf.push(db_path);

        KeyValueDb {
            map: HashMap::new(),
            list_map: HashMap::new(),
            serializer: Serializer::new(serialization_method),
            db_file_path: db_path_buf,
            dump_policy,
            last_dump: Instant::now(),
        }
    }

    // 使用 SerializationMethod::Json 作为序列化方法，其他的实现和 new 方法相同。
    // 它的作用是创建一个使用 JSON 作为序列化格式的 KeyValueDb 实例，并将其存储在指定的路径中。
    #[cfg(feature = "json")]
    pub fn new_json<P: AsRef<Path>>(db_path: P, dump_policy: KeyValueDbDumpPolicy) -> KeyValueDb {
        KeyValueDb::new(db_path, dump_policy, SerializationMethod::Json)
    }

    // 使用 SerializationMethod::Json 作为序列化方法，其他的实现和 new 方法相同。
    // 创建一个使用 Bincode 作为序列化格式的 KeyValueDb 实例，并将其存储在指定的路径中。
    #[cfg(feature = "bincode")]
    pub fn new_bin<P: AsRef<Path>>(db_path: P, dump_policy: KeyValueDbDumpPolicy) -> KeyValueDb {
        KeyValueDb::new(db_path, dump_policy, SerializationMethod::Bin)
    }

    // 使用 SerializationMethod::Json 作为序列化方法，其他的实现和 new 方法相同。
    // 创建一个使用 CBOR 作为序列化格式的 KeyValueDb 实例，并将其存储在指定的路径中。
    #[cfg(feature = "cbor")]
    pub fn new_cbor<P: AsRef<Path>>(db_path: P, dump_policy: KeyValueDbDumpPolicy) -> KeyValueDb {
        KeyValueDb::new(db_path, dump_policy, SerializationMethod::Cbor)
    }

    // load 方法读取指定路径的 KeyValueDb 文件，并使用指定的序列化方法将其反序列化为 KeyValueDb 实例。

    // 如果文件读取成功，content 变量将包含文件内容。
    // 然后使用 Serializer 类型的 deserialize_db 方法将二进制数据转换回原始的哈希表和列表映射。
    // 如果反序列化失败，则返回一个错误。
    
    // 最后，创建一个新的 KeyValueDb 实例，包括读取的哈希表和列表映射、序列化器、DB 文件路径、存储策略和上一次写入文件的时间。
    // 如果所有操作都成功，则返回 Ok 包装的 KeyValueDb 实例。
    pub fn load<P: AsRef<Path>>(
        db_path: P,
        dump_policy: KeyValueDbDumpPolicy,
        serialization_method: SerializationMethod,
    ) -> Result<KeyValueDb> {
        let content = match fs::read(db_path.as_ref()) {
            Ok(file_content) => file_content,
            Err(err) => return Err(Error::new(ErrorCode::Io(err))),
        };

        let serializer = Serializer::new(serialization_method);

        let maps_from_file: (_, _) = match serializer.deserialize_db(&content) {
            Ok(maps) => maps,
            Err(err_str) => return Err(Error::new(ErrorCode::Serialization(err_str))),
        };

        let mut db_path_buf = PathBuf::new();
        db_path_buf.push(db_path);

        Ok(KeyValueDb {
            map: maps_from_file.0,
            list_map: maps_from_file.1,
            serializer,
            db_file_path: db_path_buf,
            dump_policy,
            last_dump: Instant::now(),
        })
    }

    // 在开启 Json 特性的情况下，调用通用的 load 方法，将 SerializationMethod::Json 作为序列化方法参数传递给 load 方法。
    // 如果加载成功，则返回 Ok 包装的 KeyValueDb 实例，否则返回错误。
    #[cfg(feature = "json")]
    pub fn load_json<P: AsRef<Path>>(
        db_path: P,
        dump_policy: KeyValueDbDumpPolicy,
    ) -> Result<KeyValueDb> {
        KeyValueDb::load(db_path, dump_policy, SerializationMethod::Json)
    }

    // 在开启 Bincode 特性的情况下，调用通用的 load 方法，将 SerializationMethod::Json 作为序列化方法参数传递给 load 方法。
    // 如果加载成功，则返回 Ok 包装的 KeyValueDb 实例，否则返回错误。
    #[cfg(feature = "bincode")]
    pub fn load_bin<P: AsRef<Path>>(
        db_path: P,
        dump_policy: KeyValueDbDumpPolicy,
    ) -> Result<KeyValueDb> {
        KeyValueDb::load(db_path, dump_policy, SerializationMethod::Bin)
    }

    // 在开启 Yaml 特性的情况下，调用通用的 load 方法，将 SerializationMethod::Json 作为序列化方法参数传递给 load 方法。
    // 如果加载成功，则返回 Ok 包装的 KeyValueDb 实例，否则返回错误。
    #[cfg(feature = "yaml")]
    pub fn load_yaml<P: AsRef<Path>>(
        db_path: P,
        dump_policy: KeyValueDbDumpPolicy,
    ) -> Result<KeyValueDb> { 
        KeyValueDb::load(db_path, dump_policy, SerializationMethod::Yaml)
    }

    // 在开启 CBOR 特性的情况下，调用通用的 load 方法，将 SerializationMethod::Json 作为序列化方法参数传递给 load 方法。
    // 如果加载成功，则返回 Ok 包装的 KeyValueDb 实例，否则返回错误。
    #[cfg(feature = "cbor")]
    pub fn load_cbor<P: AsRef<Path>>(
        db_path: P,
        dump_policy: KeyValueDbDumpPolicy,
    ) -> Result<KeyValueDb> { 
        KeyValueDb::load(db_path, dump_policy, SerializationMethod::Cbor)
    }

    // 加载指定路径的 KeyValueDb 文件，但将其配置为只读模式，不会将任何更改写入文件。
    pub fn load_read_only<P: AsRef<Path>>(
        db_path: P,
        serialization_method: SerializationMethod,
    ) -> Result<KeyValueDb> {
        KeyValueDb::load(db_path, KeyValueDbDumpPolicy::NeverDump, serialization_method)
    }

    // dump 方法用于将当前的键值存储到文件中。具体实现如下：
    // 首先，如果当前设置的存储策略是 NeverDump，则直接返回成功。
    // 接着，使用 Serializer 结构体的 serialize_db 方法将当前的键值对转化为二进制格式。
    // 如果转化成功，则将转化后的数据写入到临时文件中，临时文件名为当前数据库文件名加上 .temp 后缀加上当前时间戳的字符串表示。
    // 如果写入成功，则使用 fs::rename 方法将临时文件重命名为数据库文件，以保证写入的数据完整性。
    // 如果重命名成功，则如果当前存储策略为 PeriodicDump，则更新上一次存储的时间为当前时间。
    // 如果出现任何错误，则返回一个包含错误信息的 Result 类型。
    pub fn dump(&mut self) -> Result<()> {
        if let KeyValueDbDumpPolicy::NeverDump = self.dump_policy {
            return Ok(());
        }

        match self.serializer.serialize_db(&self.map, &self.list_map) {
            Ok(ser_db) => {
                let temp_file_path = format!(
                    "{}.temp.{}",
                    self.db_file_path.to_str().unwrap(),
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs()
                );

                match fs::write(&temp_file_path, ser_db) {
                    Ok(_) => (),
                    Err(err) => return Err(Error::new(ErrorCode::Io(err))),
                }

                match fs::rename(temp_file_path, &self.db_file_path) {
                    Ok(_) => (),
                    Err(err) => return Err(Error::new(ErrorCode::Io(err))),
                }

                if let KeyValueDbDumpPolicy::PeriodicDump(_dur) = self.dump_policy {
                    self.last_dump = Instant::now();
                }
                Ok(())
            }
            Err(err_str) => Err(Error::new(ErrorCode::Serialization(err_str))),
        }
    }

    // 根据当前备份策略进行判断，
    // 如果是 AutoDump 策略，则直接调用 dump 函数进行备份；
    // 如果是 PeriodicDump 策略，则判断距离上次备份的时间是否超过指定的时间间隔，如果超过则进行备份，否则不进行备份。最后返回执行结果。
    fn dumpdb(&mut self) -> Result<()> {
        match self.dump_policy {
            KeyValueDbDumpPolicy::AutoDump => self.dump(),
            KeyValueDbDumpPolicy::PeriodicDump(duration) => {
                let now = Instant::now();
                if now.duration_since(self.last_dump) > duration {
                    self.last_dump = Instant::now();
                    self.dump()?;
                }
                Ok(())
            }

            _ => Ok(()),
        }
    }

    // set 方法将一个序列化后的值与一个键关联起来，并将它们存储在 KeyValueDb 实例的内部哈希表中。键的类型是字符串，而值必须实现 Serialize trait。如果指定的键已经存在于 list_map 中，则先从其中删除。然后，该方法将指定的值序列化为字节数组，并插入到内部哈希表中。如果插入成功，则将其结果包装在 Ok 中返回。
    // 否则，该方法将尝试恢复先前哈希表中该键的原始值。
    // 如果无法恢复，它将返回一个错误。
    // 此外，如果存储策略允许，该方法还将调用 dumpdb 方法，将哈希表中的更改写入磁盘。
    pub fn set<V>(&mut self, key: &str, value: &V) -> Result<()>
    where
        V: Serialize,
    {
        if self.list_map.contains_key(key) {
            self.list_map.remove(key);
        }
        let ser_data = match self.serializer.serialize_data(value) {
            Ok(data) => data,
            Err(err_str) => return Err(Error::new(ErrorCode::Serialization(err_str))),
        };

        let original_value = self.map.insert(String::from(key), ser_data);
        match self.dumpdb() {
            Ok(_) => Ok(()),
            Err(err) => {
                match original_value {
                    None => {
                        self.map.remove(key);
                    }
                    Some(orig_value) => {
                        self.map.insert(String::from(key), orig_value.to_vec());
                    }
                }

                Err(err)
            }
        }
    }

    pub fn get<V>(&self, key: &str) -> Option<V>
    where
        V: DeserializeOwned,
    {
        match self.map.get(key) {
            Some(val) => self.serializer.deserialize_data::<V>(val),
            None => None,
        }
    }

    pub fn exists(&self, key: &str) -> bool {
        self.map.get(key).is_some() || self.list_map.get(key).is_some()
    }

    pub fn get_all(&self) -> Vec<String> {
        [
            self.map.keys().cloned().collect::<Vec<String>>(),
            self.list_map.keys().cloned().collect::<Vec<String>>(),
        ]
        .concat()
    }

    pub fn total_keys(&self) -> usize {
        self.map.iter().len() + self.list_map.iter().len()
    }

    pub fn rem(&mut self, key: &str) -> Result<bool> {
        let remove_map = match self.map.remove(key) {
            None => None,
            Some(val) => match self.dumpdb() {
                Ok(_) => Some(val),
                Err(err) => {
                    self.map.insert(String::from(key), val);
                    return Err(err);
                }
            },
        };

        let remove_list = match self.list_map.remove(key) {
            None => None,
            Some(list) => match self.dumpdb() {
                Ok(_) => Some(list),
                Err(err) => {
                    self.list_map.insert(String::from(key), list);
                    return Err(err);
                }
            },
        };

        Ok(remove_map.is_some() || remove_list.is_some())
    }


    pub fn lcreate(&mut self, name: &str) -> Result<KeyValueDbListExtender> {
        let new_list: Vec<Vec<u8>> = Vec::new();
        if self.map.contains_key(name) {
            self.map.remove(name);
        }
        self.list_map.insert(String::from(name), new_list);
        self.dumpdb()?;
        Ok(KeyValueDbListExtender {
            db: self,
            list_name: String::from(name),
        })
    }

    pub fn lexists(&self, name: &str) -> bool {
        self.list_map.get(name).is_some()
    }

    pub fn ladd<V>(&mut self, name: &str, value: &V) -> Option<KeyValueDbListExtender>
    where
        V: Serialize,
    {
        self.lextend(name, &[value])
    }

    pub fn lextend<'a, V, I>(&mut self, name: &str, seq: I) -> Option<KeyValueDbListExtender>
    where
        V: 'a + Serialize,
        I: IntoIterator<Item = &'a V>,
    {
        let serializer = &self.serializer;
        match self.list_map.get_mut(name) {
            Some(list) => {
                let original_len = list.len();
                let serialized: Vec<Vec<u8>> = seq
                    .into_iter()
                    .map(|x| serializer.serialize_data(x).unwrap())
                    .collect();
                list.extend(serialized);
                match self.dumpdb() {
                    Ok(_) => (),
                    Err(_) => {
                        let same_list = self.list_map.get_mut(name).unwrap();
                        same_list.truncate(original_len);
                        return None;
                    }
                }
                Some(KeyValueDbListExtender {
                    db: self,
                    list_name: String::from(name),
                })
            }

            None => None,
        }
    }

    pub fn lget<V>(&self, name: &str, pos: usize) -> Option<V>
    where
        V: DeserializeOwned,
    {
        match self.list_map.get(name) {
            Some(list) => match list.get(pos) {
                Some(val) => self.serializer.deserialize_data::<V>(val),
                None => None,
            },
            None => None,
        }
    }

    pub fn llen(&self, name: &str) -> usize {
        match self.list_map.get(name) {
            Some(list) => list.len(),
            None => 0,
        }
    }

    pub fn lrem_list(&mut self, name: &str) -> Result<usize> {
        let res = self.llen(name);
        match self.list_map.remove(name) {
            Some(list) => match self.dumpdb() {
                Ok(_) => Ok(res),
                Err(err) => {
                    self.list_map.insert(String::from(name), list);
                    Err(err)
                }
            },
            None => Ok(res),
        }
    }

    pub fn lpop<V>(&mut self, name: &str, pos: usize) -> Option<V>
    where
        V: DeserializeOwned,
    {
        match self.list_map.get_mut(name) {
            Some(list) => {
                if pos < list.len() {
                    let res = list.remove(pos);
                    match self.dumpdb() {
                        Ok(_) => self.serializer.deserialize_data::<V>(&res),
                        Err(_) => {
                            let same_list = self.list_map.get_mut(name).unwrap();
                            same_list.insert(pos, res);
                            None
                        }
                    }
                } else {
                    None
                }
            }

            None => None,
        }
    }

    pub fn lrem_value<V>(&mut self, name: &str, value: &V) -> Result<bool>
    where
        V: Serialize,
    {
        match self.list_map.get_mut(name) {
            Some(list) => {
                let serialized_value = match self.serializer.serialize_data(&value) {
                    Ok(val) => val,
                    Err(err_str) => return Err(Error::new(ErrorCode::Serialization(err_str))),
                };

                match list.iter().position(|x| *x == serialized_value) {
                    Some(pos) => {
                        list.remove(pos);
                        match self.dumpdb() {
                            Ok(_) => Ok(true),
                            Err(err) => {
                                let same_list = self.list_map.get_mut(name).unwrap();
                                same_list.insert(pos, serialized_value);
                                Err(err)
                            }
                        }
                    }

                    None => Ok(false),
                }
            }

            None => Ok(false),
        }
    }

    pub fn iter(&self) -> KeyValueDbIterator {
        KeyValueDbIterator {
            map_iter: self.map.iter(),
            serializer: &self.serializer,
        }
    }

    pub fn liter(&self, name: &str) -> KeyValueDbListIterator {
        match self.list_map.get(name) {
            Some(list) => KeyValueDbListIterator {
                list_iter: list.iter(),
                serializer: &self.serializer,
            },
            None => panic!("List '{}' doesn't exist", name),
        }
    }
}

// Drop 实现的作用是，如果 self.dump_policy 不是 NeverDump 或 DumpUponRequest 时，
// 则尝试进行一次 dump 操作来保存数据库的内容。
// 这是为了确保即使程序意外崩溃或被非正常终止，数据库中的数据也能够被尽可能地保存下来，避免数据丢失的情况发生。
impl Drop for KeyValueDb {
    fn drop(&mut self) {
        if !matches!(
            self.dump_policy,
            KeyValueDbDumpPolicy::NeverDump | KeyValueDbDumpPolicy::DumpUponRequest
        ) {
            // try to dump, ignore if fails
            let _ = self.dump();
        }
    }
}