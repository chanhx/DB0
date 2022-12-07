mod types;

pub use types::{DataType, Row, Value};

use {
    byteorder::{ReadBytesExt, WriteBytesExt, LE},
    common::{
        dsa::bitmap::{Bitmap, BitmapMut},
        pub_fields_struct,
    },
    snafu::{prelude::*, Backtrace, ResultExt},
    std::{
        convert::TryInto,
        io::{self, Cursor},
        str,
        string::FromUtf8Error,
    },
};

#[derive(Debug, Snafu)]
pub enum Error {
    FromIo {
        source: io::Error,
    },

    Utf8Encoding {
        source: FromUtf8Error,
    },

    #[snafu(display("internal error"))]
    Internal {
        backtrace: Backtrace,
    },

    TypeEncoding {
        #[snafu(backtrace)]
        source: types::Error,
    },

    #[snafu(display("the count of values does not match the count of attributes"))]
    ValuesCount {
        backtrace: Backtrace,
    },
}

pub type Result<T> = std::result::Result<T, Error>;

pub_fields_struct! {
    #[derive(Debug, PartialEq)]
    struct Attribute {
        name: String,
        num: i16,
        data_type: DataType,
        is_nullable: bool,
    }
}

impl Attribute {
    pub fn new(name: String, num: i16, data_type: DataType, is_nullable: bool) -> Self {
        Self {
            name,
            num,
            data_type,
            is_nullable,
        }
    }

    pub fn to_values(&self) -> Vec<Value> {
        let (ty, len) = self.data_type.value_repr();

        vec![
            Value::String(self.name.clone()),
            Value::SmallInt(self.num),
            Value::TinyUint(ty),
            Value::Int(len),
            Value::Boolean(self.is_nullable),
        ]
    }
}

impl TryFrom<Vec<Value>> for Attribute {
    type Error = Error;

    fn try_from(values: Vec<Value>) -> Result<Self> {
        let values: [Value; 5] = values.try_into().map_err(|_| InternalSnafu.build())?;
        let [name, num, ty, len, is_nullable] = values;

        Ok(Self {
            name: name.try_into().context(TypeEncodingSnafu)?,
            num: num.try_into().context(TypeEncodingSnafu)?,
            data_type: DataType::from_value_repr((ty, len)).context(TypeEncodingSnafu)?,
            is_nullable: is_nullable.try_into().context(TypeEncodingSnafu)?,
        })
    }
}

pub fn encode_values(rows: &[Row], attributes: &[Attribute]) -> Result<Vec<Vec<u8>>> {
    let (var_lens_byte_count, bitmap_byte_count) = bytes_repr_info(&attributes);
    let attr_count = attributes.len();

    rows.iter()
        .map(|row| {
            if row.len() != attr_count {
                return Err(ValuesCountSnafu.build());
            }

            let data_byte_count = row.iter().map(|v| v.byte_count()).sum::<usize>();
            let mut bytes = vec![0u8; var_lens_byte_count + bitmap_byte_count + data_byte_count];

            let (var_lens, rest) = bytes.split_at_mut(var_lens_byte_count);
            let mut var_lens_writer = Cursor::new(var_lens);

            let (bitmap, data_region) = rest.split_at_mut(bitmap_byte_count);
            let mut bitmap = BitmapMut::new(bitmap);
            let mut data_writer = Cursor::new(data_region);

            for (i, (v, attr)) in row.iter().zip(attributes.iter()).enumerate() {
                if attr.data_type.is_variable_length() {
                    var_lens_writer
                        .write_u16::<LE>(v.byte_count() as u16)
                        .context(FromIoSnafu)?;
                }

                if attr.is_nullable && matches!(v, Value::Null) {
                    bitmap.set_unchecked(i);
                }

                data_writer.write_value(v)?;
            }

            Ok(bytes)
        })
        .collect()
}

pub fn decode_values(bytes: &[Vec<u8>], attributes: &[Attribute]) -> Result<Vec<Row>> {
    let (var_lens_byte_count, bitmap_byte_count) = bytes_repr_info(&attributes);
    let data_region_start = var_lens_byte_count + bitmap_byte_count;

    bytes
        .iter()
        .map(|bytes| {
            let mut var_lens = Cursor::new(&bytes[..var_lens_byte_count]);
            let null_bitmap = Bitmap::new(&bytes[var_lens_byte_count..data_region_start]);
            let mut reader = Cursor::new(&bytes[data_region_start..]);

            attributes
                .iter()
                .enumerate()
                .map(|(i, attr)| {
                    if attr.is_nullable && null_bitmap.is_set_unchecked(i) {
                        return Ok(Value::Null);
                    }

                    if attr.data_type.is_variable_length() {
                        let len = var_lens.read_u16::<LE>().context(FromIoSnafu)? as usize;
                        reader.read_string(len)
                    } else {
                        reader.read_fixed_size_value(&attr.data_type)
                    }
                })
                .collect()
        })
        .collect()
}

