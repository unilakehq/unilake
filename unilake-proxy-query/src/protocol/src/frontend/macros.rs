macro_rules! uint_enum {
    ($( #[$gattr:meta] )* pub enum $ty:ident { $( $( #[$attr:meta] )* $variant:ident = $val:expr,)* }) => {
        uint_enum!($( #[$gattr ])* (pub) enum $ty { $( $( #[$attr] )* $variant = $val, )* });
    };
    ($( #[$gattr:meta] )* enum $ty:ident { $( $( #[$attr:meta] )* $variant:ident = $val:expr,)* }) => {
        #[allow(missing_docs)]
        uint_enum!($( #[$gattr ])* () enum $ty { $( $( #[$attr] )* $variant = $val, )* });
    };

    ($( #[$gattr:meta] )* ( $($vis:tt)* ) enum $ty:ident { $( $( #[$attr:meta] )* $variant:ident = $val:expr,)* }) => {
        #[derive(Debug, Copy, Clone, PartialEq)]
        $( #[$gattr] )*
        #[allow(missing_docs)]
        $( $vis )* enum $ty {
            $( $( #[$attr ])* $variant = $val, )*
        }

        impl ::std::convert::TryFrom<u8> for $ty {
            type Error = ();
            fn try_from(n: u8) -> ::std::result::Result<$ty, ()> {
                match n {
                    $( x if x == $ty::$variant as u8 => Ok($ty::$variant), )*
                    _ => Err(()),
                }
            }
        }

        impl ::std::convert::TryFrom<u16> for $ty {
            type Error = ();
            fn try_from(n: u16) -> ::std::result::Result<$ty, ()> {
                match n {
                    $( x if x == $ty::$variant as u16 => Ok($ty::$variant), )*
                    _ => Err(()),
                }
            }
        }

        impl ::std::convert::TryFrom<u32> for $ty {
            type Error = ();
            fn try_from(n: u32) -> ::std::result::Result<$ty, ()> {
                match n {
                    $( x if x == $ty::$variant as u32 => Ok($ty::$variant), )*
                    _ => Err(()),
                }
            }
        }
    }
}
