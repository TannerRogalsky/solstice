use crate::Graphics2DLock;
use glyph_brush::ab_glyph::{point, FontArc};
use glyph_brush::{BrushAction, BrushError, HorizontalAlign, VerticalAlign};
use solstice::image::{Image, Settings};
use solstice::quad_batch::*;
use solstice::texture::*;
use solstice::Context;

pub struct Text {
    quad_batch: QuadBatch<super::Vertex2D>,
    font_texture: Image,
    glyph_brush: glyph_brush::GlyphBrush<Quad<super::Vertex2D>, glyph_brush::Extra, FontArc>,
}

impl Text {
    pub fn new(ctx: &mut Context, font: FontArc) -> Result<Self, solstice::GraphicsError> {
        let glyph_brush = glyph_brush::GlyphBrushBuilder::using_font(font).build();

        let font_texture = {
            let (width, height) = glyph_brush.texture_dimensions();
            println!("Initial texture size: {}, {}", width, height);
            Image::new(
                ctx,
                TextureType::Tex2D,
                solstice::PixelFormat::R8,
                width,
                height,
                Settings::default(),
            )?
        };
        // will only do texture sub data updates to initialize as empty texture
        ctx.set_texture_data(
            font_texture.get_texture_key(),
            font_texture.get_texture_info(),
            font_texture.get_texture_type(),
            None,
        );

        let quad_batch = QuadBatch::new(ctx, 1000)?;

        Ok(Self {
            quad_batch,
            font_texture,
            glyph_brush,
        })
    }

    pub fn set_text<S: AsRef<str>>(&mut self, text: S, ctx: &mut Graphics2DLock) {
        let text = glyph_brush::Text::new(text.as_ref())
            .with_color([1., 1., 1., 1.])
            .with_scale(128.);
        self.glyph_brush.queue(glyph_brush::Section {
            text: vec![text],
            screen_position: (ctx.inner.width / 2.0, ctx.inner.height / 2.0),
            bounds: (ctx.inner.width, ctx.inner.height),
            layout: glyph_brush::Layout::default()
                .h_align(HorizontalAlign::Center)
                .v_align(VerticalAlign::Center),
            ..glyph_brush::Section::default()
        });
        self.update(ctx);
    }

    pub fn draw(&mut self, ctx: &mut Graphics2DLock) {
        ctx.bind_texture(&self.font_texture);
        let shader = ctx.inner.default_shader.activate(ctx.ctx);
        let geometry = self.quad_batch.unmap(ctx.ctx);
        solstice::Renderer::draw(
            ctx.ctx,
            shader,
            &geometry,
            solstice::PipelineSettings {
                depth_state: None,
                ..solstice::PipelineSettings::default()
            },
        );
    }

    fn update(&mut self, ctx: &mut Graphics2DLock) {
        let Self {
            quad_batch,
            font_texture,
            glyph_brush,
        } = self;

        let to_vertex = |glyph_vertex: glyph_brush::GlyphVertex| {
            let glyph_brush::GlyphVertex {
                mut tex_coords,
                pixel_coords,
                bounds,
                extra,
            } = glyph_vertex;
            let mut gl_rect = glyph_brush::ab_glyph::Rect {
                min: point(pixel_coords.min.x as f32, pixel_coords.min.y as f32),
                max: point(pixel_coords.max.x as f32, pixel_coords.max.y as f32),
            };

            // handle overlapping bounds, modify uv_rect to preserve texture aspect
            if gl_rect.max.x > bounds.max.x {
                let old_width = gl_rect.width();
                gl_rect.max.x = bounds.max.x;
                tex_coords.max.x =
                    tex_coords.min.x + tex_coords.width() * gl_rect.width() / old_width;
            }
            if gl_rect.min.x < bounds.min.x {
                let old_width = gl_rect.width();
                gl_rect.min.x = bounds.min.x;
                tex_coords.min.x =
                    tex_coords.max.x - tex_coords.width() * gl_rect.width() / old_width;
            }
            if gl_rect.max.y > bounds.max.y {
                let old_height = gl_rect.height();
                gl_rect.max.y = bounds.max.y;
                tex_coords.max.y =
                    tex_coords.min.y + tex_coords.height() * gl_rect.height() / old_height;
            }
            if gl_rect.min.y < bounds.min.y {
                let old_height = gl_rect.height();
                gl_rect.min.y = bounds.min.y;
                tex_coords.min.y =
                    tex_coords.max.y - tex_coords.height() * gl_rect.height() / old_height;
            }

            Quad {
                vertices: [
                    super::Vertex2D {
                        position: [gl_rect.min.x as f32, gl_rect.min.y as f32],
                        uv: [tex_coords.min.x, tex_coords.min.y],
                        color: extra.color,
                    },
                    super::Vertex2D {
                        position: [gl_rect.max.x as f32, gl_rect.min.y as f32],
                        uv: [tex_coords.max.x, tex_coords.min.y],
                        color: extra.color,
                    },
                    super::Vertex2D {
                        position: [gl_rect.min.x as f32, gl_rect.max.y as f32],
                        uv: [tex_coords.min.x, tex_coords.max.y],
                        color: extra.color,
                    },
                    super::Vertex2D {
                        position: [gl_rect.max.x as f32, gl_rect.max.y as f32],
                        uv: [tex_coords.max.x, tex_coords.max.y],
                        color: extra.color,
                    },
                ],
            }
        };

        loop {
            let update_texture = |rect: glyph_brush::Rectangle<u32>, data: &[u8]| {
                let mut info = font_texture.get_texture_info();
                info.set_width(rect.width());
                info.set_height(rect.height());
                ctx.ctx.set_texture_sub_data(
                    font_texture.get_texture_key(),
                    info,
                    font_texture.get_texture_type(),
                    data,
                    rect.min[0],
                    rect.min[1],
                );
            };
            match glyph_brush.process_queued(update_texture, to_vertex) {
                Ok(action) => match action {
                    BrushAction::Draw(quads) => {
                        quad_batch.clear();
                        println!(
                            "cleared text quads in prepartion for {} new quads",
                            quads.len()
                        );
                        //                        quadbatch.push(Vertex::build_quad(Viewport::new(0., 0., 720., 480.)));
                        for quad in quads {
                            quad_batch.push(quad);
                        }
                        //                        for mut quad in quads {
                        //                            quad.vertices.iter_mut().for_each(|v| {
                        //                                v.position = [v.uv[0] * 720., v.uv[1] * 480.];
                        //                            });
                        //                            quadbatch.push(quad);
                        //                        }
                        break;
                    }
                    BrushAction::ReDraw => {
                        break;
                    }
                },
                Err(error) => match error {
                    BrushError::TextureTooSmall { suggested: (w, h) } => {
                        println!("Resize texture to {}, {}", w, h);
                        let mut info = font_texture.get_texture_info();
                        info.set_width(w);
                        info.set_height(h);
                        font_texture.set_texture_info(info);
                        ctx.ctx.set_texture_data(
                            font_texture.get_texture_key(),
                            font_texture.get_texture_info(),
                            font_texture.get_texture_type(),
                            None,
                        );
                        glyph_brush.resize_texture(w, h);
                    }
                },
            }
        }
    }
}
