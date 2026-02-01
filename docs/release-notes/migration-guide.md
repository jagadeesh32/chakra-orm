---
title: Migration Guide
description: Upgrade between Chakra ORM versions
---

# Migration Guide

Instructions for upgrading between Chakra ORM versions.

---

## Upgrading to v0.1.0

This is the initial release — no migration required!

### Fresh Installation

=== "Python"

    ```bash
    pip install chakra-orm
    chakra init
    ```

=== "Rust"

    ```toml
    [dependencies]
    chakra = "0.1"
    ```

---

## General Upgrade Process

When upgrading to any new version:

### 1. Read the Changelog

Check the [Changelog](changelog.md) for:

- Breaking changes
- Deprecated features
- New features you might want to use

### 2. Update Dependencies

=== "Python"

    ```bash
    pip install --upgrade chakra-orm
    ```

=== "Rust"

    ```bash
    cargo update -p chakra
    ```

### 3. Run Migrations

After upgrading, check for new migrations:

```bash
chakra migrate status
chakra migrate apply
```

### 4. Update Your Code

If there are breaking changes, update your code accordingly.

### 5. Test

Run your test suite to ensure everything works.

---

## Breaking Changes Log

### v0.1.0

No breaking changes — initial release.

---

## Deprecation Policy

- Features are deprecated in one minor version before removal
- Deprecated features show warnings when used
- Deprecated features are removed in the next major version

### Currently Deprecated

None.

---

## Getting Help

If you encounter issues during upgrade:

1. Check the [Changelog](changelog.md) for known issues
2. Search [GitHub Issues](https://github.com/chakra-orm/chakra-orm/issues)
3. Ask on [Discord](https://discord.gg/chakra-orm)
4. Open a new issue with details about your environment and error
