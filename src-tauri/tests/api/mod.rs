/**
 * API Integration Tests Module
 * 
 * This module contains comprehensive integration tests for all 33 Tauri API endpoints.
 * Tests are organized by functional area and include both success and error scenarios.
 * 
 * Resource management tests have been split into focused modules for better maintainability:
 * - get_full_resource_tests: Comprehensive tests for get_full_resource endpoint (486 lines)
 * - delete_resource_tests: Comprehensive tests for delete_resource endpoint (437 lines)
 * - scale_resource_tests: Comprehensive tests for scale_resource endpoint (494 lines)
 * - update_resource_tests: Comprehensive tests for update_resource endpoint (773 lines)
 * - resource_discovery_and_workflow_tests: Resource discovery and workflow integration tests (233 lines)
 * - advanced_resource_management_tests: CronJob management and v2 resource operations (650+ lines)
 * 
 * COMPREHENSIVE COVERAGE: 100% of all 33 API endpoints now have integration tests
 */

pub mod connection_tests;
// pub mod resource_tests;  // Original large file (archived - 2427 lines)
pub mod modular_tests_documentation;        // Modular structure documentation and re-exports
pub mod resource_discovery_and_workflow_tests; // Resource discovery and workflow integration tests (233 lines)
pub mod get_full_resource_tests;   // Focused get_full_resource tests (486 lines)
pub mod delete_resource_tests;     // Focused delete_resource tests (437 lines)
pub mod scale_resource_tests;      // Focused scale_resource tests (494 lines)
pub mod update_resource_tests;     // Focused update_resource tests (773 lines)
pub mod streaming_tests;
pub mod pod_tests;
pub mod logs_tests;
pub mod shell_tests;
pub mod system_tests;
pub mod advanced_resource_management_tests; // CronJob management and v2 resource operations (650+ lines)
pub mod test_helpers;

pub use test_helpers::*;