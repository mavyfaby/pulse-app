# pulse-server

The backend server for Pulse — Bayanihan Emergency Network. Built with Rust and Tokio.

## Requirements

- [Rust](https://rustup.rs) (edition 2024)
- [just](https://github.com/casey/just)

## Configuration

The server reads configuration from an env file. Set `PULSE_ENV_FILE` to the path of your env file relative to the working directory:

```
PULSE_ENV_FILE=.env
```

The env file must define the following variables:

| Variable | Description |
|---|---|
| `PULSE_TCP_HOST` | Host address the TCP server binds to |
| `PULSE_TCP_PORT` | Port the TCP server listens on (0–65535) |

Example `.env`:

```env
PULSE_TCP_HOST=0.0.0.0
PULSE_TCP_PORT=9000
```

## Running

```bash
just server-run
```

## Testing

```bash
just server-test
```

## License

AGPL-3.0-or-later — see [LICENSE](../LICENSE) for details.