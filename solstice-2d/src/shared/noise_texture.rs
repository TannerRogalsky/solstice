// ported from https://github.com/blackears/PerlinNoiseMaker/blob/master/noiseMaker.js

struct Random {
    seed: i32,
    m: i32,
    a: i32,
    q: i32,
    r: i32,
}

impl Random {
    fn with_seed(seed: i32) -> Self {
        let seed = Self::validate_seed(seed);
        let m = i32::max_value();
        let a = 16807; //7^5; primitive root of m
        let q = 127773; // m / a
        let r = 2836; // m % a
        Self { seed, m, a, q, r }
    }

    fn validate_seed(seed: i32) -> i32 {
        if seed <= 0 {
            -(seed % (i32::max_value() - 1)) + 1
        } else if seed > i32::max_value() - 1 {
            i32::max_value() - 1
        } else {
            seed
        }
    }

    fn next_long(&mut self) -> i32 {
        let res = self.a * (self.seed % self.q) - self.r * (self.seed / self.q);
        let res = if res <= 0 { res + self.m } else { res };
        self.seed = res;
        res
    }

    fn next(&mut self) -> f32 {
        self.next_long() as f32 / self.m as f32
    }
}

struct PerlinSampler {
    width: usize,
    height: usize,
    gradients: Vec<f32>,
}

