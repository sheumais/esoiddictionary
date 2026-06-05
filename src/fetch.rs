use std::{io::{Cursor, Read}, sync::OnceLock};
use ruzstd::decoding::StreamingDecoder;

const COMPRESSED: &[u8; 7376427] = include_bytes!("../static/data.bin.zst");
static DATA: OnceLock<Vec<u8>> = OnceLock::new();


use wasm_bindgen_futures::JsFuture;
use web_sys::{Headers, Request, RequestInit, Response, wasm_bindgen::JsCast};

pub async fn fetch_bytes(url: &str, range: Option<(u64, u64)>) -> Result<Vec<u8>, String> {
    let opts = RequestInit::new();
    opts.set_method("GET");

    if let Some((start, end)) = range {
        let headers = Headers::new().map_err(|e| format!("{e:?}"))?;
        headers
            .set("Range", &format!("bytes={}-{}", start, end - 1))
            .map_err(|e| format!("{e:?}"))?;
        opts.set_headers(&headers);
    }

    let request = Request::new_with_str_and_init(url, &opts)
        .map_err(|e| format!("{e:?}"))?;

    let window = web_sys::window().ok_or("no window")?;
    let resp_val = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| format!("{e:?}"))?;

    let response: Response = resp_val.dyn_into().map_err(|e| format!("{e:?}"))?;

    if !response.ok() {
        return Err(format!("HTTP {}", response.status()));
    }

    let buf = JsFuture::from(
        response.array_buffer().map_err(|e| format!("{e:?}"))?
    )
    .await
    .map_err(|e| format!("{e:?}"))?;

    Ok(js_sys::Uint8Array::new(&buf).to_vec())
}

pub async fn init_data() -> Result<(), String> {
    if DATA.get().is_some() {
        return Ok(());
    }

    let raw = Cursor::new(COMPRESSED);

    let mut decoder = StreamingDecoder::new(raw).map_err(|e| e.to_string())?;

    let mut decompressed = Vec::new();

    decoder
        .read_to_end(&mut decompressed)
        .map_err(|e| e.to_string())?;

    let _ = DATA.set(decompressed);

    Ok(())
}

pub fn read_bytes(range: Option<(u64, u64)>) -> Result<&'static [u8], String> {
    let data = DATA.get().ok_or("not initialized")?;

    match range {
        None => Ok(data.as_slice()),
        Some((start,end)) => {
            let s = start as usize;
            let e = end as usize;

            if e > data.len() {
                return Err("out of bounds".into());
            }

            Ok(&data[s..e])
        }
    }
}