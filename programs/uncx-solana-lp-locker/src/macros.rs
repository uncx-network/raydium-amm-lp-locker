#[macro_export]

macro_rules! debug {
    ($($arg:tt)*) => {
        #[cfg(feature = "anchor-debug")]
        {
            anchor_lang::prelude::msg!($($arg)*);
        }
    };
}

#[macro_export]

macro_rules! math_error {
    () => {{

        || {

            let error_code = $crate::error::UncxLpError::MathError;

            anchor_lang::prelude::msg!(
                "Error \"{}\" thrown at {}:{}",
                error_code,
                file!(),
                line!()
            );

            error_code
        }
    }};
}

// #[macro_export]

// macro_rules! normal_fp {
//     ($($arg:tt)*) => {
//         #[cfg(feature = "normal-fp")]
//         {
//         $($arg)*
//         }
//     };
// }
// #[macro_export]

// macro_rules! precise_calc {
//     ($($arg:tt)*) => {
//         #[cfg(feature = "precise")]
//         {
//         $($arg)*
//         }
//     };
// }