impl PerlinSampler {
    pub fn new(width: usize, height: usize, seed: i32) -> Self {
        let mut rng = Random::with_seed(seed);
        let mut gradients = Vec::with_capacity(width * height * 2);
        const TAU: f32 = std::f32::consts::PI * 2.;
        for _i in (0..(width * height * 2)).step_by(2) {
            let phi = rng.next() * TAU;
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

pub struct PerlinTextureSettings {
    pub seed: i32,
    pub width: usize,
    pub height: usize,
    pub period: u32,
    pub levels: u32,
    pub attenuation: f32,
    pub color: bool,
}

fn raster_to_bytes(raster: Vec<f32>) -> Vec<u8> {
    let mut bytes = vec![0u8; raster.len()];
    for (p, f) in bytes.iter_mut().zip(raster.into_iter()) {
        *p = (((f + 1.) / 2.) * 255.).round() as u8;
    }
    bytes
}

fn bytes(settings: PerlinTextureSettings) -> Vec<u8> {
    let PerlinTextureSettings {
        seed,
        width,
        height,
        period,
        levels,
        attenuation,
        color,
    } = settings;
    let num_channels = if color { 3 } else { 1 };
    let mut raster = vec![0f32; width * height * num_channels];
    for channel in 0..num_channels {
        let mut local_period_inv = 1. / period as f32;
        let mut freq_inv = 1f32;
        let mut atten = 1.;
        let mut weight = 0f32;

        for level in 0..levels {
            let sampler = PerlinSampler::new(
                (width as f32 * local_period_inv).ceil() as usize,
                (height as f32 * local_period_inv).ceil() as usize,
                seed * 100 + channel as i32 * 10 + level as i32,
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

    raster_to_bytes(raster)
}

pub fn create_perlin_texture(
    gl: &mut solstice::Context,
    settings: PerlinTextureSettings,
) -> crate::ImageResult {
    let width = settings.width;
    let height = settings.height;
    let format = if settings.color {
        solstice::PixelFormat::RGB8
    } else {
        solstice::PixelFormat::LUMINANCE
    };
    let bytes = bytes(settings);

    use solstice::image::*;
    use solstice::texture::*;
    Image::with_data(
        gl,
        TextureType::Tex2D,
        format,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn random_test() {
        let mut rng = Random::with_seed(1);
        assert_eq!(rng.next_long(), 16807);
        assert_eq!(rng.next_long(), 282475249);
        assert_eq!(rng.next_long(), 1622650073);

        assert_eq!(rng.next(), 0.4586501319234493);
        assert_eq!(rng.next(), 0.5327672374121692);
        assert_eq!(rng.next(), 0.21895918632809036);
    }

    #[test]
    fn sampler_test() {
        let sampler = PerlinSampler::new(2, 2, 0);

        assert_eq!(CONTROL_GRADIENTS_4X4.len(), sampler.gradients.len());
        for (a, b) in CONTROL_GRADIENTS_4X4.iter().zip(sampler.gradients.iter()) {
            // epsilon doesn't work. the sin/cos operations are too variable i think
            assert!((a - b).abs() <= 0.00001, "{} != {}", a, b);
        }

        assert_eq!(sampler.get(0., 0.), 0.);
        assert_eq!(sampler.get(0.5, 0.0), -0.18387488976327257);
        assert_eq!(sampler.get(1., 0.), 0.);
        assert_eq!(sampler.get(1.5, 0.0), 0.18387488976327257);

        assert!((sampler.get(0., 0.5) - 0.24119700031647284).abs() <= std::f32::EPSILON);
        assert!((sampler.get(1.5, 1.0) - 0.31406892987757684).abs() <= std::f32::EPSILON);
    }

    #[test]
    fn black_and_white_raster_test() {
        fn dup_channel(bytes: Vec<u8>) -> Vec<u8> {
            let mut new_bytes = Vec::with_capacity(bytes.len() * 4);
            for byte in bytes {
                new_bytes.push(byte);
                new_bytes.push(byte);
                new_bytes.push(byte);
                new_bytes.push(255);
            }
            new_bytes
        }

        let settings = PerlinTextureSettings {
            seed: 0,
            width: 4,
            height: 4,
            period: 2,
            levels: 1,
            attenuation: 0.0,
            color: false,
        };
        let bytes = dup_channel(bytes(settings));
        assert_eq!(&SEED0_CELL2_LEVEL1_4X4_BW[..], bytes.as_slice());
    }

    #[test]
    fn color_raster_test() {
        fn add_alpha(bytes: Vec<u8>) -> Vec<u8> {
            let mut new_bytes = Vec::with_capacity(bytes.len() / 3);
            for byte in bytes.chunks_exact(3) {
                new_bytes.extend_from_slice(byte);
                new_bytes.push(255);
            }
            new_bytes
        }

        let settings = PerlinTextureSettings {
            seed: 0,
            width: 4,
            height: 4,
            period: 2,
            levels: 1,
            attenuation: 0.0,
            color: true,
        };
        let bytes = add_alpha(bytes(settings));
        assert_eq!(&SEED0_CELL2_LEVEL1_4X4_COLOR[..], bytes.as_slice());
    }

    const CONTROL_GRADIENTS_4X4: [f32; 8] = [
        0.00004917452831956654,
        0.9999999987909329,
        0.7355487335814098,
        0.6774718153006694,
        -0.9993798653316448,
        0.03521199752504148,
        0.25689585417866245,
        -0.9664390928071026,
    ];

    const SEED0_CELL2_LEVEL1_4X4_BW: [u8; 64] = [
        128, 128, 128, 255, 104, 104, 104, 255, 128, 128, 128, 255, 151, 151, 151, 255, 158, 158,
        158, 255, 137, 137, 137, 255, 180, 180, 180, 255, 201, 201, 201, 255, 128, 128, 128, 255,
        87, 87, 87, 255, 128, 128, 128, 255, 168, 168, 168, 255, 97, 97, 97, 255, 54, 54, 54, 255,
        75, 75, 75, 255, 118, 118, 118, 255,
    ];

    const SEED0_CELL2_LEVEL1_4X4_COLOR: [u8; 64] = [
        128, 128, 128, 255, 104, 98, 151, 255, 128, 128, 128, 255, 151, 157, 104, 255, 158, 189,
        135, 255, 137, 154, 121, 255, 180, 142, 91, 255, 201, 178, 105, 255, 128, 128, 128, 255,
        87, 133, 120, 255, 128, 128, 128, 255, 168, 122, 135, 255, 97, 66, 120, 255, 54, 77, 150,
        255, 75, 113, 164, 255, 118, 101, 134, 255,
    ];
}
