// Copyright (c) 2021 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

global_asm!(include_str!("stack_guard_test.s"));
#[cfg(feature = "cet-ss")]
global_asm!(include_str!("cet_ss_test.s"));
