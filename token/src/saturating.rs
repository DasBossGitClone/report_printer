#![allow(dead_code)]
#![allow(unused_comparisons)]
use ::std::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Neg, Rem, Shl, Shr, Sub};

/// Feature depended trait for saturating arithmetic operations.
///
/// When the `saturating_arithmetic` feature is enabled, the methods
/// will perform saturating arithmetic operations.
///
/// When the feature is disabled, the methods will perform standard
/// arithmetic operations (which may panic on overflow).
pub trait SaturatingArithmetic<T>: Sized {
    fn sat_add(self, rhs: T) -> Self
    where
        Self: Add<Self, Output = Self>;
    fn sat_sub(self, rhs: T) -> Self
    where
        Self: Sub<Self, Output = Self>;
    fn sat_mul(self, rhs: T) -> Self
    where
        Self: Mul<Self, Output = Self>;
    fn sat_div(self, rhs: T) -> Self
    where
        Self: Div<Self, Output = Self>;
    fn sat_rem(self, rhs: T) -> Self
    where
        Self: Rem<Self, Output = Self>;
    fn sat_abs(self) -> Self
    where
        Self: Sized + PartialOrd + Neg<Output = Self> + Copy;
    fn sat_add_assign(&mut self, rhs: T)
    where
        Self: Add<Self, Output = Self>;
    fn sat_sub_assign(&mut self, rhs: T)
    where
        Self: Sub<Self, Output = Self>;
    fn sat_mul_assign(&mut self, rhs: T)
    where
        Self: Mul<Self, Output = Self>;
    fn sat_div_assign(&mut self, rhs: T)
    where
        Self: Div<Self, Output = Self>;
    fn sat_rem_assign(&mut self, rhs: T)
    where
        Self: Rem<Self, Output = Self>;
    fn sat_bitand(self, rhs: T) -> Self
    where
        Self: Sized + BitAnd<Self, Output = Self>;
    fn sat_bitor(self, rhs: T) -> Self
    where
        Self: Sized + BitOr<Self, Output = Self>;
    fn sat_bitxor(self, rhs: T) -> Self
    where
        Self: Sized + BitXor<Self, Output = Self>;
    fn sat_bitand_assign(&mut self, rhs: T)
    where
        Self: BitAnd<Self, Output = Self>;
    fn sat_bitor_assign(&mut self, rhs: T)
    where
        Self: BitOr<Self, Output = Self>;
    fn sat_bitxor_assign(&mut self, rhs: T)
    where
        Self: BitXor<Self, Output = Self>;
    fn sat_shl(self, rhs: u32) -> Self
    where
        Self: Sized + Shl<u32, Output = Self>;
    fn sat_shr(self, rhs: u32) -> Self
    where
        Self: Sized + Shr<u32, Output = Self>;
    fn sat_shl_assign(&mut self, rhs: u32)
    where
        Self: Shl<u32, Output = Self>;
    fn sat_shr_assign(&mut self, rhs: u32)
    where
        Self: Shr<u32, Output = Self>;
}

