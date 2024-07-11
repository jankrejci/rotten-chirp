# PSU sniffer

## Basic template
You can create basic template with
[esp-template](https://github.com/esp-rs/esp-template)
`cargo generate esp-rs/esp-template` 

Logging infrastructure can be added based on `defmt` example from
[no_std-training](https://github.com/esp-rs/no_std-training)

You need to remove build.rs file from template.

To run example use
`DEFMT_LOG=debug cargo run --release`





