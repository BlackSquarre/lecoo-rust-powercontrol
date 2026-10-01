# Project Layout

```text
src/                         Rust application and shared hardware-control library
scripts/                     Release build and hardware-test scripts
dist/                        Local release build output (not tracked)
docs/testing.md              Automated hardware-test instructions
docs/reference/wmi/          WMI class, method, and parameter reference
docs/reference/ui-design.md  UI design notes
logs/                        Local test results (not tracked)
target/                      Cargo build output (not tracked)
```

The public repository contains the Rust implementation and interface notes. Local audit logs, early project plans, third-party decompiled reference code, and machine-specific WMI exports are kept out of public Git history.
