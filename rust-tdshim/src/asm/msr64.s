# Copyright (c) 2021 Intel Corporation
# SPDX-License-Identifier: BSD-2-Clause-Patent

.section .text
#  asm_read_msr64(
#       index: u32, // rcx
#       );
.global asm_read_msr64
asm_read_msr64:

    rdmsr
    shlq $0x20, %rdx 
    orq %rdx, %rax
    ret

#  asm_write_msr64(
#       index: u32, // rcx
#       value: u64, // rdx
#       );
.global asm_write_msr64
asm_write_msr64:

    mov %rdx, %rax
    shr $0x20, %rdx
    wrmsr
    ret
