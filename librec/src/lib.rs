extern crate derive_more;
extern crate nom;
extern crate regex;
extern crate wasm_bindgen;
extern crate cfg_if;
#[macro_use]
extern crate error_chain;
extern crate serde;
extern crate serde_json;

pub mod bit_stream;
pub mod recording;
pub mod tas_rec;
pub mod error;

use wasm_bindgen::prelude::*;
use cfg_if::cfg_if;
use crate::bit_stream::BitStream;
use crate::recording::Recording;
use crate::tas_rec::TasFile;
use crate::error::Result;

cfg_if! {
    // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
    // allocator.
    if #[cfg(feature = "wee_alloc")] {
        extern crate wee_alloc;
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
    }
}

#[wasm_bindgen]
extern {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn import_rec(conts: Vec<u8>) -> Option<String> {
    if let Ok(result) = import_opt(conts) {
        Some(result)
    } else {
        None
    }
}

fn import_opt(conts: Vec<u8>) -> Result<String> {
    let mut bs = BitStream::new(conts);

    let r = Recording::from_stream(&mut bs)?;
    let tf = serde_json::to_string(&r)?;
    Ok(tf)
}

#[wasm_bindgen]
pub fn export_rec(input: String) -> Vec<u8> {
    match export_opt(input) {
        Ok(mut result) => {
            result.insert(0, 1);
            result
        }
        Err(error) => {
            let mut result = format!("{:?}", error).into_bytes();
            result.insert(0, 0);
            result
        }
    }
}

fn export_opt(input: String) -> Result<Vec<u8>> {
    let r = if let Ok(r) = serde_json::from_str::<Recording>(&input) {
        r
    } else {
        let tf = TasFile::parse(input)?;
        tf.into_rec()
    };

    let mut os = BitStream::new(vec![]);
    r.into_stream(&mut os)?;
    Ok(os.bytes())
}

