/*!
 * Convert between datafusion and iceberg schema
*/

use anyhow::{anyhow, Result};

use std::{collections::HashMap, convert::TryInto, sync::Arc};

use arrow::datatypes::{DataType, Field, Fields, Schema as ArrowSchema, TimeUnit};

use crate::model::data_types::{PrimitiveType, StructField, StructType, Type};

impl TryInto<ArrowSchema> for &StructType {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<ArrowSchema, Self::Error> {
        let fields = self
            .fields
            .iter()
            .map(|field| {
                Ok(Field::new_dict(
                    &field.name,
                    (&field.field_type).try_into()?,
                    !field.required,
                    field.id as i64,
                    false,
                ))
            })
            .collect::<Result<_, anyhow::Error>>()?;
        let metadata = HashMap::new();
        Ok(ArrowSchema { fields, metadata })
    }
}

impl TryFrom<&ArrowSchema> for StructType {
    type Error = anyhow::Error;

    fn try_from(value: &ArrowSchema) -> Result<Self, Self::Error> {
        let fields = value
            .fields
            .iter()
            .map(|field| {
                Ok(StructField {
                    id: field
                        .dict_id()
                        .ok_or_else(|| anyhow!("Schema field is missing id."))?
                        as i32,
                    name: field.name().to_owned(),
                    required: !field.is_nullable(),
                    field_type: field.data_type().try_into()?,
                    doc: None,
                })
            })
            .collect::<Result<_, anyhow::Error>>()?;
        Ok(StructType { fields })
    }
}

impl TryFrom<&Type> for DataType {
    type Error = anyhow::Error;

    fn try_from(value: &Type) -> Result<Self, Self::Error> {
        match value {
            Type::Primitive(primitive) => match primitive {
                PrimitiveType::Boolean => Ok(DataType::Boolean),
                PrimitiveType::Int => Ok(DataType::Int32),
                PrimitiveType::Long => Ok(DataType::Int64),
                PrimitiveType::Float => Ok(DataType::Float32),
                PrimitiveType::Double => Ok(DataType::Float64),
                PrimitiveType::Decimal { precision, scale } => {
                    Ok(DataType::Decimal128(*precision as u8, *scale as i8))
                }
                PrimitiveType::Date => Ok(DataType::Date64),
                PrimitiveType::Time => Ok(DataType::Time64(TimeUnit::Millisecond)),
                PrimitiveType::Timestamp => Ok(DataType::Timestamp(TimeUnit::Millisecond, None)),
                PrimitiveType::Timestampz => Ok(DataType::Timestamp(
                    TimeUnit::Millisecond,
                    Some(Arc::from("UTC")),
                )),
                PrimitiveType::String => Ok(DataType::Utf8),
                PrimitiveType::Uuid => Ok(DataType::Utf8),
                PrimitiveType::Fixed(len) => Ok(DataType::FixedSizeBinary(*len as i32)),
                PrimitiveType::Binary => Ok(DataType::Binary),
            },
            Type::List(list) => Ok(DataType::List(Arc::new(Field::new_dict(
                "",
                (&list.element as &Type).try_into()?,
                !list.element_required,
                list.element_id as i64,
                false,
            )))),
            Type::Struct(struc) => Ok(DataType::Struct(
                struc
                    .fields
                    .iter()
                    .map(|field| {
                        Ok(Field::new_dict(
                            &field.name,
                            (&field.field_type).try_into()?,
                            !field.required,
                            field.id as i64,
                            false,
                        ))
                    })
                    .collect::<Result<_, anyhow::Error>>()?,
            )),
            Type::Map(map) => Ok(DataType::Map(
                Arc::new(Field::new_dict(
                    "entries",
                    DataType::Struct(Fields::from(vec![
                        Field::new_dict(
                            "key",
                            (&map.key as &Type).try_into()?,
                            false,
                            map.key_id as i64,
                            false,
                        ),
                        Field::new_dict(
                            "value",
                            (&map.value as &Type).try_into()?,
                            !map.value_required,
                            map.value_id as i64,
                            false,
                        ),
                    ])),
                    false,
                    0,
                    false,
                )),
                false,
            )),
        }
    }
}

impl TryFrom<&DataType> for Type {
    type Error = anyhow::Error;

    fn try_from(value: &DataType) -> Result<Self, Self::Error> {
        match value {
            DataType::Boolean => Ok(Type::Primitive(PrimitiveType::Boolean)),
            DataType::Int32 => Ok(Type::Primitive(PrimitiveType::Int)),
            DataType::Int64 => Ok(Type::Primitive(PrimitiveType::Long)),
            DataType::Float32 => Ok(Type::Primitive(PrimitiveType::Float)),
            DataType::Float64 => Ok(Type::Primitive(PrimitiveType::Double)),
            DataType::Decimal128(precision, scale) => Ok(Type::Primitive(PrimitiveType::Decimal {
                precision: *precision as u32,
                scale: *scale as u32,
            })),
            DataType::Date64 => Ok(Type::Primitive(PrimitiveType::Date)),
            DataType::Time64(_) => Ok(Type::Primitive(PrimitiveType::Time)),
            DataType::Timestamp(_, _) => Ok(Type::Primitive(PrimitiveType::Timestamp)),
            DataType::Utf8 => Ok(Type::Primitive(PrimitiveType::String)),
            DataType::FixedSizeBinary(len) => {
                Ok(Type::Primitive(PrimitiveType::Fixed(*len as u64)))
            }
            DataType::Binary => Ok(Type::Primitive(PrimitiveType::Binary)),
            _ => Err(anyhow!("Other arrow datatypes not supported")),
        }
    }
}
