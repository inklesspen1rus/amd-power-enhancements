# Power Profile Actions

Small demon reacting to changes in the current power profile.

## Requirements
Minimum Supported [Rust](https://www.rust-lang.org/) Version ([MSRV](https://rust-lang.github.io/rfcs/2495-min-rust-version.html)) - 1.70

## Build

```sh
cargo build --release
```

## Example

Configure YAML to watch system file `/sys/devices/system/cpu/cpufreq/policy0/energy_performance_preference`.
```yaml
power_profiles:
  default:
    commands:
    - "echo Unknown power profile. Default CPU power limits..."
    - "ryzenadj -a 6000 -b 6000 -c 6000 "

  # 'power-saver' for amd-pstate at file_watch backend
  power:
    commands:
    - "echo Power saving mode. Limit CPU power up to 6W"
    - "ryzenadj -a 6000 -b 6000 -c 6000 "

  # 'balanced' for amd-pstate at file_watch backend
  balance_performance:
    commands:
    - "echo Balancing mode. Limit CPU power up to 8W"
    - "ryzenadj -a 8000 -b 8000 -c 8000 "

  # 'performance' for amd-pstate at file_watch backend
  performance:
    commands:
    - "echo Performance mode. Limit CPU power up to 10W"
    - "ryzenadj -a 10000 -b 10000 -c 10000 "

backend:
  backend: "file_watch"
  file_watch:
    file: /sys/devices/system/cpu/cpufreq/policy0/energy_performance_preference
```

Launch with specified configuration file:
```sh
power-profile-actions -c example.yaml
```

Switch current power profile to see reaction.

## Todo

- [x] Support watching system file
- [ ] Support power-profiles-daemon
- [ ] Provide installation as systemd service

## Motivation
New linux performance scaling driver for modern mobile AMD CPUs [does not limit frequency or TDP](https://www.phoronix.com/review/amd-pstate-epp-ryzen-mobile) (while Windows does it). Although the new driver seems really effective in some tasks, CPU limits are still not affected, which can lead to high power consumption (like at performance mode).
