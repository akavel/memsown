pub use ifmt::{iprintln, iformat as ifmt};

pub fn error_chain(err: &anyhow::Error) -> String {
    err.chain()
        .into_iter()
        .map(|e| e.to_string())
        .collect::<Vec<String>>()
        .join(": ")
}
