extern crate graphics;
use std::cell::RefCell;
use std::rc::Rc;

mod d2;
pub use d2::*;

fn create_default_texture(gl: &mut graphics::Context) -> graphics::image::Image {
    use data::PixelFormat;
    use graphics::image::*;
    use graphics::texture::*;
    let image = Image::new(
        gl,
        TextureType::Tex2D,
        PixelFormat::RGBA8,
        1,
        1,
        1,
        &Settings::new(false, false, 1.),
    );
    gl.set_texture_data(
        image.get_texture_key(),
        image.get_texture_info(),
        image.get_texture_type(),
        Some(&[255, 255, 255, 255]),
    );
    image
}

struct DynamicMesh<T> {
    gfx: Rc<RefCell<graphics::Context>>,
    inner: graphics::mesh::Mesh<T>,
}

impl<T> DynamicMesh<T>
where
    T: graphics::vertex::Vertex,
{
    fn new(mut gfx: Rc<RefCell<graphics::Context>>, initial_size: usize) -> Self {
        let inner = graphics::mesh::Mesh::new(&mut gfx.borrow_mut(), initial_size);
        Self { gfx, inner }
    }

    fn set_vertices(&mut self, vertices: &[T]) {
        self.set_vertices_at_offset(vertices, 0)
    }

    fn set_vertices_at_offset(&mut self, vertices: &[T], offset: usize) {
        let current_vertices = self.inner.get_vertices();
        if current_vertices.len() < vertices.len() + offset {
            let mut new_inner = graphics::mesh::Mesh::new(
                &mut self.gfx.borrow_mut(),
                (vertices.len() + offset) * 2,
            );
            new_inner.set_vertices(current_vertices, 0);
            new_inner.set_indices(self.inner.get_indices(), 0);
            self.inner = new_inner;
        }
        self.inner.set_vertices(vertices, offset);
    }

    fn set_indices(&mut self, indices: &[u32]) {
        self.set_indices_at_offset(indices, 0)
    }

    fn set_indices_at_offset(&mut self, indices: &[u32], offset: usize) {
        let current_indices = self.inner.get_indices();
        if current_indices.len() < indices.len() + offset {
            let mut new_inner =
                graphics::mesh::Mesh::new(&mut self.gfx.borrow_mut(), (indices.len() + offset) * 2);
            new_inner.set_vertices(self.inner.get_vertices(), 0);
            new_inner.set_indices(current_indices, 0);
            self.inner = new_inner;
        }
        self.inner.set_indices(indices, offset);
    }
}

impl<T> Drop for DynamicMesh<T> {
    fn drop(&mut self) {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
