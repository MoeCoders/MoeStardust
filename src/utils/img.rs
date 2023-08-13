pub mod color {
    use std::io;

    use hyper::{Body, Client, Request, Uri};
    use hyper_tls::HttpsConnector;
    use image::GenericImageView;
    use image::{imageops::FilterType, io::Reader as ImageReader, DynamicImage};
    use palette::LinSrgb;

    macro_rules! fix_url {
        ($url:expr) => {{
            let url = $url;
            if url.starts_with("http://") || url.starts_with("https://") {
                url.to_string()
            } else {
                format!("http://{}", url)
            }
        }};
    }
    async fn download_image_and_parse(
        url: &str,
    ) -> Result<DynamicImage, Box<dyn std::error::Error + Send + Sync>> {
        // 将URL解析为Uri类型
        let url = fix_url!(url);
        println!("{}", url);
        let uri = url.parse::<Uri>()?;

        // 创建一个新的hyper客户端
        let https = HttpsConnector::new();
        let client = Client::builder().build::<_, hyper::Body>(https);

        // 创建一个新的请求
        let request = Request::builder()
            .uri(uri)
            .header("User-Agent", "Mozilla/5.0")
            .body(Body::empty())?;

        // 发送请求并等待响应
        let response = client.request(request).await?;
        if response.status() == 404 {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::NotFound,
                "图片不存在",
            )));
        }
        // 从响应中提取字节流
        let bytes = hyper::body::to_bytes(response.into_body()).await?;

        // 使用image库解析字节流中的图像
        let img = ImageReader::new(std::io::Cursor::new(bytes))
            .with_guessed_format()?
            .decode()?;

        Ok(img)
    }
    pub async fn get_theme_color(
        url: &str,
        resize: bool,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let mut img = download_image_and_parse(url).await?;
        if resize {
            img = img.resize(50, (img.height() * 50) / img.width(), FilterType::Lanczos3);
        }
        // Get the image dimensions
        let (width, height) = img.dimensions();
        // Calculate the average color of the image
        let mut sum_red: u32 = 0;
        let mut sum_green: u32 = 0;
        let mut sum_blue: u32 = 0;

        for x in 0..width {
            for y in 0..height {
                let pixel = img.get_pixel(x, y);
                sum_red += pixel[0] as u32;
                sum_green += pixel[1] as u32;
                sum_blue += pixel[2] as u32;
            }
        }

        let pixel_count = (width * height) as f32;
        let avg_red = (sum_red as f32 / pixel_count).round() as u8;
        let avg_green = (sum_green as f32 / pixel_count).round() as u8;
        let avg_blue = (sum_blue as f32 / pixel_count).round() as u8;

        // Create a palette color from the average color
        let avg_color = LinSrgb::new(
            avg_red as f32 / 255.0,
            avg_green as f32 / 255.0,
            avg_blue as f32 / 255.0,
        );

        // Convert the color to hexadecimal format
        Ok(format!("#{:X}", avg_color.into_format::<u8>()))
    }
}
