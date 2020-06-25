//! # prima_bridge
//!
//! This module is responsible of issuing external request.
//! It is supposed to give the basics building blocks for building bridges to the external world
//! while abstracting the low level stuffs like adding our custom headers and request tracing.
//!
//! Right now it supports Rest and GraphQL requests.
//!
//! You should start by creating a [Bridge](struct.Bridge.html) instance.
//! This instance should live for all the application lifetime.
//!
//! **Do not create a new bridge on every request!**
//!
//! You should use something like lazy_static, or some sort of inversion of control container to
//! pass around.
//!
//! The bridge implement a type state pattern to build the external request. Follow the types!

mod errors;
pub mod prelude;

#[cfg(feature = "blocking")]
mod blocking;
