macro_rules! resolve_clone {
    ($name:ident) => (
        impl Resolve for $name {
            type Output = Self;
            fn resolve(&self, options: &Options) -> Self::Output {
                self.clone()
            }
        }
    )
}
macro_rules! primitive_interpolate {
    ($name:ident) => (
        impl Interpolate for $name {
            #[inline]
            fn lerp(self, to: Self, x: f32) -> Self {
                self * (1.0 - x) + to * x
            }
            #[inline]
            fn scale(self, x: f32) -> Self {
                self * x
            }
        }
    )
}
macro_rules! wrap_interpolate {
    ($name:ident) => (
        impl Interpolate for $name {
            #[inline]
            fn lerp(self, to: Self, x: f32) -> Self {
                $name(self.0.lerp(to.0, x))
            }
            #[inline]
            fn scale(self, x: f32) -> Self {
                $name(self.0.scale(x))
            }
        }
    )
}
macro_rules! wrap_compose {
    ($name:ident) => (
        impl Compose for $name {
            #[inline]
            fn compose(self, rhs: Self) -> Self {
                $name(self.0.compose(rhs.0))
            }
        }
    )
}
macro_rules! wrap_option_iterpolate {
    ($name:ident) => {
        impl Interpolate for $name {
            fn lerp(self, to: Self, x: f32) -> Self {
                match (self.0, to.0) {
                    (Some(from), Some(to)) => $name(Some(from.lerp(to, x))),
                    (from, to) => $name(from.or(to))
                }
            }
            fn scale(self, x: f32) -> Self {
                $name(self.0.map(|v| v.scale(x)))
            }
        }
    };
}
macro_rules! get_or_return {
    ($opt:expr) => (
        match $opt {
            Some(val) => val,
            None => return
        }
    );
    ($opt:expr, $msg:tt $(,$args:tt)*) => (
        match $opt {
            Some(val) => val,
            None => {
                println!($msg $(,$args)*);
                return;
            }
        }
    );
}
macro_rules! get_ref_or_return {
    ($opt:expr) => (
        match $opt {
            Some(ref val) => val,
            None => return
        }
    );
    ($opt:expr, $msg:tt $(,$args:tt)*) => (
        match $opt {
            Some(ref val) => val,
            None => {
                println!($msg $(,$args)*);
                return;
            }
        }
    );
}
