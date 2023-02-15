/// Macro for providing upstream [`From`] implementations.
macro_rules! transient_from {
    // Implement [`From`] [`$source`] for [`$target`]
    // if [`From`] [`$interm`] for [`$target`] is implemented.
    ($source:ident, $interm:ident, $target:ident) => {
        impl From<$source> for $target {
            fn from(value: $source) -> $target {
                let interm = $interm::from(value);
                $target::from(interm)
            }
        }
    };

    // Implement [`From`] [`$first`] for [`$target`]
    // and [`From`] [`$source`] for [`$target`]
    // if [`From`] [`$second`] for [`$target`] is implemented.
    ($source:ident, $first:ident, $second:ident, $target:ident) => {
        impl From<$first> for $target {
            fn from(value: $first) -> $target {
                let second = $second::from(value);
                $target::from(second)
            }
        }

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
