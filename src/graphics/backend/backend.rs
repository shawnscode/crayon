use std::str;

use gl;
use gl::types::*;

use super::*;
use super::super::pipeline::*;
use super::super::resource::*;

pub struct GLRenderState {
    viewport: ((u16, u16), (u16, u16)),
    cull_face: CullFace,
    front_face_order: FrontFaceOrder,
    depth_test: Comparison,
    depth_mask: bool,
    color_blend: Option<(Equation, BlendFactor, BlendFactor)>,
    color_mask: (bool, bool, bool, bool),
}

impl RenderStateVisitor for GLRenderState {
    /// Set the viewport relative to the top-lef corner of th window, in pixels.
    unsafe fn set_viewport(&mut self, position: (u16, u16), size: (u16, u16)) -> Result<()> {
        if self.viewport.0 != position || self.viewport.1 != size {
            gl::Viewport(position.0 as i32,
                         position.1 as i32,
                         size.0 as i32,
                         size.1 as i32);
            self.viewport = (position, size);
        }

        check()
    }

    /// Specify whether front- or back-facing polygons can be culled.
    unsafe fn set_face_cull(&mut self, face: CullFace) -> Result<()> {
        if self.cull_face != face {
            if face != CullFace::Nothing {
                gl::Enable(gl::CULL_FACE);
                gl::CullFace(match face {
                    CullFace::Front => gl::FRONT,
                    CullFace::Back => gl::BACK,
                    CullFace::Nothing => unreachable!(""),
                });
            } else {
                gl::Disable(gl::CULL_FACE);
            }

            self.cull_face = face;
        }

        check()
    }

    /// Define front- and back-facing polygons.
    unsafe fn set_front_face(&mut self, front: FrontFaceOrder) -> Result<()> {
        if self.front_face_order != front {
            gl::FrontFace(match front {
                FrontFaceOrder::Clockwise => gl::CW,
                FrontFaceOrder::CounterClockwise => gl::CCW,
            });
            self.front_face_order = front;
        }

        check()
    }

    /// Specify the value used for depth buffer comparisons.
    unsafe fn set_depth_test(&mut self, comparsion: Comparison) -> Result<()> {
        if self.depth_test != comparsion {
            if comparsion != Comparison::Always {
                gl::Enable(gl::DEPTH_TEST);
                gl::DepthFunc(comparsion.into());
            } else {
                gl::Disable(gl::DEPTH_TEST);
            }

            self.depth_test = comparsion;
        }

        check()
    }

    /// Enable or disable writing into the depth buffer.
    ///
    /// Optional `offset` to address the scale and units used to calculate depth values.
    unsafe fn set_depth_write(&mut self, enable: bool, offset: Option<(f32, f32)>) -> Result<()> {
        if self.depth_mask != enable {
            if enable {
                gl::DepthMask(gl::TRUE);
            } else {
                gl::DepthMask(gl::FALSE);
            }
            self.depth_mask = enable;
        }

        if enable {
            if let Some(v) = offset {
                if v.0 != 0.0 || v.1 != 0.0 {
                    gl::Enable(gl::POLYGON_OFFSET_FILL);
                    gl::PolygonOffset(v.0, v.1);
                } else {
                    gl::Disable(gl::POLYGON_OFFSET_FILL);
                }
            }
        }

        check()
    }

    // Specifies how source and destination are combined.
    unsafe fn set_color_blend(&mut self,
                              blend: Option<(Equation, BlendFactor, BlendFactor)>)
                              -> Result<()> {
        if let Some((equation, src, dst)) = blend {
            if self.color_blend == None {
                gl::Enable(gl::BLEND);
            }

            if self.color_blend != blend {
                gl::BlendFunc(src.into(), dst.into());
                gl::BlendEquation(equation.into());
            }

        } else {
            if self.color_blend != None {
                gl::Disable(gl::BLEND);
            }
        }

        self.color_blend = blend;

        check()
    }

    /// Enable or disable writing color elements into the color buffer.
    unsafe fn set_color_write(&mut self,
                              red: bool,
                              green: bool,
                              blue: bool,
                              alpha: bool)
                              -> Result<()> {
        if self.color_mask.0 != red || self.color_mask.1 != green || self.color_mask.2 != blue ||
           self.color_mask.3 != alpha {

            self.color_mask = (red, green, blue, alpha);
            gl::ColorMask(red as u8, green as u8, blue as u8, alpha as u8);
        }

        check()
    }
}

unsafe fn check() -> Result<()> {
    match gl::GetError() {
        gl::NO_ERROR => Ok(()),
        gl::INVALID_ENUM => Err(ErrorKind::InvalidEnum.into()),
        gl::INVALID_VALUE => Err(ErrorKind::InvalidValue.into()),
        gl::INVALID_OPERATION => Err(ErrorKind::InvalidOperation.into()),
        gl::INVALID_FRAMEBUFFER_OPERATION => Err(ErrorKind::InvalidFramebufferOperation.into()),
        gl::OUT_OF_MEMORY => Err(ErrorKind::OutOfBounds.into()),
        _ => Err(ErrorKind::Unknown.into()),
    }
}

impl From<ResourceHint> for GLenum {
    fn from(hint: ResourceHint) -> Self {
        match hint {
            ResourceHint::Static => gl::STATIC_DRAW,
            ResourceHint::Dynamic => gl::DYNAMIC_DRAW,
        }
    }
}

impl From<Resource> for GLuint {
    fn from(res: Resource) -> GLuint {
        match res {
            Resource::Vertex => gl::ARRAY_BUFFER,
            Resource::Index => gl::ELEMENT_ARRAY_BUFFER,
        }
    }
}


impl From<Comparison> for GLenum {
    fn from(cmp: Comparison) -> GLenum {
        match cmp {
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

impl From<Equation> for GLenum {
    fn from(eq: Equation) -> GLenum {
        match eq {
            Equation::Add => gl::FUNC_ADD,
            Equation::Subtract => gl::FUNC_SUBTRACT,
            Equation::ReverseSubtract => gl::FUNC_REVERSE_SUBTRACT,
        }
    }
}

impl From<BlendFactor> for GLenum {
    fn from(factor: BlendFactor) -> GLenum {
        match factor {
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