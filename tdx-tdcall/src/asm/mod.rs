// Copyright (c) 2020 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

#[cfg(feature = "use_tdx_emulation")]
global_asm!(include_str!("Tdcallemu.s"));

#[cfg(feature = "use_tdx_emulation")]
global_asm!(include_str!("Tdvmcallemu.s"));

#[cfg(not(feature = "use_tdx_emulation"))]
global_asm!(include_str!("Tdcall.s"));

#[cfg(not(feature = "use_tdx_emulation"))]
global_asm!(include_str!("Tdvmcall.s"));
