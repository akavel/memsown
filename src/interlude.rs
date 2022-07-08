pub use std::sync::Mutex;

pub use anyhow::anyhow;
pub use ifmt::{ieprintln, iformat as ifmt, iprint, iprintln};

pub fn error_chain(err: &anyhow::Error) -> String {
    err.chain()
        .into_iter()
        .map(|e| e.to_string())
        .collect::<Vec<String>>()
        .join(": ")
}

// use anyhow::anyhow;
// // TODO: how to properly export it?? like ifmt! above?
// #[macro_export]
// macro_rules! ianyhow {
//     ($($arg:tt)*) => {
//         anyhow!(ifmt!( $($arg)* ))
//     };
// }
