use gl;
use gl::types::*;

use super::Error;
use super::{Buffer, BufferHint, CullFace, FrontFaceOrder, Comparison, Equation, BlendFactor,
            BlendValue, Primitive};

impl From<GLenum> for Error {
    fn from(error: GLenum) -> Self {
        match error {
            gl::INVALID_ENUM => Error::InvalidEnum,
            gl::INVALID_VALUE => Error::InvalidValue,
            gl::INVALID_OPERATION => Error::InvalidOperation,
            gl::INVALID_FRAMEBUFFER_OPERATION => Error::InvalidFramebufferOperation,
            gl::OUT_OF_MEMORY => Error::OutOfMemory,
            _ => Error::UnknownError,
        }
    }
}

impl BufferHint {
    pub fn to_native(self) -> GLenum {
        match self {
            BufferHint::Static => gl::STATIC_DRAW,
            BufferHint::Dynamic => gl::DYNAMIC_DRAW,
        }
    }
}

impl Buffer {
    pub fn to_native(self) -> GLuint {
        match self {
            Buffer::Vertex => gl::ARRAY_BUFFER,
            Buffer::Index => gl::ELEMENT_ARRAY_BUFFER,
        }
    }
}

impl Comparison {
    pub fn to_native(self) -> GLenum {
        match self {
            Comparison::Never => gl::NEVER,
            Comparison::Less => gl::LESS,
            Comparison::LessOrEqual => gl::LEQUAL,
            Comparison::Greater => gl::GREATER,
            Comparison::GreaterOrEqual => gl::GEQUAL,
            Comparison::Equal => gl::EQUAL,
            Comparison::NotEqual => gl::NOTEQUAL,
            Comparison::Always => gl::ALWAYS,
        }
    }
}

impl Equation {
    pub fn to_native(self) -> GLenum {
        match self {
            Equation::Add => gl::FUNC_ADD,
            Equation::Subtract => gl::FUNC_SUBTRACT,
            Equation::ReverseSubtract => gl::FUNC_REVERSE_SUBTRACT,
        }
    }
}

impl BlendFactor {
    pub fn to_native(self) -> GLenum {
        match self {
            BlendFactor::Zero => gl::ZERO,
            BlendFactor::One => gl::ONE,
            BlendFactor::Value(BlendValue::SourceColor) => gl::SRC_COLOR,
            BlendFactor::Value(BlendValue::SourceAlpha) => gl::SRC_ALPHA,
            BlendFactor::Value(BlendValue::DestinationColor) => gl::DST_COLOR,
            BlendFactor::Value(BlendValue::DestinationAlpha) => gl::DST_ALPHA,
            BlendFactor::OneMinusValue(BlendValue::SourceColor) => gl::ONE_MINUS_SRC_COLOR,
            BlendFactor::OneMinusValue(BlendValue::SourceAlpha) => gl::ONE_MINUS_SRC_ALPHA,
            BlendFactor::OneMinusValue(BlendValue::DestinationColor) => gl::ONE_MINUS_DST_COLOR,
            BlendFactor::OneMinusValue(BlendValue::DestinationAlpha) => gl::ONE_MINUS_DST_ALPHA,
        }
    }
}