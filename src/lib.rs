/// API Models
pub mod model;

/// Wrapper implemented with custom type that implements the [`futures::stream::Stream`] trait
pub mod using_custom_stream;

/// Wrapper implemented using [`futures::stream::StreamExt`]
pub mod using_stream_ext;
