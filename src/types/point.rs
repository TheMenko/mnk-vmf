use chumsky::{error::Rich, prelude::*, select, text, Parser as _};

use crate::{
    parser::{InternalParser, TokenError, TokenSource},
    Parser,
};

#[derive(Debug, Clone)]
pub struct Point3D {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