#[allow(unused)]
macro_rules! impl_saturating_arithmetic {
    ($self_ty:ty, $other_ty:ty, $self:ident, $other:ident,
    [
        (
            $sat_op_1:expr,
            $std_op_1:expr
            $(,)?
        ),
        (
            $sat_op_2:expr,
            $std_op_2:expr
            $(,)?
        ),
        (
            $sat_op_3:expr,
            $std_op_3:expr
            $(,)?
        ),
        (
            $sat_op_4:expr,
            $std_op_4:expr
            $(,)?
        ),
        (
            $sat_op_5:expr,
            $std_op_5:expr
            $(,)?
        ),
        (
            $sat_op_6:expr,
            $std_op_6:expr
            $(,)?
        ),
        (
            $sat_op_7:expr,
            $std_op_7:expr
            $(,)?
        ),
        (
            $sat_op_8:expr,
            $std_op_8:expr
            $(,)?
        ),
        (
            $sat_op_9:expr,
            $std_op_9:expr
            $(,)?
        ),
        (
            $sat_op_10:expr,
            $std_op_10:expr
            $(,)?
        ),
        (
            $sat_op_11:expr,
            $std_op_11:expr
            $(,)?
        ),
        (
            $sat_op_12:expr,
            $std_op_12:expr
            $(,)?
        ),
        (
            $sat_op_13:expr,
            $std_op_13:expr
            $(,)?
        ),
        (
            $sat_op_14:expr,
            $std_op_14:expr
            $(,)?
        ),
        (
            $sat_op_15:expr,
            $std_op_15:expr
            $(,)?
        ),
        (
            $sat_op_16:expr,
            $std_op_16:expr
            $(,)?
        ),
        (
            $sat_op_17:expr,
            $std_op_17:expr
            $(,)?
        ),
        (
            $sat_op_18:expr,
            $std_op_18:expr
            $(,)?
        ),
        (
            $sat_op_19:expr,
            $std_op_19:expr
            $(,)?
        ),
        (
            $sat_op_20:expr,
            $std_op_20:expr
            $(,)?
        ),
        (
            $sat_op_21:expr,
            $std_op_21:expr
            $(,)?
        )
        $(,)?
    ]) => {
        impl SaturatingArithmetic<$other_ty> for $self_ty {
            #[inline(always)]
            fn sat_add(self, rhs: $other_ty) -> $self_ty
            where
                Self: Add<$self_ty, Output = $self_ty>,
            {
                let $self = self as $self_ty;
                let $other = rhs as $self_ty;
                #[cfg(feature = "saturating_arithmetic")]
                {
                    $sat_op_1
                }
                #[cfg(not(feature = "saturating_arithmetic"))]
                {
                    $std_op_1
                }
            }
            #[inline(always)]
            fn sat_sub(self, rhs: $other_ty) -> $self_ty
            where
                Self: Sub<$self_ty, Output = $self_ty>,
            {
                let $self = self as $self_ty;
                let $other = rhs as $self_ty;
                #[cfg(feature = "saturating_arithmetic")]
                {
                    $sat_op_2
                }
                #[cfg(not(feature = "saturating_arithmetic"))]
                {
                    $std_op_2
                }
            }
            #[inline(always)]
            fn sat_mul(self, rhs: $other_ty) -> $self_ty
            where
                Self: Mul<$self_ty, Output = $self_ty>,
            {
                let $self = self as $self_ty;
                let $other = rhs as $self_ty;
                #[cfg(feature = "saturating_arithmetic")]
                {
                    $sat_op_3
                }
                #[cfg(not(feature = "saturating_arithmetic"))]
                {
                    $std_op_3
                }
            }
            #[inline(always)]
            fn sat_div(self, rhs: $other_ty) -> $self_ty
            where
                Self: Div<$self_ty, Output = $self_ty>,
            {
                let $self = self as $self_ty;
                let $other = rhs as $self_ty;
                #[cfg(feature = "saturating_arithmetic")]
                {
                    $sat_op_4
                }
                #[cfg(not(feature = "saturating_arithmetic"))]
                {
                    $std_op_4
                }
            }
            #[inline(always)]
            fn sat_rem(self, rhs: $other_ty) -> $self_ty
            where
                Self: Rem<$self_ty, Output = $self_ty>,
            {
                let $self = self as $self_ty;
                let $other = rhs as $self_ty;
                #[cfg(feature = "saturating_arithmetic")]
                {
                    $sat_op_5
                }
                #[cfg(not(feature = "saturating_arithmetic"))]
                {
                    $std_op_5
                }
            }
            #[inline(always)]
            fn sat_abs(self) -> $self_ty
            where
                Self: Sized + PartialOrd + Neg<Output = $self_ty> + Copy,
            {
                let $self = self;
                #[cfg(feature = "saturating_arithmetic")]
                {
                    $sat_op_6
                }
                #[cfg(not(feature = "saturating_arithmetic"))]
                {
                    $std_op_6
                }
            }
            #[inline(always)]
            fn sat_add_assign(&mut self, rhs: $other_ty)
            where
                Self: Add<$self_ty, Output = $self_ty>,
            {
                let $self = self as &mut $self_ty;
                let $other = rhs as $self_ty;
                #[cfg(feature = "saturating_arithmetic")]
                {
                    $sat_op_7
                }
                #[cfg(not(feature = "saturating_arithmetic"))]
                {
                    $std_op_7
                }
            }
            #[inline(always)]
            fn sat_sub_assign(&mut self, rhs: $other_ty)
            where
                Self: Sub<$self_ty, Output = $self_ty>,
            {
                let $self = self as &mut $self_ty;
                let $other = rhs as $self_ty;
                #[cfg(feature = "saturating_arithmetic")]
                {
                    $sat_op_8
                }
                #[cfg(not(feature = "saturating_arithmetic"))]
                {
                    $std_op_8
                }
            }
            #[inline(always)]
            fn sat_mul_assign(&mut self, rhs: $other_ty)
            where
                Self: Mul<$self_ty, Output = $self_ty>,
            {
                let $self = self as &mut $self_ty;
                let $other = rhs as $self_ty;
                #[cfg(feature = "saturating_arithmetic")]
                {
                    $sat_op_9
                }
                #[cfg(not(feature = "saturating_arithmetic"))]
                {
                    $std_op_9
                }
            }
            #[inline(always)]
            fn sat_div_assign(&mut self, rhs: $other_ty)
            where
                Self: Div<$self_ty, Output = $self_ty>,
            {
                let $self = self as &mut $self_ty;
                let $other = rhs as $self_ty;
                #[cfg(feature = "saturating_arithmetic")]
                {
                    $sat_op_10
                }
                #[cfg(not(feature = "saturating_arithmetic"))]
                {
                    $std_op_10
                }
            }
            #[inline(always)]
            fn sat_rem_assign(&mut self, rhs: $other_ty)
            where
                Self: Rem<$self_ty, Output = $self_ty>,
            {
                let $self = self as &mut $self_ty;
                let $other = rhs as $self_ty;
                #[cfg(feature = "saturating_arithmetic")]
                {
                    $sat_op_11
                }
                #[cfg(not(feature = "saturating_arithmetic"))]
                {
                    $std_op_11
                }
            }
            #[inline(always)]
            fn sat_bitand(self, rhs: $other_ty) -> $self_ty
            where
                Self: Sized + BitAnd<$self_ty, Output = $self_ty>,
            {
                let $self = self as $self_ty;
                let $other = rhs as $self_ty;
                #[cfg(feature = "saturating_arithmetic")]
                {
                    $sat_op_12
                }
                #[cfg(not(feature = "saturating_arithmetic"))]
                {
                    $std_op_12
                }
            }
            #[inline(always)]
            fn sat_bitor(self, rhs: $other_ty) -> $self_ty
            where
                Self: Sized + BitOr<$self_ty, Output = $self_ty>,
            {
                let $self = self as $self_ty;
                let $other = rhs as $self_ty;
                #[cfg(feature = "saturating_arithmetic")]
                {
                    $sat_op_13
                }
                #[cfg(not(feature = "saturating_arithmetic"))]
                {
                    $std_op_13
                }
            }
            #[inline(always)]
            fn sat_bitxor(self, rhs: $other_ty) -> $self_ty
            where
                Self: Sized + BitXor<$self_ty, Output = $self_ty>,
            {
                let $self = self as $self_ty;
                let $other = rhs as $self_ty;
                #[cfg(feature = "saturating_arithmetic")]
                {
                    $sat_op_14
                }
                #[cfg(not(feature = "saturating_arithmetic"))]
                {
                    $std_op_14
                }
            }
            #[inline(always)]
            fn sat_bitand_assign(&mut self, rhs: $other_ty)
            where
                Self: BitAnd<$self_ty, Output = $self_ty>,
            {
                let $self = self as &mut $self_ty;
                let $other = rhs as $self_ty;
                #[cfg(feature = "saturating_arithmetic")]
                {
                    $sat_op_15
                }
                #[cfg(not(feature = "saturating_arithmetic"))]
                {
                    $std_op_15
                }
            }
            #[inline(always)]
            fn sat_bitor_assign(&mut self, rhs: $other_ty)
            where
                Self: BitOr<$self_ty, Output = $self_ty>,
            {
                let $self = self as &mut $self_ty;
                let $other = rhs as $self_ty;
                #[cfg(feature = "saturating_arithmetic")]
                {
                    $sat_op_16
                }
                #[cfg(not(feature = "saturating_arithmetic"))]
                {
                    $std_op_16
                }
            }
            #[inline(always)]
            fn sat_bitxor_assign(&mut self, rhs: $other_ty)
            where
                Self: BitXor<$self_ty, Output = $self_ty>,
            {
                let $self = self as &mut $self_ty;
                let $other = rhs as $self_ty;
                #[cfg(feature = "saturating_arithmetic")]
                {
                    $sat_op_17
                }
                #[cfg(not(feature = "saturating_arithmetic"))]
                {
                    $std_op_17
                }
            }
            #[inline(always)]
            fn sat_shl(self, rhs: u32) -> $self_ty
            where
                Self: Sized + Shl<u32, Output = $self_ty>,
            {
                let $self = self as $self_ty;
                let $other = rhs as $self_ty;
                #[cfg(feature = "saturating_arithmetic")]
                {
                    $sat_op_18
                }
                #[cfg(not(feature = "saturating_arithmetic"))]
                {
                    $std_op_18
                }
            }
            #[inline(always)]
            fn sat_shr(self, rhs: u32) -> $self_ty
            where
                Self: Sized + Shr<u32, Output = $self_ty>,
            {
                let $self = self as $self_ty;
                let $other = rhs as $self_ty;
                #[cfg(feature = "saturating_arithmetic")]
                {
                    $sat_op_19
                }
                #[cfg(not(feature = "saturating_arithmetic"))]
                {
                    $std_op_19
                }
            }
            #[inline(always)]
            fn sat_shl_assign(&mut self, rhs: u32)
            where
                Self: Shl<u32, Output = $self_ty>,
            {
                let $self = self as &mut $self_ty;
                let $other = rhs as $self_ty;
                #[cfg(feature = "saturating_arithmetic")]
                {
                    $sat_op_20
                }
                #[cfg(not(feature = "saturating_arithmetic"))]
                {
                    $std_op_20
                }
            }
            #[inline(always)]
            fn sat_shr_assign(&mut self, rhs: u32)
            where
                Self: Shr<u32, Output = $self_ty>,
            {
                let $self = self as &mut $self_ty;
                let $other = rhs as $self_ty;
                #[cfg(feature = "saturating_arithmetic")]
                {
                    $sat_op_21
                }
                #[cfg(not(feature = "saturating_arithmetic"))]
                {
                    $std_op_21
                }
            }
        }
    };
    (
        @[
            $(
                $self_ty:ty,
                $other_ty:ty,
                $self:ident,
                $other:ident,
                [
                    (
                        $sat_op_1:expr,
                        $std_op_1:expr
                        $(,)?
                    ),
                    (
                        $sat_op_2:expr,
                        $std_op_2:expr
                        $(,)?
                    ),
                    (
                        $sat_op_3:expr,
                        $std_op_3:expr
                        $(,)?
                    ),
                    (
                        $sat_op_4:expr,
                        $std_op_4:expr
                        $(,)?
                    ),
                    (
                        $sat_op_5:expr,
                        $std_op_5:expr
                        $(,)?
                    ),
                    (
                        $sat_op_6:expr,
                        $std_op_6:expr
                        $(,)?
                    ),
                    (
                        $sat_op_7:expr,
                        $std_op_7:expr
                        $(,)?
                    ),
                    (
                        $sat_op_8:expr,
                        $std_op_8:expr
                        $(,)?
                    ),
                    (
                        $sat_op_9:expr,
                        $std_op_9:expr
                        $(,)?
                    ),
                    (
                        $sat_op_10:expr,
                        $std_op_10:expr
                        $(,)?
                    ),
                    (
                        $sat_op_11:expr,
                        $std_op_11:expr
                        $(,)?
                    ),
                    (
                        $sat_op_12:expr,
                        $std_op_12:expr
                        $(,)?
                    ),
                    (
                        $sat_op_13:expr,
                        $std_op_13:expr
                        $(,)?
                    ),
                    (
                        $sat_op_14:expr,
                        $std_op_14:expr
                        $(,)?
                    ),
                    (
                        $sat_op_15:expr,
                        $std_op_15:expr
                        $(,)?
                    ),
                    (
                        $sat_op_16:expr,
                        $std_op_16:expr
                        $(,)?
                    ),
                    (
                        $sat_op_17:expr,
                        $std_op_17:expr
                        $(,)?
                    ),
                    (
                        $sat_op_18:expr,
                        $std_op_18:expr
                        $(,)?
                    ),
                    (
                        $sat_op_19:expr,
                        $std_op_19:expr
                        $(,)?
                    ),
                    (
                        $sat_op_20:expr,
                        $std_op_20:expr
                        $(,)?
                    ),
                    (
                        $sat_op_21:expr,
                        $std_op_21:expr
                        $(,)?
                    )
                    $(,)?
                ]
            );*
            // Match trailing sep
            $(;)?
        ]
    ) => {
        $(
            impl_saturating_arithmetic!(
                $self_ty,
                $other_ty,
                $self,
                $other,
                [
                    (
                        $sat_op_1,
                        $std_op_1
                    ),
                    (
                        $sat_op_2,
                        $std_op_2
                    ),
                    (
                        $sat_op_3,
                        $std_op_3
                    ),
                    (
                        $sat_op_4,
                        $std_op_4
                    ),
                    (
                        $sat_op_5,
                        $std_op_5
                    ),
                    (
                        $sat_op_6,
                        $std_op_6
                    ),
                    (
                        $sat_op_7,
                        $std_op_7
                    ),
                    (
                        $sat_op_8,
                        $std_op_8
                    ),
                    (
                        $sat_op_9,
                        $std_op_9
                    ),
                    (
                        $sat_op_10,
                        $std_op_10
                    ),
                    (
                        $sat_op_11,
                        $std_op_11
                    ),
                    (
                        $sat_op_12,
                        $std_op_12
                    ),
                    (
                        $sat_op_13,
                        $std_op_13
                    ),
                    (
                        $sat_op_14,
                        $std_op_14
                    ),
                    (
                        $sat_op_15,
                        $std_op_15
                    ),
                    (
                        $sat_op_16,
                        $std_op_16
                    ),
                    (
                        $sat_op_17,
                        $std_op_17
                    ),
                    (
                        $sat_op_18,
                        $std_op_18
                    ),
                    (
                        $sat_op_19,
                        $std_op_19
                    ),
                    (
                        $sat_op_20,
                        $std_op_20
                    ),
                    (
                        $sat_op_21,
                        $std_op_21
                    )
                ]
            );
        )+
    };
    (
        @@[
            $(
                (
                    $self_ty:ty,
                    $other_ty:ty
                    // Match trailing sep
                    $(,)?
                )
            ),+
            // Match trailing sep
            $(,)?
        ],
        $self:ident,
        $other:ident,
        [
            (
                $sat_op_1:expr,
                $std_op_1:expr
                $(,)?
            ),
            (
                $sat_op_2:expr,
                $std_op_2:expr
                $(,)?
            ),
            (
                $sat_op_3:expr,
                $std_op_3:expr
                $(,)?
            ),
            (
                $sat_op_4:expr,
                $std_op_4:expr
                $(,)?
            ),
            (
                $sat_op_5:expr,
                $std_op_5:expr
                $(,)?
            ),
            (
                $sat_op_6:expr,
                $std_op_6:expr
                $(,)?
            ),
            (
                $sat_op_7:expr,
                $std_op_7:expr
                $(,)?
            ),
            (
                $sat_op_8:expr,
                $std_op_8:expr
                $(,)?
            ),
            (
                $sat_op_9:expr,
                $std_op_9:expr
                $(,)?
            ),
            (
                $sat_op_10:expr,
                $std_op_10:expr
                $(,)?
            ),
            (
                $sat_op_11:expr,
                $std_op_11:expr
                $(,)?
            ),
            (
                $sat_op_12:expr,
                $std_op_12:expr
                $(,)?
            ),
            (
                $sat_op_13:expr,
                $std_op_13:expr
                $(,)?
            ),
            (
                $sat_op_14:expr,
                $std_op_14:expr
                $(,)?
            ),
            (
                $sat_op_15:expr,
                $std_op_15:expr
                $(,)?
            ),
            (
                $sat_op_16:expr,
                $std_op_16:expr
                $(,)?
            ),
            (
                $sat_op_17:expr,
                $std_op_17:expr
                $(,)?
            ),
            (
                $sat_op_18:expr,
                $std_op_18:expr
                $(,)?
            ),
            (
                $sat_op_19:expr,
                $std_op_19:expr
                $(,)?
            ),
            (
                $sat_op_20:expr,
                $std_op_20:expr
                $(,)?
            ),
            (
                $sat_op_21:expr,
                $std_op_21:expr
                $(,)?
            )
            $(,)?
        ]
    ) => {
        $(
            impl_saturating_arithmetic!(
                $self_ty,
                $other_ty,
                $self,
                $other,
                [
                    (
                        $sat_op_1,
                        $std_op_1
                    ),
                    (
                        $sat_op_2,
                        $std_op_2
                    ),
                    (
                        $sat_op_3,
                        $std_op_3
                    ),
                    (
                        $sat_op_4,
                        $std_op_4
                    ),
                    (
                        $sat_op_5,
                        $std_op_5
                    ),
                    (
                        $sat_op_6,
                        $std_op_6
                    ),
                    (
                        $sat_op_7,
                        $std_op_7
                    ),
                    (
                        $sat_op_8,
                        $std_op_8
                    ),
                    (
                        $sat_op_9,
                        $std_op_9
                    ),
                    (
                        $sat_op_10,
                        $std_op_10
                    ),
                    (
                        $sat_op_11,
                        $std_op_11
                    ),
                    (
                        $sat_op_12,
                        $std_op_12
                    ),
                    (
                        $sat_op_13,
                        $std_op_13
                    ),
                    (
                        $sat_op_14,
                        $std_op_14
                    ),
                    (
                        $sat_op_15,
                        $std_op_15
                    ),
                    (
                        $sat_op_16,
                        $std_op_16
                    ),
                    (
                        $sat_op_17,
                        $std_op_17
                    ),
                    (
                        $sat_op_18,
                        $std_op_18
                    ),
                    (
                        $sat_op_19,
                        $std_op_19
                    ),
                    (
                        $sat_op_20,
                        $std_op_20
                    ),
                    (
                        $sat_op_21,
                        $std_op_21
                    )
                ]
            );
        )+
    };
}

