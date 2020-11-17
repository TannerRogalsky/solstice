mod d2;
pub mod d3;

pub use d2::*;
pub use solstice;

type ImageResult = Result<solstice::image::Image, solstice::GraphicsError>;

pub fn create_default_texture(gl: &mut solstice::Context) -> ImageResult {
    use solstice::image::*;
    use solstice::texture::*;
    Image::with_data(
        gl,
        TextureType::Tex2D,
        solstice::PixelFormat::RGBA8,
        1,
        1,
        &[255, 255, 255, 255],
        Settings {
            mipmaps: false,
            filter: FilterMode::Nearest,
            wrap: WrapMode::Clamp,
            ..Settings::default()
        },
    )
}

struct PerlinSampler {
    width: usize,
    height: usize,
    gradients: Vec<f32>,
}

impl PerlinSampler {
    pub fn new<R: rand::Rng>(width: usize, height: usize, mut rng: R) -> Self {
        let mut gradients = Vec::with_capacity(width * height * 2);
        const TAU: f32 = std::f32::consts::PI * 2.;
        for _i in (0..(width * height * 2)).step_by(2) {
            let phi = rng.gen::<f32>() * TAU;
            let (x, y) = phi.sin_cos();
            gradients.push(x);
            gradients.push(y);
        }
        Self {
            width,
            height,
            gradients,
        }
    }

    pub fn dot(&self, x_cell: usize, y_cell: usize, vx: f32, vy: f32) -> f32 {
        let offset = (x_cell + y_cell * self.width) * 2;
        let wx = self.gradients[offset];
        let wy = self.gradients[offset + 1];
        wx * vx + wy * vy
    }

    pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
        a + t * (b - a)
    }

    pub fn s_curve(t: f32) -> f32 {
        t * t * (3. - 2. * t)
    }

    pub fn get(&self, x: f32, y: f32) -> f32 {
        let x_cell = x.trunc() as usize;
        let y_cell = y.trunc() as usize;
        let x_fract = x.fract();
        let y_fract = y.fract();

        let x0 = x_cell;
        let y0 = y_cell;
        let x1 = if x_cell == (self.width - 1) {
            0
        } else {
            x_cell + 1
        };
        let y1 = if y_cell == (self.height - 1) {
            0
        } else {
            y_cell + 1
        };

        let v00 = self.dot(x0, y0, x_fract, y_fract);
        let v10 = self.dot(x1, y0, x_fract - 1., y_fract);
        let v01 = self.dot(x0, y1, x_fract, y_fract - 1.);
        let v11 = self.dot(x1, y1, x_fract - 1., y_fract - 1.);

        let vx0 = Self::lerp(v00, v10, Self::s_curve(x_fract));
        let vx1 = Self::lerp(v01, v11, Self::s_curve(x_fract));

        return Self::lerp(vx0, vx1, Self::s_curve(y_fract));
    }
}

// spec.randseed = document.getElementById("randseed").value;
// spec.period = document.getElementById("period").value;
// spec.levels = document.getElementById("numLevels").value;
// spec.atten = document.getElementById("atten").value;
// spec.absolute = document.getElementById("absolute").checked;
// spec.color = document.getElementById("noiseColor").checked;
// spec.alpha = document.getElementById("noiseAlpha").checked;
pub struct PerlinTextureSettings<R> {
    pub rng: R,
    pub width: usize,
    pub height: usize,
    pub period: u32,
    pub levels: u32,
    pub attenuation: f32,
}

pub fn create_perlin_texture<R: rand::Rng>(
    gl: &mut solstice::Context,
    settings: PerlinTextureSettings<R>,
) -> ImageResult {
    let PerlinTextureSettings {
        mut rng,
        width,
        height,
        period,
        levels,
        attenuation,
    } = settings;
    let num_channels = 3;
    let mut raster = vec![0f32; width * height * num_channels];
    for channel in 0..num_channels {
        let mut local_period_inv = 1. / period as f32;
        let mut freq_inv = 1f32;
        let mut atten = 1.;
        let mut weight = 0f32;

        for _level in 0..levels {
            let sampler = PerlinSampler::new(
                (width as f32 * local_period_inv).ceil() as usize,
                (height as f32 * local_period_inv).ceil() as usize,
                &mut rng,
            );
            for y in 0..height {
                for x in 0..width {
                    let val = sampler.get(x as f32 * local_period_inv, y as f32 * local_period_inv);
                    raster[(x + y * width) * num_channels + channel] += val * freq_inv.powf(atten);
                }
            }
            weight += freq_inv.powf(atten);
            freq_inv *= 0.5;
            local_period_inv *= 2.;
            atten *= attenuation;
        }

        let weight_inv = 1. / weight;
        for y in 0..height {
            for x in 0..width {
                raster[(x + y * width) * num_channels + channel] *= weight_inv;
            }
        }
    }

    let mut bytes = vec![0u8; width * height * num_channels];
    for (p, f) in bytes.iter_mut().zip(raster.into_iter()) {
        *p = (((f + 1.) / 2.) * 255.) as u8;
    }

    use solstice::image::*;
    use solstice::texture::*;
    Image::with_data(
        gl,
        TextureType::Tex2D,
        solstice::PixelFormat::RGB8,
        width as u32,
        height as u32,
        bytes.as_slice(),
        Settings {
            mipmaps: false,
            filter: FilterMode::Linear,
            wrap: WrapMode::Repeat,
            ..Settings::default()
        },
    )
}
