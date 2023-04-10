use std::fmt;
use std::io;
use std::result;

// ErrorType 是一个枚举类型，它表示了可能出现的错误类型，包括 Io 和 Serialization。其中，Io 表示 I/O 错误，例如文件读写错误等，Serialization 表示序列化错误，例如在使用 JSON 或者 bincode 时遇到的错误。
// 该枚举类型可以用于在程序中捕获特定类型的错误并进行相应处理。
#[derive(Debug)]
pub enum ErrorType {
    Io,
    Serialization,
}

// Error 结构体，其中包含一个 err_code 字段，类型为 ErrorCode 枚举类型。 
// ErrorCode 枚举类型包含了不同的错误码，例如 Io 或 Serialization。
// 通过这种方式可以在代码中捕获和处理不同类型的错误。
pub struct Error {
    err_code: ErrorCode,
}

// Result 类型的别名
pub type Result<T> = result::Result<T, Error>;

impl Error {
    // 创建一个新的Error实例
    pub(crate) fn new(err_code: ErrorCode) -> Error {
        Error { err_code }
    }

    // 通过匹配err_code字段的值来确定错误类型，
    // 如果错误代码是Io类型的，那么返回ErrorType::Io，
    // 如果是Serialization类型的，则返回ErrorType::Serialization。
    pub fn get_type(&self) -> ErrorType {
        match self.err_code {
            ErrorCode::Io(_) => ErrorType::Io,
            ErrorCode::Serialization(_) => ErrorType::Serialization,
        }
    }
}

// 通过匹配错误代码（err_code）来确定错误类型
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.err_code {
            ErrorCode::Io(ref err) => fmt::Display::fmt(err, f),
            ErrorCode::Serialization(ref err_str) => f.write_str(err_str),
        }
    }
}

// 当需要在调试信息中打印 Error 类型时，该 trait 方法会被调用。
// 它会将错误类型转换为字符串并格式化为类似于 Error { msg: ... } 的字符串，其中 ... 为错误消息。
impl fmt::Debug for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(&format!(
            "Error {{ msg: {} }}",
            match self.err_code {
                ErrorCode::Io(ref err) => err.to_string(),
                ErrorCode::Serialization(ref err_str) => err_str.to_string(),
            }
        ))
    }
}

// 实现该 trait，可以为 Error 结构体提供更多的错误处理特性，
// 比如使用 ? 运算符捕获错误、自定义错误类型等。
impl std::error::Error for Error {}

// ErrorCode 和 ErrorType 都是用来表示错误类型的枚举类型，但是它们在设计上有不同的目的和用途。

// ErrorCode 主要用来在程序内部标识和处理错误，它包含了具体的错误信息，
// 例如 I/O 错误和序列化错误。在程序中捕获到错误时，可以通过 ErrorCode 的类型来判断错误的具体类型，并根据需要进行相应的处理。

// ErrorType 则主要用来向用户或调用者展示错误信息的高层次抽象。
// 它是对 ErrorCode 的简化和归纳，只包含了更一般性的错误类型，例如 I/O 错误和序列化错误可以归为文件操作错误和数据处理错误两类。
// 在向用户或调用者报告错误时，可以使用 ErrorType 来描述错误的大致类型，并根据需要提供更详细的错误信息。
pub(crate) enum ErrorCode {
    Io(io::Error),
    Serialization(String),
}
