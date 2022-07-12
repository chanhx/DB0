#[macro_export]
macro_rules! pub_fields_struct {
    {
        $(
            $(#[$($attr:tt)*])*
            struct $name:ident $(<$($g:tt),*>)? {
                $($field:ident: $t:ty,)*
            }
        )*
    } => {
        $(
            $(#[$($attr)*])*
            pub struct $name $(<$($g,)*>)? {
                $(pub $field: $t),*
            }
        )*
    }
}
