// Copyright (c) 2020 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

global_asm!(include_str!("switch_stack.s"), options(att_syntax));
global_asm!(include_str!("msr64.s"), options(att_syntax));
