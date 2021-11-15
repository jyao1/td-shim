# Copyright (c) 2020 Intel Corporation
# SPDX-License-Identifier: BSD-2-Clause-Patent

.section .text
.equ USE_TDX_EMULATION, 1
.equ number_of_regs_pushed, 8
.equ number_of_parameters,  4

.equ first_variable_on_stack_offset,   (number_of_regs_pushed * 8) + (number_of_parameters * 8) + 8
.equ second_variable_on_stack_offset,  first_variable_on_stack_offset + 8

#  TdCall (
#    UINT64  Leaf,
#    UINT64  P1,
#    UINT64  P2,
#    UINT64  P3,
#    UINT64  Results,
#    )
.global td_call
td_call:
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

       movq %rcx, %rax
       movq %rdx, %rcx
       movq %r8, %rdx
       movq %r9, %r8

       # tdcall
       .if USE_TDX_EMULATION != 0
       vmcall
       .else
       .byte 0x66,0x0f,0x01,0xcc
       .endif

       # exit if tdcall reports failure.
       testq %rax, %rax
       jnz exit

       # test if caller wanted results
       movq  first_variable_on_stack_offset(%rbp), %r12
       testq %r12, %r12
       jz exit
       movq %rcx, 0(%r12)
       movq %rdx, 8(%r12)
       movq %r8,  16(%r12)
       movq %r9,  24(%r12)
       movq %r10, 32(%r12)
       movq %r11, 408(%r12)
exit:
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
