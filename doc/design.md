# rust-td design

The rust-td is one design choice for a TD.

A `td-shim` takes over the reset vector and prepares the TD environment setup. It could be considered as a lightweigth TD Virtual Firmware (TDVF). The td-shim will transfer control to a td-payload.

A `td-payload` is a new execution environment. It could be a bare-metal environment, a UEFI virtual firmware, an OS loader, a OS kernel, etc.

Below figure shows the high level design.

   ```
                     +----------------------+
                     |       td-payload     |
                     |                      |
                     +----------------------+
                                 ^
                                 | <-------------------- td-shim spec
                     +----------------------+
                     |        td-shim       |
                     |           ^          |
                     |           |          |
                     |     (reset vector)   |
      Guest          +----------------------+
      ============================================ <--- td-shim spec
      Host           +----------------------+
                     |         VMM          |
                     +----------------------+

   ```

 The [td-shim specification](https://github.com/jyao1/td-shim/blob/init_version/doc/tdshim_spec.md) defines the interface between td-shim and vmm, and the interface between td-shim and td-payload.

 The [td-shim threat model](https://github.com/jyao1/td-shim/blob/init_version/doc/threat_model.md) defines the threat model for the td-shim.

This repo includes a full `td-shim`, and sample `td-payload`. The consumer may create other td-payload. For example, to support [TDX](https://www.intel.com/content/www/us/en/developer/articles/technical/intel-trust-domain-extensions.html) [migration TD](https://www.intel.com/content/dam/develop/external/us/en/documents/tdx-migration-td-design-guide-348987-001.pdf), a rust-migtd can include a td-shim and a migtd-payload.

## td-shim

[rust-tdshim](https://github.com/jyao1/td-shim/tree/init_version/rust-tdshim) is a core of td-shim. The entrypoint is `_start()` at [main](https://github.com/jyao1/td-shim/blob/init_version/rust-tdshim/src/main.rs). It will initialize the td-shim and switch to td-payload at `switch_stack_call()` of [main](https://github.com/jyao1/td-shim/blob/init_version/rust-tdshim/src/main.rs).

The TD_HOB is measured and event log is created at `create_td_event()` in [tcg.rs](https://github.com/jyao1/td-shim/blob/init_version/rust-tdshim/src/tcg.rs).

Data Execution Prevention (DEP) is setup at `find_and_report_entry_point()` in [ipl.rs](https://github.com/jyao1/td-shim/blob/init_version/rust-tdshim/src/ipl.rs). The primitive `set_nx_bit()` and `set_write_protect()` are provided by [memory.rs](https://github.com/jyao1/td-shim/blob/init_version/rust-tdshim/src/memory.rs).

Control flow Enforcement Technology (CET) Shadow Stack is setup at `enable_cet_ss()` in [cet_ss.rs](https://github.com/jyao1/td-shim/blob/init_version/rust-tdshim/src/cet_ss.rs).

Stack guard is setup at `stack_guard_enable()` in [stack_guard.rs](https://github.com/jyao1/td-shim/blob/init_version/rust-tdshim/src/stack_guard.rs).

### reset vector

[ResetVector](https://github.com/jyao1/td-shim/tree/init_version/rust-tdshim/ResetVector) is the reset vector inside of the td-shim. It owns the first instruction in TD at address 0xFFFFFFF0 - [resetVector](https://github.com/jyao1/td-shim/blob/init_version/rust-tdshim/ResetVector/Ia32/ResetVectorVtf0.asm). Then it switches to long mode, parks APs, initializes the stack, copies the td-shim core to low memory (1MB) and call to rust-tdshim `call    rsi` at [main](https://github.com/jyao1/td-shim/blob/init_version/rust-tdshim/ResetVector/Main.asm)

### TDX related lib

[tdx-exception](https://github.com/jyao1/td-shim/tree/init_version/tdx-exception) provides execution handler in TD.

[tdx-logger](https://github.com/jyao1/td-shim/tree/init_version/tdx-logger) provides debug logger in TD.

[tdx-tdcall](https://github.com/jyao1/td-shim/tree/init_version/tdx-logger) provides TDCALL function.

### Generic lib

[elf-loader](https://github.com/jyao1/td-shim/tree/init_version/elf-loader) is an ELF image loader.

[pe-loader](https://github.com/jyao1/td-shim/tree/init_version/pe-loader) is an PE image loader.

[fw-pci](https://github.com/jyao1/td-shim/tree/init_version/fw-pci) provides the access to PCI space.

[fw-virtio](https://github.com/jyao1/td-shim/tree/init_version/fw-virtio) provides virtio interface.

[fw-vsock](https://github.com/jyao1/td-shim/tree/init_version/fw-vsock) provides vsock interface.

[r-uefi-pi](https://github.com/jyao1/td-shim/tree/init_version/r-uefi-pi) defines uefi-pi data structure.

[uefi-pi](https://github.com/jyao1/td-shim/tree/init_version/uefi-pi) provide uefi-pi structure access function.

[rust-paging](https://github.com/jyao1/td-shim/tree/init_version/rust-paging) provides function to manage the page table.

### External dependency

[ring](https://github.com/jyao1/ring/tree/uefi_support) is crypto function. The SHA384 function is used to calculate measurement.

## build

### tools

[rust-td-tool](https://github.com/jyao1/td-shim/tree/init_version/rust-td-tool) is the tool to assembly all components into a TD.bin.

### layout

[rust-td-layout](https://github.com/jyao1/td-shim/tree/init_version/rust-td-layout) defines the layout of a TD.

## sample td-payload

[rust-td-payload](https://github.com/jyao1/td-shim/tree/init_version/rust-td-payload) is a sample payload. It supports benchmark collection, json parsing.

## test tools

[benchmark](https://github.com/jyao1/td-shim/tree/init_version/benchmark) is to help collect benchmark information, such as stack usage, heap usage, execution time.

[fuzzing-test](https://github.com/jyao1/td-shim/tree/init_version/fuzzing) includes sample fuzzing test. Refer to [fuzzing](https://github.com/jyao1/td-shim/blob/init_version/doc/fuzzing.md) doc for more detail.

[test-coverage](https://github.com/jyao1/td-shim/blob/init_version/doc/unit_test_coverage.md) describes how to collect the coverage data.

[rudra](https://github.com/jyao1/td-shim/blob/init_version/doc/rudra.md) describes how to scan the vulnerable rust code by using [rudra](https://github.com/sslab-gatech/Rudra).

[cargo-deny](https://github.com/jyao1/td-shim/blob/init_version/.github/workflows/deny.yml) is used to scan the vulnerable rust crate dependency according to [rustsec](https://rustsec.org/).

[no_std_test](https://github.com/jyao1/td-shim/tree/init_version/no_std_test) is used to run test for no_std code.

[test_lib](https://github.com/jyao1/td-shim/tree/init_version/test_lib) is to provide support function for unit test.
