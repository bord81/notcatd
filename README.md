# notcatd — NotCat Daemon

`notcatd` is a native Android daemon written in **Rust**, designed as a lightweight and extensible alternative to the traditional `logcat` facility. It provides structured logging over a Unix socket using an epoll-based server and supports multiple output log sinks such as file storage and `logcat` redirection.

---

## 🔧 Features

- 📡 **Unix Socket Logging** — Accepts log messages via a SEQPACKET Unix domain socket.
- 🧵 **Asynchronous Runtime** — Built using [Tokio](https://tokio.rs/) for efficient async IO and internal task management.
- 📁 **Multiple Output Sinks**:
  - Forwarding logs to traditional `logcat`.
  - Persisting logs to rotating file sequence under `/data/vendor/notcat/`.
- 🔐 **SEPolicy Ready** — Secure integration with Android SELinux policies.
- 🧩 **Modular Design** — Easily extendable to support more sinks or message formats.

---

## 🔗 Client Library

To send logs to `notcatd`, use the companion library [`notcat_lib`](https://github.com/bord81/notcat_lib), which supports Rust, C, and Kotlin (via JNI).

---

## 🛡️ License

This project is licensed under the [MIT License](LICENSE).

© 2025 Borys Zakaliuk

---
