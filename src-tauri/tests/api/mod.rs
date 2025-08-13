/**
 * API Integration Tests Module
 * 
 * This module contains comprehensive integration tests for all 26 Tauri API endpoints.
 * Tests are organized by functional area and include both success and error scenarios.
 */

pub mod connection_tests;
pub mod resource_tests;
pub mod streaming_tests;
pub mod pod_tests;
pub mod logs_tests;
pub mod shell_tests;
pub mod system_tests;
pub mod test_helpers;

pub use test_helpers::*;