//! APIs exposing `parquet2`'s statistics as arrow's statistics.
use crate::datatypes::DataType;
use crate::error::ArrowError;
use parquet2::schema::types::PhysicalType;
use parquet2::statistics::PrimitiveStatistics as ParquetPrimitiveStatistics;
use parquet2::statistics::Statistics as ParquetStatistics;
use std::any::Any;

use crate::error::Result;

mod primitive;
pub use primitive::*;
mod binary;
pub use binary::*;
mod boolean;
pub use boolean::*;
mod fixlen;
pub use fixlen::*;

/// Trait representing a deserialized parquet statistics into arrow.
pub trait Statistics: std::fmt::Debug {
    /// returns the [`DataType`] of the statistics.
    fn data_type(&self) -> &DataType;

    /// Returns `dyn Any` can used to downcast to a physical type.
    fn as_any(&self) -> &dyn Any;

    /// Return the null count statistic
    fn null_count(&self) -> Option<i64>;
}

impl PartialEq for &dyn Statistics {
    fn eq(&self, other: &Self) -> bool {
        self.data_type() == other.data_type()
    }
}

impl PartialEq for Box<dyn Statistics> {
    fn eq(&self, other: &Self) -> bool {
        self.data_type() == other.data_type()
    }
}

pub fn deserialize_statistics(stats: &dyn ParquetStatistics) -> Result<Box<dyn Statistics>> {
    match stats.physical_type() {
        PhysicalType::Int32 => {
            let stats = stats.as_any().downcast_ref().unwrap();
            primitive::statistics_from_i32(stats, stats.descriptor.type_())
        }
        PhysicalType::Int64 => {
            let stats = stats.as_any().downcast_ref().unwrap();
            primitive::statistics_from_i64(stats, stats.descriptor.type_())
        }
        PhysicalType::ByteArray => {
            let stats = stats.as_any().downcast_ref().unwrap();
            binary::statistics_from_byte_array(stats, stats.descriptor.type_())
        }
        PhysicalType::Boolean => {
            let stats = stats.as_any().downcast_ref().unwrap();
            Ok(Box::new(BooleanStatistics::from(stats)))
        }
        PhysicalType::Float => {
            let stats = stats
                .as_any()
                .downcast_ref::<ParquetPrimitiveStatistics<f32>>()
                .unwrap();
            Ok(Box::new(PrimitiveStatistics::<f32>::from((
                stats,
                DataType::Float32,
            ))))
        }
        PhysicalType::Double => {
            let stats = stats
                .as_any()
                .downcast_ref::<ParquetPrimitiveStatistics<f64>>()
                .unwrap();
            Ok(Box::new(PrimitiveStatistics::<f64>::from((
                stats,
                DataType::Float64,
            ))))
        }
        PhysicalType::FixedLenByteArray(_) => {
            let stats = stats.as_any().downcast_ref().unwrap();
            fixlen::statistics_from_fix_len(stats, stats.descriptor.type_())
        }
        _ => Err(ArrowError::NotYetImplemented(
            "Reading Fixed-len array statistics is not yet supported".to_string(),
        )),
    }
}
