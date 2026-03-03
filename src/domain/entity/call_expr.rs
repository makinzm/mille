/// A syntactically extracted call expression from source code.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct RawCallExpr {
    /// Source file where the call was found.
    pub file: String,
    /// 1-indexed line number of the call.
    pub line: usize,
    /// For static path calls (`Foo::method()`), the root type name ("Foo").
    /// `None` for instance method calls (`var.method()`) where the type is unknown.
    pub receiver_type: Option<String>,
    /// The method or function name being called.
    pub method: String,
}
