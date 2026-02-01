---
title: Getting Started
description: Start using Chakra ORM in minutes
---

# Getting Started

Get up and running with Chakra ORM in minutes.

## Prerequisites

- **Python 3.9+** or **Rust 1.70+**
- A supported database (PostgreSQL, MySQL, SQLite, or Oracle)

## Quick Navigation

<div class="grid cards" markdown>

-   **[Installation](installation.md)**

    Install Chakra for Python or Rust

-   **[Quick Start (Python)](quickstart-python.md)**

    Build your first Python app

-   **[Quick Start (Rust)](quickstart-rust.md)**

    Build your first Rust app

-   **[Configuration](configuration.md)**

    Configure database and settings

</div>

## Choose Your Path

=== "Python"

    ```bash
    # Install
    pip install chakra-orm

    # Initialize project
    chakra init

    # Create models, migrate, and go!
    ```

    [:octicons-arrow-right-24: Python Quick Start](quickstart-python.md)

=== "Rust"

    ```bash
    # Add to Cargo.toml
    cargo add chakra

    # Or with specific features
    cargo add chakra --features postgres,derive
    ```

    [:octicons-arrow-right-24: Rust Quick Start](quickstart-rust.md)
