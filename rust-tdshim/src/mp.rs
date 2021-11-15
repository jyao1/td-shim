// Copyright (c) 2020 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

use crate::acpi::{self, GenericSdtHeader};
use core::mem::size_of;
use tdx_tdcall::tdx;
use zerocopy::{AsBytes, FromBytes};

const PAGE_ACCEPT_CHUNK_SIZE: u64 = 0x2000;

const MADT_MAX_SIZE: usize = 0x400;

const NUM_8259_IRQS: usize = 16;

const ACPI_1_0_PROCESSOR_LOCAL_APIC: u8 = 0x00;
const ACPI_1_0_IO_APIC: u8 = 0x01;
const ACPI_1_0_INTERRUPT_SOURCE_OVERRIDE: u8 = 0x02;
const ACPI_MADT_MPWK_STRUCT_TYPE: u8 = 0x10;
const ACPI_1_0_LOCAL_APIC_NMI: u8 = 0x04;

pub struct Madt {
    pub data: [u8; MADT_MAX_SIZE],
    pub size: usize,
}

impl Madt {
    fn default() -> Self {
        Madt {
            data: [0; MADT_MAX_SIZE],
            size: 0,
        }
    }

    fn write(&mut self, data: &[u8]) {
        self.data[self.size..self.size + data.len()].copy_from_slice(data);
        self.size += data.len();
    }

    fn update_checksum(&mut self) {
        let checksum = acpi::calculate_checksum(&self.data[0..self.size]);
        self.data[9] = checksum;
    }
}

#[repr(packed)]
#[derive(Default, AsBytes, FromBytes)]
struct LocalApic {
    pub r#type: u8,
    pub length: u8,
    pub processor_id: u8,
    pub apic_id: u8,
    pub flags: u32,
}

#[repr(packed)]
#[derive(Default, AsBytes, FromBytes)]
struct LocalApicNmi {
    pub r#type: u8,
    pub length: u8,
    pub acpi_processor_id: u8,
    pub flags: u16,
    pub local_apic_inti: u8,
}

#[repr(packed)]
#[derive(Default, AsBytes, FromBytes)]
struct IoApic {
    pub r#type: u8,
    pub length: u8,
    pub ioapic_id: u8,
    _reserved: u8,
    pub apic_address: u32,
    pub gsi_base: u32,
}

#[repr(packed)]
#[derive(Default, AsBytes, FromBytes)]
struct InterruptSourceOverride {
    pub r#type: u8,
    pub length: u8,
    pub bus: u8,
    pub source: u8,
    pub gsi: u32,
    pub flags: u16,
}

#[repr(packed)]
#[derive(Default, AsBytes, FromBytes)]
struct MadtMpwkStruct {
    r#type: u8,
    length: u8,
    mail_box_version: u16,
    reserved: u32,
    mail_box_address: u64,
}

pub fn create_madt(cpu_num: u8, mailbox_base: u64) -> Madt {
    log::info!("create_madt(): cpu_num: {:x}\n", cpu_num);

    let table_length = size_of::<GenericSdtHeader>()
        + 8
        + cpu_num as usize * size_of::<LocalApic>()
        + size_of::<IoApic>()
        + NUM_8259_IRQS * size_of::<InterruptSourceOverride>()
        + size_of::<LocalApicNmi>()
        + size_of::<MadtMpwkStruct>();

    let mut madt = Madt::default();

    let header = GenericSdtHeader::new(*b"APIC", table_length as u32, 1);

    madt.write(header.as_bytes());

    madt.write(&0xfee00000u32.to_le_bytes());
    madt.write(&1u32.to_le_bytes());

    for cpu in 0..cpu_num {
        let lapic = LocalApic {
            r#type: ACPI_1_0_PROCESSOR_LOCAL_APIC,
            length: size_of::<LocalApic>() as u8,
            processor_id: cpu as u8,
            apic_id: cpu as u8,
            flags: 1,
        };
        madt.write(lapic.as_bytes());
    }

    let ioapic = IoApic {
        r#type: ACPI_1_0_IO_APIC,
        length: size_of::<IoApic>() as u8,
        ioapic_id: cpu_num,
        apic_address: 0xFEC00000,
        gsi_base: 0,
        ..Default::default()
    };
    madt.write(ioapic.as_bytes());

    let iso = InterruptSourceOverride {
        r#type: ACPI_1_0_INTERRUPT_SOURCE_OVERRIDE,
        length: size_of::<InterruptSourceOverride>() as u8,
        bus: 0,
        source: 0,
        gsi: 2,
        flags: 5,
    };
    madt.write(iso.as_bytes());

    for irq in 1..NUM_8259_IRQS {
        let iso = InterruptSourceOverride {
            r#type: ACPI_1_0_INTERRUPT_SOURCE_OVERRIDE,
            length: size_of::<InterruptSourceOverride>() as u8,
            bus: 0,
            source: irq as u8,
            gsi: irq as u32,
            flags: 5,
        };
        madt.write(iso.as_bytes());
    }

    let nmi = LocalApicNmi {
        r#type: ACPI_1_0_LOCAL_APIC_NMI,
        length: size_of::<LocalApicNmi>() as u8,
        acpi_processor_id: 0xff,
        flags: 0,
        local_apic_inti: 0x01,
    };
    madt.write(nmi.as_bytes());

    let mpwk = MadtMpwkStruct {
        r#type: ACPI_MADT_MPWK_STRUCT_TYPE,
        length: size_of::<MadtMpwkStruct>() as u8,
        mail_box_version: 1,
        reserved: 0,
        mail_box_address: mailbox_base,
    };
    madt.write(mpwk.as_bytes());

    madt.update_checksum();
    madt
}

fn td_accept_page(address: u64, pages: u64) {
    for i in 0..pages {
        tdx::tdcall_accept_page(address + i * 0x1000);
    }
}

pub fn mp_accept_memory_resource_range(address: u64, size: u64) {
    log::info!(
        "mp_accept_memory_resource_range: 0x{:x} - 0x{:x} ... (wait for 1 min)\n",
        address,
        size
    );

    let pages = PAGE_ACCEPT_CHUNK_SIZE >> 12;

    for i in 0..(size / PAGE_ACCEPT_CHUNK_SIZE) {
        // TBD accept failed if remove this!
        if (address + i * PAGE_ACCEPT_CHUNK_SIZE) % 0x800000 == 0 {
            log::info!(
                "accept pages 0x{:X}\n",
                address + i * PAGE_ACCEPT_CHUNK_SIZE
            );
        }
        td_accept_page(address + i * PAGE_ACCEPT_CHUNK_SIZE, pages);
    }

    log::info!("mp_accept_memory_resource_range: done\n");
}
