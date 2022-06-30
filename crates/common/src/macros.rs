#[macro_export]
macro_rules! pub_fields_struct {
    {
        $(
            $(#[$($attr:tt)*])*
            struct $name:ident {
                $($field:ident: $t:ty,)*
            }
        )*
    } => {
        $(
            $(#[$($attr)*])*
            pub struct $name {
                $(pub $field: $t),*
            }
        )*
    }
}
