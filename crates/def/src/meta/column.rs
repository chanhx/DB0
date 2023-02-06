use {crate::DataType, common::pub_fields_struct};

pub_fields_struct! {
    #[derive(Debug, Clone, PartialEq)]
    struct ColumnDef {
        name: String,
        data_type: DataType,
        is_nullable: bool,
    }
}

#[cfg(test)]
mod test {
    use {
        super::*,
        crate::{meta::Column, Value},
    };

    #[test]
    fn build_column() {
        [
            ("name", DataType::Varchar(20)),
            ("num", DataType::SmallInt),
            ("type", DataType::TinyInt),
            ("length", DataType::Int),
            ("is_nullable", DataType::Boolean),
        ]
        .into_iter()
        .enumerate()
        .map(|(i, (name, ty))| {
            let (type_id, type_len) = ty.value_repr();
            Column::new(
                1,
                i as i16 + 1,
                name.to_string(),
                type_id,
                type_len as u16,
                false,
            )
        })
        .for_each(|col| {
            let values: Vec<Value> = col.clone().into();
            let column_from_values: Column = values.try_into().unwrap();

            assert_eq!(col, column_from_values);
        });
    }
}
