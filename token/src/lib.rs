#![feature(iter_array_chunks, trivial_bounds)]
#![deny(dead_code, unused)]

use ::itertools::Itertools;
use ::misc_extensions::bool::*;
use ::std::{
    fmt::{Debug, Display},
    str::FromStr,
};

mod colors_chars;
pub use colors_chars::*;
mod token;
pub use token::*;
mod single_line_stream;
pub use single_line_stream::*;
mod multiline_stream;
pub use multiline_stream::*;
pub mod saturating;

#[macro_export(local_inner_macros)]
macro_rules! impl_field {
    ($($struct_name:ident, $field_name:ident, $field_type:ty $(, $return_type:ty as $expr:expr)?);*$(;)?) => {
        $(
            impl_field!(@coerce $struct_name, $field_name, $field_type $(, $return_type as $expr)?);
        )*
    };
    ($struct_name:ident, $field_name:ident, $field_type:ty $(, $return_type:ty as $expr:expr)?) => {
        impl_field!(@coerce $struct_name, $field_name, $field_type $(, $return_type as $expr)?);
    };
    (@coerce $struct_name:ident, $field_name:ident, $field_type:ty, $return_type:ty as $expr:expr) => {
        impl_field!(@expand $struct_name, $field_name, $field_type, $return_type, $expr);
    };
    (@coerce $struct_name:ident, $field_name:ident, $field_type:ty) => {
        impl_field!(@expand $struct_name, $field_name, $field_type, $field_type);
    };
    (@expand $struct_name:ident, $field_name:ident, $field_type:ty, $return_type:ty $(, $expr:expr)?) => {
        impl $struct_name {
            paste::paste! {
                pub fn [< $field_name >](&self) -> $return_type
                where
                    $field_type: Copy
                {
                    // If expr is Some, apply it
                    impl_field!(@expand_expr self.$field_name $(=> $expr)?)
                }
                pub fn [< $field_name _cloned>](&self) -> $return_type
                where
                    $field_type: Clone
                {
                    // If expr is Some, apply it
                    impl_field!(@expand_expr self.$field_name.clone() $(=> $expr)?)
                }
                pub fn [< $field_name _ref>]<'a>(&'a self) -> &'a $return_type {
                    // If expr is Some, apply it
                    impl_field!(@expand_expr &self.$field_name $(=> $expr)?)
                }
                pub fn [< $field_name _mut>]<'a>(&'a mut self) -> &'a mut $return_type {
                    // If expr is Some, apply it
                    impl_field!(@expand_expr &mut self.$field_name $(=> $expr)?)
                }
                $(
                    pub fn [< $field_name _as >]<T>(&self) -> T
                    where
                        T: From<$field_type>,
                    {
                        // If expr is Some, apply it
                        impl_field!(@expand_expr self.$field_name.clone().into() => $expr)
                    }
                )?
            }
        }
    };
    (@expand_expr $field:expr => $expr:expr) => {
        $field.$expr
    };
    (@expand_expr $field:expr ) => {{
        $field
    }}

}
