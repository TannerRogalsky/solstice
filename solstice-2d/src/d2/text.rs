use glyph_brush::ab_glyph::{point, FontVec};
use glyph_brush::{BrushAction, BrushError, FontId};
use solstice::image::{Image, Settings};
use solstice::mesh::IndexedMesh;
use solstice::quad_batch::*;
use solstice::texture::*;
use solstice::Context;

pub struct Text {
    quad_batch: QuadBatch<super::Vertex2D>,
    font_texture: Image,
    glyph_brush: glyph_brush::GlyphBrush<Quad<super::Vertex2D>, glyph_brush::Extra, FontVec>,
}

pub const DEFAULT_VERT: &str = r#"
vec4 pos(mat4 transform_projection, vec4 vertex_position) {
    return transform_projection * vertex_position;
}
"#;

pub const DEFAULT_FRAG: &str = r#"
vec4 effect(vec4 color, Image texture, vec2 texture_coords, vec2 screen_coords) {
    float a = Texel(texture, texture_coords).a;
    color.a *= a;
    return color;
}
"#;

impl Text {
    pub fn new(ctx: &mut Context) -> Result<Self, crate::GraphicsError> {
        let glyph_brush = glyph_brush::GlyphBrushBuilder::using_fonts(vec![]).build();

        let font_texture = {
            let (width, height) = glyph_brush.texture_dimensions();
            Image::new(
                ctx,
                TextureType::Tex2D,
                solstice::PixelFormat::Alpha,
                width,
                height,
                Settings {
                    mipmaps: false,
                    ..Settings::default()
                },
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

    pub fn add_font(&mut self, font_data: FontVec) -> FontId {
        self.glyph_brush.add_font(font_data)
    }

    pub fn set_text(
        &mut self,
        text: glyph_brush::Text,
        bounds: super::Rectangle,
        ctx: &mut Context,
    ) {
        self.glyph_brush.queue(glyph_brush::Section {
            text: vec![text],
            screen_position: (bounds.x, bounds.y),
            bounds: (bounds.width, bounds.height),
            layout: glyph_brush::Layout::default(),
        });
        self.update(ctx);
    }

    pub fn texture(&self) -> &solstice::image::Image {
        &self.font_texture
    }

    pub fn geometry(
        &mut self,
        ctx: &mut Context,
    ) -> solstice::Geometry<&IndexedMesh<super::Vertex2D, u16>> {
        self.quad_batch.unmap(ctx)
    }

    fn update(&mut self, ctx: &mut Context) {
        let Self {
            quad_batch,
            font_texture,
            glyph_brush,
            ..
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
                ctx.set_texture_sub_data(
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
                        for quad in quads {
                            quad_batch.push(quad);
                        }
                        break;
                    }
                    BrushAction::ReDraw => {
                        break;
                    }
                },
                Err(error) => match error {
                    BrushError::TextureTooSmall { suggested: (w, h) } => {
                        let mut info = font_texture.get_texture_info();
                        info.set_width(w);
                        info.set_height(h);
                        font_texture.set_texture_info(info);
                        ctx.set_texture_data(
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