impl_saturating_arithmetic!(
    @@[
        (u8,u8),(u8,u16),(u16,u8),(u8,u32),(u32,u8),(u8,u64),(u64,u8),(u8,usize),(usize,u8),
        (u16,u16),(u16,u32),(u32,u16),(u16,u64),(u64,u16),(u16,usize),(usize,u16),
        (u32,u32),(u32,u64),(u64,u32),(u32,usize),(usize,u32),
        (u64,u64),(u64,usize),(usize,u64),
        (usize,usize),
        (i8,i8),(i8,i16),(i16,i8),(i8,i32),(i32,i8),(i8,i64),(i64,i8),(i8,isize),(isize,i8),
        (i16,i16),(i16,i32),(i32,i16),(i16,i64),(i64,i16),(i16,isize),(isize,i16),
        (i32,i32),(i32,i64),(i64,i32),(i32,isize),(isize,i32),
        (i64,i64),(i64,isize),(isize,i64),
        (isize,isize),
        (u8,i8),(i8,u8),(i8,u16),(u16,i8),(i8,u32),(u32,i8),(i8,u64),(u64,i8),(i8,usize),(usize,i8),
        (u16,i16),(i16,u16),(i16,u32),(u32,i16),(i16,u64),(u64,i16),(i16,usize),(usize,i16),
        (u32,i32),(i32,u32),(i32,u64),(u64,i32),(i32,usize),(usize,i32),
        (u64,i64),(i64,u64),(i64,usize),(usize,i64),
        (usize,isize),(isize,usize)
    ],
    s,
    rhs,
    [
        (
            s.saturating_add(rhs),
            s + rhs
        ),
        (
            s.saturating_sub(rhs),
            s - rhs
        ),
        (
            s.saturating_mul(rhs),
            s * rhs
        ),
        (
            s.saturating_div(rhs),
            s / rhs
        ),
        (
            s.saturating_rem(rhs),
            s % rhs
        ),
        (
            if s == isize::MIN {
                isize::MAX
            } else if s < 0 {
                -s
            } else {
                s
            },
            s
        ),
        (
            *s = s.saturating_add(rhs),
            *s = *s + rhs,
        ),
        (
            *s = s.saturating_sub(rhs),
            *s = *s - rhs,
        ),
        (
            *s = s.saturating_mul(rhs),
            *s = *s * rhs,
        ),
        (
            *s = s.saturating_div(rhs),
            *s = *s / rhs,
        ),
        (
            *s = s.saturating_rem(rhs),
            *s = *s % rhs,
        ),
        (
            s & rhs,
            s & rhs
        ),
        (
            s | rhs,
            s | rhs
        ),
        (
            s ^ rhs,
            s ^ rhs
        ),
        (
            *s = s.saturating_bitand_assign(rhs),
            *s = *s & rhs
        ),
        (
            *s = s.saturating_bitor_assign(rhs),
            *s = *s | rhs
        ),
        (
            *s = s.saturating_bitxor_assign(rhs),
            *s = *s ^ rhs
        ),
        (
            if rhs >= Self::MAX { 0 } else { s << rhs },
            if rhs >= Self::MAX { 0 } else { s << rhs }
        ),
        (
            if rhs >= Self::MAX { 0 } else { s >> rhs },
            if rhs >= Self::MAX { 0 } else { s >> rhs }
        ),
        (
            *s = if rhs >= Self::MAX { 0 } else { *s << rhs },
            *s = if rhs >= Self::MAX { 0 } else { *s << rhs }
        ),
        (
            *s = if rhs >= Self::MAX { 0 } else { *s >> rhs },
            *s = if rhs >= Self::MAX { 0 } else { *s >> rhs }
        )
    ]
);

