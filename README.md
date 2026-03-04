# Shipfe

[![npm version](https://img.shields.io/npm/v/shipfe.svg)](https://www.npmjs.com/package/shipfe)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![GitHub](https://img.shields.io/badge/GitHub-Master--Jian/shipfe--rust-blue.svg)](https://github.com/Master-Jian/shipfe-rust)

A powerful, **free**, **Rust-based** web application deployment tool that **does not request network**, enabling **one-click upload of frontend static deployment packages to servers**. Supports multi-environment and sub-environment deployment with zero-downtime atomic deployment functionality.

## Documentation

📖 [Full Documentation](https://master-jian.github.io/shipfe-rust/)

## Installation

```bash
npm install -g shipfe
```

## Quick Start

1. Initialize the project:
```bash
shipfe init
```

2. Configure deployment in `shipfe.config.json`

3. Deploy:
```bash
shipfe deploy --profile prod
```

## Common Commands

- `shipfe deploy --profile <env>` - Deploy to specified environment
- `shipfe deploy --atomic` - Atomic deployment
- `shipfe rollback --profile <env> --to <timestamp>` - Rollback to specified version

## License

MIT