use std::{io, path::Path};

pub async fn load_file(path: impl AsRef<Path>) -> io::Result<Vec<u8>> {
  let path = path.as_ref();

  #[cfg(target_arch = "wasm32")]
  {
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;

    // Load files via window::fetch
    let window = web_sys::window().unwrap();
    let resp_value = JsFuture::from(window.fetch_with_str(path.to_str().unwrap()))
      .await
      .unwrap();
    let resp: web_sys::Response = resp_value.dyn_into().unwrap();
    let data = JsFuture::from(resp.array_buffer().unwrap()).await.unwrap();
    let bytes = js_sys::Uint8Array::new(&data).to_vec();
    return Ok(bytes);
  };

  #[cfg(not(target_arch = "wasm32"))]
  {
    use tokio::{fs::File, io::AsyncReadExt};

    // Load files via standard i/o methods, but using Tokio to become async
    let mut file = File::open(path).await?;
    let mut contents = vec![];
    file.read_to_end(&mut contents).await?;
    return Ok(contents);
  };
}

pub async fn load_string(path: impl AsRef<Path>) -> anyhow::Result<String> {
  let bytes = load_file(path).await?;
  Ok(String::from_utf8(bytes)?)
}
