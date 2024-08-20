use crate::gl::types::float16;

macro_rules! impl_type {
    ($ty:ty as $symbol:ident) => {
        impl Type for $ty {
            const ID: u32 = ::glb::$symbol;
        }
    };
}


// Packed pixel formats Table 8.5
pub mod types {

}

// TODO: Fixed type points to representation

pub trait Type {
    const ID: u32;
}
 
impl_type! { u8      as UNSIGNED_BYTE                }
impl_type! { i8      as BYTE                         }
impl_type! { u16     as UNSIGNED_SHORT               }
impl_type! { i16     as SHORT                        }
impl_type! { u32     as UNSIGNED_INT                 }
impl_type! { i32     as INT                          }
impl_type! { float16 as HALF_FLOAT                   }
impl_type! { f32     as FLOAT                        }
// impl_type! { u8      as UNSIGNED_BYTE_3_3_2          }
// impl_type! { u8      as UNSIGNED_BYTE_2_3_3_REV      }
// impl_type! { u16     as UNSIGNED_SHORT_5_6_5         }
// impl_type! { u16     as UNSIGNED_SHORT_5_6_5_REV     }
// impl_type! { u16     as UNSIGNED_SHORT_4_4_4_4       }
// impl_type! { u16     as UNSIGNED_SHORT_4_4_4_4_REV   }
// impl_type! { u16     as UNSIGNED_SHORT_5_5_5_1       }
// impl_type! { u16     as UNSIGNED_SHORT_1_5_5_5_REV   }
// impl_type! { u32     as UNSIGNED_INT_8_8_8_8         }
// impl_type! { u32     as UNSIGNED_INT_8_8_8_8_REV     }
// impl_type! { u32     as UNSIGNED_INT_10_10_10_2      }
// impl_type! { u32     as UNSIGNED_INT_2_10_10_10_REV  }
// impl_type! { u32     as UNSIGNED_INT_24_8            }
// impl_type! { u32     as UNSIGNED_INT_10F_11F_11F_REV }
// impl_type! { u32     as UNSIGNED_INT_5_9_9_9_REV     }
// impl_type! { n/a as FLOAT_32_UNSIGNED_INT_24_8_REV }

pub trait Format {
    const ID: u32;
}

/// Base internal formats specify if its depth, depth/stencil, RED, RG, RGB, RGBA or stencil index.
/// These can be derived from other internal formats in short
/// For each internal format has associated with it base internal format.

/// Internal format maybe a 
/// - 8.11 base internal format, 
/// - 8.12 / 8.13 sized internal format, 
/// - 8.14 generic compressed internal formats, or 
/// - if listed in 8.14 specific compressed internal formats
/// Target determinuje czy base internal format jest valid
/// format określa jakie dane mają być przesłane.
///  
/// > Textures with a base internal format of DEPTH_COMPONENT or DEPTH_
///   STENCIL require either depth component data or depth/stencil component data.
///   Textures with other base internal formats require RGBA component data.
/// 
/// #  internal component resolution
/// The internal component resolution is the number of bits allocated to each value
///  in a texture image. 
/// 
///  NOTE: If internalformat is specified as a base internal format, the GL stores the resulting texture with internal component resolutions of its own chooeing, 
///  NOTE: referred to as the effective internal format
/// 
/// TODO: quivalent to the mapping of the corresponding base internal format’s components,
 /// TODO: as specified in table 8.11; the type (unsigned int, float, etc.) is assigned the same
 /// TODO: type specified by internalformat; and the memory allocation per texture component
 /// TODO: is assigned by the GL to match the allocations listed in tables 8.12- 8.13 as closely
 /// TODO: as possible. (The definition of closely is left up to the implementation. However,
 /// TODO: a non-zero number of bits must be allocated for each component whose desired
 /// TODO: allocation in tables 8.12- 8.13 is non-zero, and zero bits must be allocated for all
 /// TODO: other components).
/// 
/// Table 8.14
pub trait InternalFormat {
    const ID: u32;
}

// COMPRESSED_RED RED Generic unorm
// COMPRESSED_RG RG Generic unorm
// COMPRESSED_RGB RGB Generic unorm
// COMPRESSED_RGBA RGBA Generic unorm
// COMPRESSED_SRGB RGB Generic unorm
// COMPRESSED_SRGB_ALPHA RGBA Generic unorm
// COMPRESSED_RED_RGTC1 RED Specific unorm
// COMPRESSED_SIGNED_RED_RGTC1 RED Specific snorm
// COMPRESSED_RG_RGTC2 RG Specific unorm
// COMPRESSED_SIGNED_RG_RGTC2 RG Specific snorm
// COMPRESSED_RGBA_BPTC_UNORM RGBA Specific unorm
// COMPRESSED_SRGB_ALPHA_BPTC_UNORM RGBA Specific unorm COMPRESSED_RGB_BPTC_SIGNED_
// FLOAT RGB Specific float COMPRESSED_RGB_BPTC_UNSIGNED_
// FLOAT RGB Specific float
// COMPRESSED_RGB8_ETC2 RGB Specific unorm
// COMPRESSED_SRGB8_ETC2 RGB Specific unorm
// COMPRESSED_RGB8_PUNCHTHROUGH_
// ALPHA1_ETC2 RGB Specific unorm
// COMPRESSED_SRGB8_PUNCHTHROUGH_ALPHA1_ETC2 RGB Specific unorm
// COMPRESSED_RGBA8_ETC2_EAC RGBA Specific unorm
// COMPRESSED_SRGB8_ALPHA8_ETC2_EAC RGBA Specific unorm
// COMPRESSED_R11_EAC RED Specific unorm
// COMPRESSED_SIGNED_R11_EAC RED Specific snorm
// COMPRESSED_RG11_EAC RG Specific unorm
// COMPRESSED_SIGNED_RG11_EAC RG Specific snorm



