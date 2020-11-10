use image::GenericImageView;
use solstice::image::MappedImage;
use solstice::texture::Texture;
use std::num::NonZeroU32;
use wfc::orientation::OrientationTable;
use wfc::overlapping::{OverlappingPatterns, Pattern};
use wfc::retry as wfc_retry;
use wfc::*;

pub mod retry {
    pub use super::wfc_retry::RetryOwn as Retry;
    pub use super::wfc_retry::{Forever, NumTimes};

    pub trait ImageRetry: Retry {
        type ImageReturn;
        #[doc(hidden)]
        fn image_return(
            r: Self::Return,
            image_patterns: &super::ImagePatterns,
            ctx: &mut solstice::Context,
        ) -> Self::ImageReturn;
    }
}

struct Forbid {
    bottom_left_corner_id: PatternId,
    flower_id: PatternId,
    sprout_coord: Coord,
    sprout_id: PatternId,
}

impl ForbidPattern for Forbid {
    fn forbid<W: Wrap, R: rand::Rng>(&mut self, fi: &mut ForbidInterface<W>, rng: &mut R) {
        fi.forbid_all_patterns_except(self.sprout_coord, self.sprout_id, rng)
            .unwrap();
        let output_size = fi.wave_size();
        for i in 0..(output_size.width() as i32) {
            let coord = Coord::new(i, output_size.height() as i32 - 1);
            fi.forbid_all_patterns_except(coord, self.bottom_left_corner_id, rng)
                .unwrap();
        }
        for i in 0..8 {
            for j in 0..(output_size.width() as i32) {
                let coord = Coord::new(j, output_size.height() as i32 - 1 - i);
                fi.forbid_pattern(coord, self.flower_id, rng).unwrap();
            }
        }
    }
}

pub struct ImagePatterns {
    overlapping_patterns: OverlappingPatterns<[u8; 4]>,
    empty_colour: [u8; 4],
}

impl ImagePatterns {
    pub fn new(
        image: &MappedImage,
        pattern_size: NonZeroU32,
        orientations: &[Orientation],
    ) -> Self {
        let width = image.inner().get_texture_info().width() as u32;
        let size = Size::new(width, image.inner().get_texture_info().height() as u32);
        let grid = grid_2d::Grid::new_fn(size, |Coord { x, y }| {
            let pixel = image.get_pixel(x as usize, y as usize);
            [pixel[0], pixel[1], pixel[2], 255]
        });
        let overlapping_patterns = OverlappingPatterns::new(grid, pattern_size, orientations);
        Self {
            overlapping_patterns,
            empty_colour: [0, 0, 0, 0],
        }
    }

    pub fn set_empty_colour(&mut self, empty_colour: [u8; 4]) {
        self.empty_colour = empty_colour;
    }

    pub fn image_from_wave(&self, wave: &Wave, ctx: &mut solstice::Context) -> MappedImage {
        let size = wave.grid().size();
        let mut image = MappedImage::with_data(
            ctx,
            solstice::texture::TextureType::Tex2D,
            solstice::PixelFormat::RGB8,
            size.width(),
            size.height(),
            vec![0; (size.width() * size.height() * 3) as usize],
            solstice::image::Settings {
                filter: solstice::texture::FilterMode::Nearest,
                ..solstice::image::Settings::default()
            },
        )
        .unwrap();
        let pixel_stride = image.pixel_stride();
        wave.grid().enumerate().for_each(|(Coord { x, y }, cell)| {
            let color = match cell.chosen_pattern_id() {
                Ok(pattern_id) => *self.overlapping_patterns.pattern_top_left_value(pattern_id),
                Err(_) => self.empty_colour,
            };
            image.set_pixel(x as usize, y as usize, &color[..pixel_stride]);
        });
        image
    }

    pub fn weighted_average_colour<'a>(&self, cell: &'a WaveCellRef<'a>) -> [u8; 4] {
        use wfc::EnumerateCompatiblePatternWeights::*;
        match cell.enumerate_compatible_pattern_weights() {
            MultipleCompatiblePatternsWithoutWeights | NoCompatiblePattern => self.empty_colour,
            SingleCompatiblePatternWithoutWeight(pattern_id) => {
                *self.overlapping_patterns.pattern_top_left_value(pattern_id)
            }
            CompatiblePatternsWithWeights(iter) => {
                let (r, g, b, a) = iter
                    .map(|(pattern_id, weight)| {
                        let &[r, g, b, a] =
                            self.overlapping_patterns.pattern_top_left_value(pattern_id);
                        [
                            r as u32 * weight,
                            g as u32 * weight,
                            b as u32 * weight,
                            a as u32 * weight,
                        ]
                    })
                    .fold(
                        (0, 0, 0, 0),
                        |(acc_r, acc_g, acc_b, acc_a), [r, g, b, a]| {
                            (acc_r + r, acc_g + g, acc_b + b, acc_a + a)
                        },
                    );
                let total_weight = cell.sum_compatible_pattern_weight();
                [
                    (r / total_weight) as u8,
                    (g / total_weight) as u8,
                    (b / total_weight) as u8,
                    (a / total_weight) as u8,
                ]
            }
        }
    }

    pub fn grid(&self) -> &grid_2d::Grid<[u8; 4]> {
        self.overlapping_patterns.grid()
    }

    pub fn id_grid(&self) -> grid_2d::Grid<OrientationTable<PatternId>> {
        self.overlapping_patterns.id_grid()
    }

    pub fn id_grid_original_orientation(&self) -> grid_2d::Grid<PatternId> {
        self.overlapping_patterns.id_grid_original_orientation()
    }

    pub fn pattern(&self, pattern_id: PatternId) -> &Pattern {
        self.overlapping_patterns.pattern(pattern_id)
    }

    pub fn pattern_mut(&mut self, pattern_id: PatternId) -> &mut Pattern {
        self.overlapping_patterns.pattern_mut(pattern_id)
    }

    pub fn global_stats(&self) -> GlobalStats {
        self.overlapping_patterns.global_stats()
    }

    pub fn collapse_wave_retrying<W, F, RT, R>(
        &self,
        output_size: Size,
        wrap: W,
        forbid: F,
        retry: RT,
        rng: &mut R,
    ) -> RT::Return
    where
        W: Wrap,
        F: ForbidPattern + Send + Sync + Clone,
        RT: retry::Retry,
        R: rand::Rng + Send + Sync + Clone,
    {
        let global_stats = self.global_stats();
        let run = RunOwn::new_wrap_forbid(output_size, &global_stats, wrap, forbid, rng);
        run.collapse_retrying(retry, rng)
    }
}