fn bytes_repr_info(attributes: &[Attribute]) -> (usize, usize) {
    let mut contain_nullable = false;

    let var_len_area_byte_count = attributes
        .iter()
        .inspect(|attr| contain_nullable |= attr.is_nullable)
        .filter(|attr| attr.data_type.is_variable_length())
        .map(|_| {
            // TODO: there should be a type table recording the max length of each variable-length type
            2usize
        })
        .sum();

    let bitmap_byte_count = if contain_nullable {
        (attributes.len() + 7) / 8
    } else {
        0
    };

    (var_len_area_byte_count, bitmap_byte_count)
}

trait ReadValue: io::Read {
    fn read_fixed_size_value(&mut self, data_type: &DataType) -> Result<Value> {
        Ok(match data_type {
            DataType::Boolean => Value::Boolean({
                match self.read_u8().context(FromIoSnafu)? {
                    0 => false,
                    1 => true,
                    _ => return Err(InternalSnafu.build()),
                }
            }),

            DataType::TinyInt => Value::TinyInt(self.read_i8().context(FromIoSnafu)?),
            DataType::SmallInt => Value::SmallInt(self.read_i16::<LE>().context(FromIoSnafu)?),
            DataType::Int => Value::Int(self.read_i32::<LE>().context(FromIoSnafu)?),
            DataType::BigInt => Value::BigInt(self.read_i64::<LE>().context(FromIoSnafu)?),

            DataType::TinyUint => Value::TinyUint(self.read_u8().context(FromIoSnafu)?),
            DataType::SmallUint => Value::SmallUint(self.read_u16::<LE>().context(FromIoSnafu)?),
            DataType::Uint => Value::Uint(self.read_u32::<LE>().context(FromIoSnafu)?),
            DataType::BigUint => Value::BigUint(self.read_u64::<LE>().context(FromIoSnafu)?),

            DataType::Float => Value::Float(self.read_f32::<LE>().context(FromIoSnafu)?),
            DataType::Decimal => Value::Decimal(self.read_f64::<LE>().context(FromIoSnafu)?),

            DataType::Char(len) => self.read_string(*len as usize)?,
            DataType::Varchar(_) => return Err(InternalSnafu.build()),
        })
    }

    fn read_string(&mut self, len: usize) -> Result<Value> {
        let mut buf = vec![0; len];
        self.read_exact(&mut buf).context(FromIoSnafu)?;

        Ok(Value::String(
            String::from_utf8(buf).context(Utf8EncodingSnafu)?,
        ))
    }
}

impl<T> ReadValue for Cursor<T> where T: AsRef<[u8]> {}

trait WriteValue: io::Write {
    fn write_value(&mut self, value: &Value) -> Result<()> {
        match value {
            Value::Null => return Ok(()),
            Value::Boolean(v) => self.write_u8(if *v { 1 } else { 0 }),

            Value::TinyInt(v) => self.write_i8(*v),
            Value::SmallInt(v) => self.write_i16::<LE>(*v),
            Value::Int(v) => self.write_i32::<LE>(*v),
            Value::BigInt(v) => self.write_i64::<LE>(*v),

            Value::TinyUint(v) => self.write_u8(*v),
            Value::SmallUint(v) => self.write_u16::<LE>(*v),
            Value::Uint(v) => self.write_u32::<LE>(*v),
            Value::BigUint(v) => self.write_u64::<LE>(*v),

            Value::Float(v) => self.write_f32::<LE>(*v),
            Value::Decimal(v) => self.write_f64::<LE>(*v),

            Value::String(s) => {
                self.write(s.as_bytes()).context(FromIoSnafu)?;
                return Ok(());
            }
        }
        .context(FromIoSnafu)
    }
}

impl WriteValue for Cursor<&mut [u8]> {}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn build_attribute() {
        let meta_attributes = [
            Attribute::new("name".to_string(), 0, DataType::Varchar(20), false),
            Attribute::new("num".to_string(), 1, DataType::SmallInt, false),
            Attribute::new("type".to_string(), 2, DataType::TinyInt, false),
            Attribute::new("length".to_string(), 3, DataType::Int, false),
            Attribute::new("is_nullable".to_string(), 3, DataType::Boolean, false),
        ];

        meta_attributes.iter().for_each(|attr| {
            let values = attr.to_values();
            let attr_from_values: Attribute = values.try_into().unwrap();

            assert_eq!(*attr, attr_from_values);
        })
    }

    #[test]
    fn encode_decode() {
        let attrs = [
            Attribute::new("name".to_string(), 0, DataType::Varchar(6), false),
            Attribute::new("phone".to_string(), 1, DataType::Char(5), true),
            Attribute::new("age".to_string(), 2, DataType::TinyInt, true),
        ];

        let rows = vec![
            vec![
                Value::String("abc".into()),
                Value::String("12345".into()),
                Value::TinyInt(16),
            ],
            vec![
                Value::String("def".into()),
                Value::String("45678".into()),
                Value::Null,
            ],
            vec![Value::String("abcde".into()), Value::Null, Value::Null],
        ];

        let bytes = encode_values(&rows, &attrs).unwrap();

        let values_from_bytes = decode_values(&bytes, &attrs).unwrap();

        assert_eq!(rows, values_from_bytes);
    }
}
