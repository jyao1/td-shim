// Copyright (c) 2021 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

#![no_std]
#![feature(asm)]
#![allow(unused)]

#[macro_use]
extern crate alloc;

use core::alloc::GlobalAlloc;
use core::alloc::Layout;
use linked_list_allocator::LockedHeap;
use rust_td_layout::RuntimeMemoryLayout;
use scroll::Pread;

#[global_allocator]
pub static ALLOCATOR: MyHeap = MyHeap::empty();

pub struct MyHeap {
    max_heap: usize,
    used_heap: usize,
    inner: LockedHeap,
}

impl MyHeap {
    pub const fn empty() -> Self {
        Self {
            max_heap: 0,
            used_heap: 0,
            inner: LockedHeap::empty(),
        }
    }

    pub fn init(&self, heap_size: usize, heap_start: usize) {
        unsafe {
            self.inner.lock().init(heap_start, heap_size);
        }
    }
}

#[allow(clippy::cast_ref_to_mut)]
unsafe impl GlobalAlloc for MyHeap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let res = self.inner.alloc(layout);
        if !res.is_null() {
            unsafe {
                (*(self as *const MyHeap as *mut MyHeap)).used_heap += layout.size();
            }
            if self.max_heap < self.used_heap {
                unsafe { (*(self as *const MyHeap as *mut MyHeap)).max_heap = self.used_heap };
            }
        }
        res
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.inner.dealloc(ptr, layout);
        if layout.size() != 0 {
            unsafe { (*(self as *const MyHeap as *mut MyHeap)).used_heap -= layout.size() };
        }
    }
}

#[derive(Default)]
pub struct BenchmarkContext {
    name: &'static str,
    start_timestamp: u64,
    end_timestamp: u64,
    max_stack: usize,
    max_heap: usize,
}

impl BenchmarkContext {
    pub fn new(memory_layout: RuntimeMemoryLayout, name: &'static str) -> Self {
        BenchmarkContext {
            name,
            ..Default::default()
        }
    }

    pub fn bench_start(&mut self) {
        let rsp: usize;
        unsafe {
            asm!("mov {}, rsp", out(reg) rsp);
        }
        log::info!("rsp_start: {:x}\n", rsp);
        let stack_buffer = unsafe {
            core::slice::from_raw_parts_mut(
                0x7F002000 as *const u8 as *mut u8,
                rsp - 0x7F002000usize - 0x20usize,
            )
        };

        for x in stack_buffer.iter_mut() {
            *x = 0x5Au8;
        }

        log::info!("bench start ...\n");
        self.start_timestamp = unsafe { x86::time::rdtsc() };
    }

    pub fn bench_end(&mut self) {
        self.end_timestamp = unsafe { x86::time::rdtsc() };
        log::info!("bench end ...\n");
        let rsp: usize;
        unsafe {
            asm!("mov {}, rsp", out(reg) rsp);
        }
        log::info!("rsp_end: {:x}\n", rsp);
        let stack_buffer = unsafe {
            core::slice::from_raw_parts_mut(
                0x7F002000 as *const u8 as *mut u8,
                rsp - 0x7F002000usize - 0x20usize,
            )
        };

        let max_stack_used = detect_stack_in_buffer(stack_buffer, 0x5A5A5A5A5A5A5A5Au64).unwrap();
        self.max_stack = max_stack_used;

        log::info!(" detla: {}\n", self.end_timestamp - self.start_timestamp);
        log::info!("detect max stack size is: 0x{:0x}\n", self.max_stack);
        log::info!("detect max heap size is: 0x{:0x}\n\n", ALLOCATOR.max_heap);
    }
}

fn detect_stack_in_buffer(buffer: &[u8], expected_value: u64) -> Option<usize> {
    for i in 0..(buffer.len() / 8) {
        let value: u64 = buffer.pread(i * 8).unwrap();
        if value != expected_value {
            return Some(buffer.len() - i * 8);
        }
    }
    None
}
