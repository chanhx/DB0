#[macro_export]
macro_rules! pub_fields_struct {
    {
        $(
            $(#[$($attr:tt)*])*
            struct $name:ident {
                $($(#[$meta:meta])* $field:ident: $t:ty,)*
            }
        )*
    } => {
        $(
            $(#[$($attr)*])*
            pub struct $name {
                $($(#[$meta])* pub $field: $t),*
            }
        )*
    }
}
