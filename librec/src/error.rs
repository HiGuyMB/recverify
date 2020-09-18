
error_chain! {
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    foreign_links {
        Fmt(::std::fmt::Error);
        Io(::std::io::Error);
        Json(::serde_json::Error);
        FromUtf8(::std::string::FromUtf8Error);
    }

    errors {
        GenericError(t: &'static str) {
            description("Generic error")
            display("Generic error: {}", t)
        }
        GenericError2(t: String) {
            description("Generic error")
            display("Generic error: {}", t)
        }
    }
}
