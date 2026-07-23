# Crates Overview

Typhoon is organized into several crates, each providing specific functionality for building Solana programs.

## Core Crates

- **`typhoon`**: The main entry point, re-exporting modules from other crates for ease of use.
- **`typhoon-accounts`**: Handles account-related abstractions and data structures.
- **`typhoon-context`**: Provides the infrastructure for instruction contexts.
- **`typhoon-errors`**: Defines the error handling system used throughout the framework.
- **`typhoon-events`**: Emitting events to the transaction logs (the `Emit` trait behind `#[event]`).
- **`typhoon-traits`**: Core traits used for cross-compatible implementations.

## Macro Crates

- **`typhoon-context-macro`**: Implements the `#[context]` attribute macro.
- **`typhoon-account-macro`**: Macros for working with account states and data.
- **`typhoon-program-id-macro`**: Implements the `program_id!` macro.

## Utility Crates

- **`typhoon-utility`**: General-purpose utilities, including byte manipulation.
- **`typhoon-utility-traits`**: Common utility traits for efficient data handling.
- **`typhoon-discriminator`**: Logic for generating instruction and account discriminators.
