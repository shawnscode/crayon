extern crate lemon3d;

use std::mem;

// Shader sources
static VS_SRC: &'static str = "#version 150\nin vec2 position;\nvoid main() {\ngl_Position = \
                               vec4(position, 0.0, 1.0);\n}";

static FS_SRC: &'static str = "#version 150\nout vec4 out_color;\nvoid main() {\nout_color = \
                               vec4(1.0, 1.0, 1.0, 1.0);\n}";

// Vertex data
static VERTEX_DATA: [f32; 6] = [0.0, 0.5, 0.5, -0.5, -0.5, -0.5];

/// A trait for plain-old-data types.
///
/// A POD type does not have invalid bit patterns and can be safely
/// created from arbitrary bit pattern.
pub unsafe trait Pod {}

macro_rules! impl_pod {
    ( ty = $($ty:ty)* ) => { $( unsafe impl Pod for $ty {} )* };
    ( ar = $($tt:expr)* ) => { $( unsafe impl<T: Pod> Pod for [T; $tt] {} )* };
}

impl_pod! { ty = isize usize i8 u8 i16 u16 i32 u32 i64 u64 f32 f64 }
impl_pod! { ar =
    0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31 32
}

unsafe impl<T: Pod, U: Pod> Pod for (T, U) {}

/// Cast a slice from one POD type to another.
pub fn cast_slice<A: Pod, B: Pod>(slice: &[A]) -> &[B] {
    use std::slice;

    let raw_len = mem::size_of::<A>().wrapping_mul(slice.len());
    let len = raw_len / mem::size_of::<B>();
    assert_eq!(raw_len, mem::size_of::<B>().wrapping_mul(len));
    unsafe { slice::from_raw_parts(slice.as_ptr() as *const B, len) }
}

fn main() {
    use lemon3d::graphics;
    use lemon3d::graphics::backend::*;
    let mut backend = device::Device::new();

    let program = lemon3d::utility::Handle::new(0, 0);
    let vb = lemon3d::utility::Handle::new(0, 0);
    let ib = lemon3d::utility::Handle::new(1, 0);

    lemon3d::Application::setup("examples/resources/configs/basic.json")
        .unwrap()
        .perform(|_| {
            unsafe {
                backend.create_program(program, VS_SRC, FS_SRC, None);
                let slice = cast_slice(&VERTEX_DATA);
                backend.create_buffer(vb,
                                      graphics::Buffer::Vertex,
                                      graphics::BufferHint::Static,
                                      slice.len(),
                                      Some(slice));
            }
        })
        .run(|_| {
            unsafe {
                backend.clear(Some([0.75, 0.75, 0.75, 1.0]), None, None);
            }
            return true;
        })
        .perform(|_| println!("hello world."));
}
