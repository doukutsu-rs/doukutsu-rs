use std::mem;

use crate::framework::backend::VertexData;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum VertexElementFormat {
    Float1,
    Float2,
    Float3,
    Float4,
    /// 4 x u8 normalized to [0.0, 1.0]
    Color,
    Byte4,
    Short2,
    Short4,
}

impl VertexElementFormat {
    pub const fn byte_size(&self) -> u16 {
        match self {
            VertexElementFormat::Float1 => 4,
            VertexElementFormat::Float2 => 8,
            VertexElementFormat::Float3 => 12,
            VertexElementFormat::Float4 => 16,
            VertexElementFormat::Color => 4,
            VertexElementFormat::Byte4 => 4,
            VertexElementFormat::Short2 => 4,
            VertexElementFormat::Short4 => 8,
        }
    }

    pub const fn component_count(&self) -> u8 {
        match self {
            VertexElementFormat::Float1 => 1,
            VertexElementFormat::Float2 => 2,
            VertexElementFormat::Float3 => 3,
            VertexElementFormat::Float4 => 4,
            VertexElementFormat::Color => 4,
            VertexElementFormat::Byte4 => 4,
            VertexElementFormat::Short2 => 2,
            VertexElementFormat::Short4 => 4,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum VertexElementUsage {
    Position,
    Normal,
    TextureCoordinate,
    Color,
    Tangent,
    Binormal,
}

#[derive(Clone, Debug)]
pub struct VertexElement {
    pub offset: u16,
    pub format: VertexElementFormat,
    pub usage: VertexElementUsage,
    pub usage_index: u8,
}

#[derive(Clone, Debug)]
pub struct VertexDeclaration {
    pub stride: u16,
    pub elements: Vec<VertexElement>,
}

pub trait HasVertexDeclaration {
    fn vertex_declaration() -> VertexDeclaration;
}

impl HasVertexDeclaration for VertexData {
    fn vertex_declaration() -> VertexDeclaration {
        VertexDeclaration {
            stride: mem::size_of::<VertexData>() as u16,
            elements: vec![
                VertexElement {
                    offset: mem::offset_of!(VertexData, position) as u16,
                    format: VertexElementFormat::Float2,
                    usage: VertexElementUsage::Position,
                    usage_index: 0,
                },
                VertexElement {
                    offset: mem::offset_of!(VertexData, color) as u16,
                    format: VertexElementFormat::Color,
                    usage: VertexElementUsage::Color,
                    usage_index: 0,
                },
                VertexElement {
                    offset: mem::offset_of!(VertexData, uv) as u16,
                    format: VertexElementFormat::Float2,
                    usage: VertexElementUsage::TextureCoordinate,
                    usage_index: 0,
                },
            ],
        }
    }
}
