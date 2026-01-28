macro_rules! string_enum {
    ($(#[$meta:meta])* $vis:vis enum $name:ident { $( $variant:ident => $label:expr, )+ $(,)? }) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        $vis enum $name {
            $( $variant, )+
        }

        impl $name {
            fn parse(value: &str) -> Option<Self> {
                match value {
                    $( $label => Some(Self::$variant), )+
                    _ => None,
                }
            }

            fn as_str(self) -> &'static str {
                match self {
                    $( Self::$variant => $label, )+
                }
            }
        }
    };
}
