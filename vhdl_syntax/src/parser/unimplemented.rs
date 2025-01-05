// TODO: Implement these parsing functions properly!

use crate::parser::Parser;
use crate::tokens::TokenStream;

impl<T: TokenStream> Parser<T> {
    pub fn discrete_range(&mut self) {}

    pub fn opt_signature(&mut self) {}

    pub fn signature(&mut self) {}

    pub fn attribute_designator(&mut self) {}

    pub fn function_call(&mut self) {}
}
