# Solana Leader Schedule CLI Tool

A command-line tool to fetch and display Solana validator leader schedule information. This tool shows when a validator is scheduled to be a leader in a specific epoch with accurate timestamps in your local timezone.

## Features

- 🔍 **Validator Lookup**: Find leader slots for any Solana validator by public key
- 🌐 **Multi-Cluster Support**: Works with mainnet, testnet, and devnet
- 📅 **Epoch Selection**: Query current or specific epoch schedules
- 🕒 **Local Timestamps**: Displays slot times in your local timezone
- ⏰ **Time Difference**: Shows how long ago past slots occurred or when future slots will happen
- 📊 **Beautiful Tables**: Clean, formatted output with proper alignment
- ⚡ **Fast & Reliable**: Built with Rust for performance and safety

## Installation

### Prerequisites

- [Rust](https://rustup.rs/) (1.70.0 or later)
- Internet connection to access Solana RPC endpoints

### From Source

1. Clone this repository:
   ```bash
   git clone https://github.com/your-username/leader_schedule.git
   cd leader_schedule
   ```

2. Build and install:
   ```bash
   cargo build --release
   ```

3. Run the tool:
   ```bash
   cargo run -- [OPTIONS]
   ```

   Or install globally:
   ```bash
   cargo install --path .
   leader_schedule [OPTIONS]
   ```

## Usage

### Basic Usage

Get leader schedule for the default validator on mainnet:
```bash
cargo run
```

### Command Line Options

```bash
leader_schedule [OPTIONS]

Options:
  -i, --identity <IDENTITY>  Validator identity public key [default: 99PWsEpnfFaBMbW8epmC1pnRp1HrsFxASniofXNxDaQQ]
  -c, --cluster <CLUSTER>    Solana cluster (mainnet, testnet, devnet) [default: mainnet]
  -e, --epoch <EPOCH>        Epoch number (if not specified, uses current epoch)
  -h, --help                 Print help
  -V, --version              Print version
```

### Examples

1. **Check current epoch for a specific validator:**
   ```bash
   cargo run -- --identity 7Np41oeYqPefeNQEHSv1UDhYrehxin3NStELsSKCT4K2
   ```

2. **Check testnet validator:**
   ```bash
   cargo run -- --cluster testnet --identity 5D1fNXzvv5NjV1ysLjirC4WY92RNsVH18vjmcszZd8on
   ```

3. **Check specific epoch:**
   ```bash
   cargo run -- --epoch 815 --identity 7Np41oeYqPefeNQEHSv1UDhYrehxin3NStELsSKCT4K2
   ```

4. **Devnet with specific epoch:**
   ```bash
   cargo run -- --cluster devnet --epoch 100
   ```

### Sample Output

```
🔍 Fetching leader schedule for validator: 99PWsEpnfFaBMbW8epmC1pnRp1HrsFxASniofXNxDaQQ
🌐 Cluster: mainnet
🕒 Using current slot 353410757 as time reference
📅 Target epoch: 818
🎯 Epoch 818 starts at absolute slot: 353376000
✅ Found 64 leader slots for validator 99PWsEpnfFaBMbW8epmC1pnRp1HrsFxASniofXNxDaQQ in epoch 818

+-------+-----------+----------------------------+-------------+
| Epoch | Slot      | Time (Local)               | Time Diff (10:47:43)   |
+-------+-----------+----------------------------+-------------+
| 818   | 353403088 | 2025-07-15 09:49:46 +04:00 | 57m 56s ago |
| 818   | 353424968 | 2025-07-15 12:15:37 +04:00 | 1h 27m      |
| 818   | 353438656 | 2025-07-15 13:46:52 +04:00 | 2h 59m      |
| 818   | 353642376 | 2025-07-16 12:25:00 +04:00 | 1d 1h 37m   |
+-------+-----------+----------------------------+-------------+

📊 Total leader slots: 64
```

## Technical Details

### Timestamp Accuracy

The tool dynamically fetches the current slot and timestamp from the Solana network to use as a real-time reference point for timestamp calculations. This ensures maximum accuracy by:

- **Live Network Sync**: Uses current network state instead of hardcoded values
- **Self-Updating**: Automatically stays accurate without manual updates
- **Minimal Latency**: Real-time slot/time relationship eliminates drift over time
- **Network-Specific**: Adapts to different clusters' timing characteristics

### Time Difference Display

The **Time Diff** column provides intuitive relative timing information with reference time context:

- **Past Slots**: Shows elapsed time since the slot occurred (e.g., "2h 15m ago", "1d 3h ago")
- **Future Slots**: Shows time remaining until the slot (e.g., "45m", "2h 30m", "1d 5h")
- **Reference Time**: Column header includes the exact time when calculations were made (e.g., "Time Diff (10:47:43)")
- **Smart Formatting**: Automatically adjusts precision based on time scale (seconds for short periods, omitted for longer periods)
- **Real-Time Context**: Helps validators understand their schedule relative to current time

### Slot Calculations

- **Absolute Slots**: Shows actual network slot numbers, not relative positions within epochs
- **Epoch Boundaries**: Correctly calculates epoch start slots, including warmup periods
- **Current vs Historical**: Handles both current epoch data and historical epoch queries

### Supported Clusters

| Cluster | RPC Endpoint                          |
|---------|---------------------------------------|
| Mainnet | `https://api.mainnet-beta.solana.com` |
| Testnet | `https://api.testnet.solana.com`      |
| Devnet  | `https://api.devnet.solana.com`       |

### Dependencies

- **solana-client**: 2.3.1 - Latest Solana RPC client
- **solana-sdk**: 2.3.1 - Core Solana types and utilities
- **clap**: 4.5 - Command-line argument parsing
- **tabled**: 0.16 - Beautiful table formatting
- **chrono**: 0.4 - Date and time handling with timezone support
- **tokio**: 1.0 - Async runtime
- **anyhow**: 1.0 - Error handling

## Development

### Running Tests

```bash
cargo test
```

### Code Formatting

```bash
cargo fmt
```

### Linting

```bash
cargo clippy
```

### Building for Release

```bash
cargo build --release
```

## Error Handling

The tool provides clear error messages for common issues:

- **Invalid validator identity**: Checks public key format
- **Network connectivity**: Handles RPC connection failures
- **Invalid cluster names**: Validates cluster parameters
- **Missing data**: Gracefully handles epochs without leader schedules

## Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature-name`
3. Make your changes and add tests
4. Ensure code passes: `cargo test && cargo clippy`
5. Format code: `cargo fmt`
6. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- Built using the [Solana SDK](https://github.com/anza-xyz/agave)
- Inspired by the official Solana CLI tools
- Thanks to the Solana community for documentation and support

## Support

For questions, issues, or feature requests, please open an issue on GitHub.