impl retry::ImageRetry for retry::Forever {
    type ImageReturn = MappedImage;
    fn image_return(
        r: Self::Return,
        image_patterns: &ImagePatterns,
        ctx: &mut solstice::Context,
    ) -> Self::ImageReturn {
        image_patterns.image_from_wave(&r, ctx)
    }
}

impl retry::ImageRetry for retry::NumTimes {
    type ImageReturn = Result<MappedImage, PropagateError>;
    fn image_return(
        r: Self::Return,
        image_patterns: &ImagePatterns,
        ctx: &mut solstice::Context,
    ) -> Self::ImageReturn {
        match r {
            Ok(r) => Ok(image_patterns.image_from_wave(&r, ctx)),
            Err(e) => Err(e),
        }
    }
}

fn main() {
    use glutin::{
        event::*,
        event_loop::{ControlFlow, EventLoop},
        window::WindowBuilder,
    };

    let event_loop = EventLoop::new();
    let (width, height) = (1280, 720);
    let wb = WindowBuilder::new().with_inner_size(glutin::dpi::PhysicalSize::new(width, height));
    let window = glutin::ContextBuilder::new()
        .with_multisampling(16)
        .with_double_buffer(Some(true))
        .with_vsync(true)
        .build_windowed(wb, &event_loop)
        .unwrap();
    let window = unsafe { window.make_current().unwrap() };
    let glow_ctx = unsafe {
        solstice::glow::Context::from_loader_function(|name| window.get_proc_address(name))
    };
    let mut context = solstice::Context::new(glow_ctx);
    let mut d2 = solstice_2d::Graphics2D::new(&mut context, width as _, height as _).unwrap();

    let image_settings = solstice::image::Settings {
        mipmaps: false,
        filter: solstice::texture::FilterMode::Nearest,
        ..solstice::image::Settings::default()
    };
    let output_size = Size::new(64, 64);
    let pattern_size = NonZeroU32::new(3).unwrap();

    let input_image = {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("examples")
            .join("resources")
            .join("flowers.png");
        image::open(path).unwrap()
    };

    let proto = {
        let buf = input_image.as_rgb8().unwrap().to_vec();
        solstice::image::MappedImage::with_data(
            &mut context,
            solstice::texture::TextureType::Tex2D,
            solstice::PixelFormat::RGB8,
            input_image.width(),
            input_image.height(),
            buf,
            image_settings,
        )
        .unwrap()
    };

    let image = {
        use rand::SeedableRng;
        let image_patterns = ImagePatterns::new(&proto, pattern_size, &[Orientation::Original]);
        let wave = image_patterns.collapse_wave_retrying(
            output_size,
            wfc::wrap::WrapXY,
            ForbidNothing,
            retry::NumTimes(10),
            &mut rand::rngs::StdRng::from_entropy(),
        );
        let mut image = <retry::NumTimes as retry::ImageRetry>::image_return(
            wave,
            &image_patterns,
            &mut context,
        )
        .unwrap();
        image.unmap(&mut context);
        image
    };

    event_loop.run(move |event, _, cf| match event {
        Event::WindowEvent { window_id, event } => {
            if window_id == window.window().id() {
                match event {
                    WindowEvent::CloseRequested => *cf = ControlFlow::Exit,
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => *cf = ControlFlow::Exit,
                    _ => {}
                }
            }
        }
        Event::RedrawRequested(window_id) => {
            if window_id == window.window().id() {
                let mut g = d2.start(&mut context);
                g.clear([1., 0., 0., 1.]);
                let rectangle = solstice_2d::Rectangle {
                    x: 0.0,
                    y: 0.0,
                    width: 400.0,
                    height: 400.0,
                };
                g.image(rectangle, image.inner());

                let rectangle = solstice_2d::Rectangle {
                    x: 400.0,
                    y: 0.0,
                    width: 400.0,
                    height: 400.0,
                };
                g.image(rectangle, proto.inner());
                drop(g);

                window.swap_buffers().unwrap();
            }
        }
        Event::MainEventsCleared => {
            window.window().request_redraw();
        }
        _ => {}
    })
}
