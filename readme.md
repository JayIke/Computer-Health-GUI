# Computer Health Monitor (Windows)

A desktop-based monitoring tool that provides real-time system health data, including CPU speed, memory usage, disk space, uptime, and network interface details. Built with Rust for the backend and HTML/Tera for the frontend, this project showcases low-level Windows API integration, system diagnostics, and web templating in a desktop environment. My motivation for this project was to gain familiarity with Windows API, the long term goal is to create a distributable windows application (using Windows UI).

---

## Project Summary

The **Computer Health Monitor** is a lightweight Windows utility that displays basic hardware and system performance metrics in a browser-based interface.Leverages Windows API calls to deliver accurate system insights without third-party dependencies or bloat.

Key features:
- System uptime
- CPU clock speed
- Memory usage statistics
- Disk space usage
- Enumerated network interface list

---

## Tools & Libraries Used

### 🦀 Rust Crates
- [`windows`](https://crates.io/crates/windows) — access to Windows APIs (e.g., `GetIfTable`, `GlobalMemoryStatusEx`)
- [`actix-web`](https://crates.io/crates/actix-web) — asynchronous web server framework
- [`tera`](https://crates.io/crates/tera) — HTML templating engine (Jinja2-style)
- [`serde`](https://crates.io/crates/serde) — data serialization/deserialization
- [`serde_json`](https://crates.io/crates/serde_json) — JSON conversion (if needed for API endpoints)
- [`futures`](https://crates.io/crates/futures) — async support for request handling

### Frontend
- HTML-based local webpage (rendered using Tera)

---

## Client–Server Data Flow

```plaintext
Browser (HTML interface)
        │
        ▼
 GET request to `/` (root route)
        │
        ▼
 Actix-web server (Rust)
 ├── Gathers system data:
 │   ├─ CPU speed
 │   ├─ Memory info
 │   ├─ Disk info
 │   └─ Network interfaces via Win32 APIs
 └── Renders HTML using Tera templates
        │
        ▼
 Renders system info in browser UI
```

## How to build and run application
``` sh
cd ./windows_monitor
cargo build
cargo run
```

``` sh
# rebuild after changes
cargo clean
cargo build
```

