// Copyright © 2019 Intel Corporation
//
// SPDX-License-Identifier: Apache-2.0 OR BSD-3-Clause
//

use crate::VfioContainer;
use crate::{get_host_address_range, ExternalDmaMapping};
use std::convert::TryInto;
use std::io;
use std::sync::Arc;
use vm_memory::{GuestAddress, GuestAddressSpace};

/// This structure implements the ExternalDmaMapping trait. It is meant to
/// be used when the caller tries to provide a way to update the mappings
/// associated with a specific VFIO container.
pub struct VfioDmaMapping<M: GuestAddressSpace> {
    container: Arc<VfioContainer>,
    memory: Arc<M>,
}

impl<M: GuestAddressSpace> VfioDmaMapping<M> {
    /// Create a DmaMapping object.
    ///
    /// # Parameters
    /// * `container`: VFIO container object.
    /// * `memory·: guest memory to mmap.
    pub fn new(container: Arc<VfioContainer>, memory: Arc<M>) -> Self {
        VfioDmaMapping { container, memory }
    }
}

impl<M: GuestAddressSpace + Sync + Send> ExternalDmaMapping for VfioDmaMapping<M> {
    fn map(&self, iova: u64, gpa: u64, size: u64) -> std::result::Result<(), io::Error> {
        let user_addr = if let Some(addr) = get_host_address_range(
            &*self.memory.memory(),
            GuestAddress(gpa),
            size.try_into().unwrap(),
        ) {
            addr as u64
        } else {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "failed to convert guest address 0x{:x} into \
                     host user virtual address",
                    gpa
                ),
            ));
        };

        self.container
            .vfio_dma_map(iova, size, user_addr)
            .map_err(|e| {
                io::Error::new(
                    io::ErrorKind::Other,
                    format!(
                        "failed to map memory for VFIO container, \
                         iova 0x{:x}, gpa 0x{:x}, size 0x{:x}: {:?}",
                        iova, gpa, size, e
                    ),
                )
            })
    }

    fn unmap(&self, iova: u64, size: u64) -> std::result::Result<(), io::Error> {
        self.container.vfio_dma_unmap(iova, size).map_err(|e| {
            io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "failed to unmap memory for VFIO container, \
                     iova 0x{:x}, size 0x{:x}: {:?}",
                    iova, size, e
                ),
            )
        })
    }
}
