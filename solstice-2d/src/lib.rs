mod d2;
pub use d2::*;

fn create_default_texture(gl: &mut solstice::Context) -> solstice::image::Image {
    use solstice::image::*;
    use solstice::texture::*;
    use solstice::PixelFormat;
    let image = Image::new(
        gl,
        TextureType::Tex2D,
        PixelFormat::RGBA8,
        1,
        1,
        Settings {
            mipmaps: false,
            filter: FilterMode::Nearest,
            wrap: WrapMode::Clamp,
            ..Settings::default()
        },
    )
    .unwrap();
    gl.set_texture_data(
        image.get_texture_key(),
        image.get_texture_info(),
        image.get_texture_type(),
        Some(&[255, 255, 255, 255]),
    );
    image
}
