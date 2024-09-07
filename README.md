# RapidMQ

RapidMQ is a high-performance, AI-enhanced message queue system.

## Quick Start

1. Set up the development environment:
   ```
   ./scripts/setup_dev_environment.sh
   ```

2. Build the project:
   ```
   cargo build
   ```

3. Run the tests:
   ```
   make test
   ```

4. Run the benchmarks:
   ```
   make benchmark
   ```

5. Start the server:
   ```
   cargo run --bin rapidmq -- --config config/rapidmq_dev.yaml
   ```

## Development

- Use `cargo watch -x run` for automatic recompilation during development.
- Run `make check` before committing to ensure code quality.
- See the `Makefile` for other useful development commands.

## Contributing

Please read [CONTRIBUTING.md](CONTRIBUTING.md) for details on our code of conduct and the process for submitting pull requests.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.