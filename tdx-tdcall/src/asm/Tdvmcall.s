# Copyright (c) 2020 Intel Corporation
# SPDX-License-Identifier: BSD-2-Clause-Patent

.section .text

.equ USE_TDX_EMULATION, 0
.equ TDVMCALL_EXPOSE_REGS_MASK,       0xffec
.equ TDVMCALL,                        0x0
.equ EXIT_REASON_CPUID,               0xa

.equ number_of_regs_pushed, 8
.equ number_of_parameters,  4

.equ first_variable_on_stack_offset,   (number_of_regs_pushed * 8) + (number_of_parameters * 8) + 8
.equ second_variable_on_stack_offset,  first_variable_on_stack_offset + 8

#  UINT64
#  TdVmCall (
#    UINT64  Leaf,
#    UINT64  P1,
#    UINT64  P2,
#    UINT64  P3,
#    UINT64  P4,
#    UINT64  *Val
#    )
.global td_vm_call
td_vm_call:
        # tdcall_push_regs
        pushq %rbp
        movq %rsp, %rbp
        pushq %r15
        pushq %r14
        pushq %r13
        pushq %r12
        pushq %rbx
        pushq %rsi
        pushq %rdi

       movq %rcx, %r11
       movq %rdx, %r12
       movq %r8, %r13
       movq %r9, %r14
       movq first_variable_on_stack_offset(%rsp), %r15

       #tdcall_regs_preamble TDVMCALL, TDVMCALL_EXPOSE_REGS_MASK
        movq $TDVMCALL, %rax

        movl $TDVMCALL_EXPOSE_REGS_MASK, %ecx

        # R10 = 0 (standard TDVMCALL)

        xorl %r10d, %r10d

        # Zero out unused (for standard TDVMCALL) registers to avoid leaking
        # secrets to the VMM.

        xorl %ebx, %ebx
        xorl %esi, %esi
        xorl %edi, %edi

        xorl %edx, %edx
        xorl %ebp, %ebp
        xorl %r8d, %r8d
        xorl %r9d, %r9d

       # tdcall
       .if USE_TDX_EMULATION != 0
       vmcall
       .else
       .byte 0x66,0x0f,0x01,0xcc
       .endif

       # ignore return dataif TDCALL reports failure.
       testq %rax, %rax
       jnz no_return_data

       # Propagate TDVMCALL success/failure to return value.
       movq %r10, %rax

       # Retrieve the Val pointer.
       movq second_variable_on_stack_offset(%rsp), %r9
       testq %r9, %r9
       jz no_return_data

       # On success, propagate TDVMCALL output value to output param
       testq %rax, %rax
       jnz no_return_data
       mov %r11, (%r9)

no_return_data:
        #tdcall_regs_postamble
        xorl %ebx, %ebx
        xorl %esi, %esi
        xorl %edi, %edi

        xorl %ecx, %ecx
        xorl %edx, %edx
        xorl %r8d,  %r8d
        xorl %r9d,  %r9d
        xorl %r10d, %r10d
        xorl %r11d, %r11d

        # tdcall_pop_regs
        popq %rdi
        popq %rsi
        popq %rbx
        popq %r12
        popq %r13
        popq %r14
        popq %r15
        popq %rbp

       ret
