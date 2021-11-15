# Copyright (c) 2021 Intel Corporation
# SPDX-License-Identifier: BSD-2-Clause-Patent

.section .text

#  cet_ss_test(
#       loop: usize,  // rcx
#       );
.global cet_ss_test
cet_ss_test:
        movq %rsp, %rdx
rcx_test:
        cmp $1000, %rcx
        jnz rcx_test
write_stack:
        movb $100, (%rdx)
        addq $1, %rdx
        decq %rcx
        jnz write_stack

        ret