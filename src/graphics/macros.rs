#[macro_export]
macro_rules! offset_of {
    ($ty:ty, $field:ident) => {
        unsafe { &(*(0 as *const $ty)).$field as *const _ as usize }
    }
}

#[macro_export]
macro_rules! impl_vertex {
    ($name: ident { $($field: ident => [$attribute: tt; $format: tt; $size: tt; $normalized: tt],)* }) => (
        #[repr(C)]
        #[derive(Debug, Copy, Clone)]
        pub struct $name {
            $($field: impl_vertex_field!{VertexFormat::$format, $size}, )*
        }

        impl $name {
            pub fn new($($field: impl_vertex_field!{VertexFormat::$format, $size}, ) *) -> Self {
                $name {
                    $($field: $field,)*
                }
            }

            pub fn layout() -> $crate::graphics::resource::VertexLayout {
                let mut builder = $crate::graphics::resource::CustomVertexLayoutBuilder::new();

                $( builder.with(
                    $crate::graphics::resource::VertexAttribute::$attribute,
                    $crate::graphics::resource::VertexFormat::$format,
                    $size,
                    $normalized,
                    offset_of!($name, $field) as u8); ) *

                builder.finish(::std::mem::size_of::<$name>() as u8)
            }

            pub fn as_bytes(values: &[Self]) -> &[u8] {
                let len = values.len() * ::std::mem::size_of::<Self>();
                unsafe { ::std::slice::from_raw_parts(values.as_ptr() as *const u8, len) }
            }
        }
    )
}

#[macro_export]
macro_rules! impl_vertex_field {
    (VertexFormat::Byte, 2) => ([i8; 2]);
    (VertexFormat::Byte, 3) => ([i8; 3]);
    (VertexFormat::Byte, 4) => ([i8; 4]);
    (VertexFormat::UByte, 2) => ([u8; 2]);
    (VertexFormat::UByte, 3) => ([u8; 3]);
    (VertexFormat::UByte, 4) => ([u8; 4]);
    (VertexFormat::Short, 2) => ([i16; 2]);
    (VertexFormat::Short, 3) => ([i16; 3]);
    (VertexFormat::Short, 4) => ([i16; 4]);
    (VertexFormat::UShort, 2) => ([u16; 2]);
    (VertexFormat::UShort, 3) => ([u16; 3]);
    (VertexFormat::UShort, 4) => ([u16; 4]);
    (VertexFormat::Float, 2) => ([f32; 2]);
    (VertexFormat::Float, 3) => ([f32; 3]);
    (VertexFormat::Float, 4) => ([f32; 4]);
}

#[cfg(test)]
mod test {
    use super::super::resource::*;

    impl_vertex! {
        Vertex {
            position => [Position; Float; 3; false],
            texcoord => [Texcoord0; Float; 2; false],
        }
    }

    impl_vertex! {
        Vertex2 {
            position => [Position; Float; 2; false],
            color => [Color0; UByte; 4; true],
            texcoord => [Texcoord0; Byte; 2; false],
        }
    }

    fn as_bytes<T>(values: &[T]) -> &[u8]
        where T: Copy
    {
        let len = values.len() * ::std::mem::size_of::<T>();
        unsafe { ::std::slice::from_raw_parts(values.as_ptr() as *const u8, len) }
    }

    #[test]
    fn basic() {
        let layout = Vertex::layout();
        assert_eq!(layout.stride(), 20);
        assert_eq!(layout.offset(VertexAttribute::Position), Some(0));
        assert_eq!(layout.offset(VertexAttribute::Texcoord0), Some(12));
        assert_eq!(layout.offset(VertexAttribute::Normal), None);

        let bytes: [f32; 5] = [1.0, 1.0, 1.0, 0.0, 0.0];
        let bytes = as_bytes(&bytes);
        assert_eq!(bytes,
                   Vertex::as_bytes(&[Vertex::new([1.0, 1.0, 1.0], [0.0, 0.0])]));

        let bytes: [f32; 10] = [1.0, 1.0, 1.0, 0.0, 0.0, 2.0, 2.0, 2.0, 3.0, 3.0];
        let bytes = as_bytes(&bytes);
        assert_eq!(bytes,
                   Vertex::as_bytes(&[Vertex::new([1.0, 1.0, 1.0], [0.0, 0.0]),
                                      Vertex::new([2.0, 2.0, 2.0], [3.0, 3.0])]));
    }

    #[test]
    fn representation() {
        let layout = Vertex::layout();
        assert_eq!(layout.stride() as usize, ::std::mem::size_of::<Vertex>());

        let layout = Vertex2::layout();
        let _v = Vertex2::new([1.0, 1.0], [0, 0, 0, 0], [0, 0]);
        let _b = Vertex2::as_bytes(&[]);
        assert_eq!(layout.stride() as usize, ::std::mem::size_of::<Vertex2>());
    }
}