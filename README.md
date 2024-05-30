A very simple example showing how to use the `reload` function of the Rust [`tracing`](https://docs.rs/tracing/latest/tracing/) crate to change log directives at runtime.

Just to make things a little bit fancier, we set up a gRPC interface and server using [`tonic`](https://docs.rs/tonic/latest/tonic/).

This code has been originally written as a companion project for the talk "_Taming the log storm: a systematic approach to effective logging_", at [GDG DevFest Pisa 2024](https://devfest.gdgpisa.it).