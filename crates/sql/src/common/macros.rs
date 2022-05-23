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

pub(crate) use pub_fields_struct;