impl_saturating_arithmetic!(
    f32,
    f32,
    s,
    rhs,
    [
        (s.saturating_add(rhs), s + rhs),
        (s.saturating_sub(rhs), s - rhs),
        (s.saturating_mul(rhs), s * rhs),
        (s.saturating_div(rhs), s / rhs),
        (s.saturating_rem(rhs), s % rhs),
        (
            if s == isize::MIN {
                isize::MAX
            } else if s < 0 {
                -s
            } else {
                s
            },
            s
        ),
        (*s = s.saturating_add(rhs), *s = *s + rhs,),
        (*s = s.saturating_sub(rhs), *s = *s - rhs,),
        (*s = s.saturating_mul(rhs), *s = *s * rhs,),
        (*s = s.saturating_div(rhs), *s = *s / rhs,),
        (*s = s.saturating_rem(rhs), *s = *s % rhs,),
        (s & rhs, s & rhs),
        (s | rhs, s | rhs),
        (s ^ rhs, s ^ rhs),
        (*s = s.saturating_bitand_assign(rhs), *s = *s & rhs),
        (*s = s.saturating_bitor_assign(rhs), *s = *s | rhs),
        (*s = s.saturating_bitxor_assign(rhs), *s = *s ^ rhs),
        (
            if rhs >= Self::MAX {
                0 as Self
            } else {
                s << rhs as u32
            } as Self,
            if rhs >= Self::MAX {
                0 as Self
            } else {
                s << rhs as u32
            } as Self
        ),
        (
            if rhs >= Self::MAX {
                0 as Self
            } else {
                s >> rhs as u32
            } as Self,
            if rhs >= Self::MAX {
                0 as Self
            } else {
                s >> rhs as u32
            } as Self
        ),
        (
            *s = if rhs >= Self::MAX {
                0 as Self
            } else {
                *s << rhs as u32
            } as Self,
            *s = if rhs >= Self::MAX {
                0 as Self
            } else {
                *s << rhs as u32
            } as Self
        ),
        (
            *s = if rhs >= Self::MAX {
                0 as Self
            } else {
                *s >> rhs as u32
            } as Self,
            *s = if rhs >= Self::MAX {
                0 as Self
            } else {
                *s >> rhs as u32
            } as Self
        )
    ]
);
