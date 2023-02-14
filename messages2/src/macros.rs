macro_rules! transient_from {
    ($source:ident, $interm:ident, $target:ident) => {
        impl From<$source> for $target {
            fn from(value: $source) -> $target {
                let interm = $interm::from(value);
                $target::from(interm)
            }
        }
    };

    ($source:ident, $first:ident, $second:ident, $target:ident) => {
        impl From<$source> for $target {
            fn from(value: $source) -> $target {
                let first = $first::from(value);
                let second = $second::from(first);
                $target::from(second)
            }
        }
    };
}

pub(crate) use transient_from;
