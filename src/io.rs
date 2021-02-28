use anyhow::Context;
use image::DynamicImage;
use std::{io, path::Path};

#[cfg(target_arch = "wasm32")]
fn js_error(value: wasm_bindgen::JsValue) -> anyhow::Error {
  anyhow::Error::msg(format!("{:?}", value))
}

#[cfg(target_arch = "wasm32")]
macro_rules! js_call {
  ($e:expr) => {
    $e.map_err(js_error)?
  };
}

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

// pub struct Image {
//   pub data: Vec<u8>,
//   pub width: u32,
//   pub height: u32,
// }

pub async fn load_image(path: impl AsRef<Path>) -> anyhow::Result<DynamicImage> {
  let path = path.as_ref();

  #[cfg(target_arch = "wasm32")]
  {
    use image::RgbaImage;
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;

    let image_elt = web_sys::HtmlImageElement::new().map_err(js_error)?;
    image_elt.set_src(path.to_str().context("Path::to_str")?);

    let promise = js_sys::Promise::new(&mut |resolve, reject| {
      image_elt.set_onload(Some(&resolve));
      image_elt.set_onerror(Some(&reject));
    });
    js_call!(JsFuture::from(promise).await);

    let width = image_elt.width() as u32;
    let height = image_elt.height() as u32;

    let window = web_sys::window().context("window")?;
    let document = window.document().context("document")?;

    let canvas =
      js_call!(document.create_element("canvas")).unchecked_into::<web_sys::HtmlCanvasElement>();
    canvas.set_width(width);
    canvas.set_height(height);

    let ctx = js_call!(canvas.get_context("2d"))
      .context("get_context")?
      .unchecked_into::<web_sys::CanvasRenderingContext2d>();

    js_call!(ctx.draw_image_with_html_image_element(&image_elt, 0., 0.));
    let image_data = js_call!(ctx.get_image_data(0., 0., width as f64, height as f64));
    let data = image_data.data().0;

    Ok(DynamicImage::ImageRgba8(
      RgbaImage::from_raw(width, height, data).context("Image::from_raw")?,
    ))
  }

  #[cfg(not(target_arch = "wasm32"))]
  {
    use image::io::Reader;
    use std::io::Cursor;

    let bytes = load_file(path).await?;
    let format = image::guess_format(bytes.as_slice())?;
    let mut img_reader = Reader::new(Cursor::new(bytes));
    img_reader.set_format(format);
    Ok(img_reader.decode()?)
  }
}

pub async fn load_string(path: impl AsRef<Path>) -> anyhow::Result<String> {
  let bytes = load_file(path).await?;
  Ok(String::from_utf8(bytes)?)
}
