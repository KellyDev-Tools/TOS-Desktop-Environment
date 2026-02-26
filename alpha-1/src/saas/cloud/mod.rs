//! Cloud Integration Submodule
//!
//! Submodule for different cloud provider integrations (AWS, GCP, Azure).

pub mod aws;

pub use aws::{AwsManager, AwsConfig};
