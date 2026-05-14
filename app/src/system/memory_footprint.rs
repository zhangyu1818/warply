/// Returns the full memory footprint of the current process, in bytes.
///
/// Unlike RSS (resident set size), this includes memory that has been swapped
/// out or compressed by the OS.  On macOS, this returns `phys_footprint` from
/// `task_info(TASK_VM_INFO)`, which is the same value displayed by Activity
/// Monitor.
pub fn memory_footprint_bytes() -> u64 {
    platform::memory_footprint_bytes()
}

/// Returns a platform-specific JSON object with a detailed breakdown of the
/// current process's memory usage.
///
/// Each platform populates whichever fields it can natively provide.
pub fn memory_breakdown() -> serde_json::Value {
    platform::memory_breakdown()
}

mod platform {
    use std::mem;

    use mach2::kern_return::KERN_SUCCESS;
    use mach2::task::task_info;
    use mach2::task_info::{task_vm_info, TASK_VM_INFO};
    use mach2::traps::mach_task_self;

    /// Calls `task_info(TASK_VM_INFO)` and returns the populated struct on
    /// success, or `None` if the call fails.
    fn query_task_vm_info() -> Option<task_vm_info> {
        // SAFETY: We zero-initialise the struct and pass its exact size to the
        // kernel.  `task_info` writes into the struct up to `count` natural
        // ints and returns `KERN_SUCCESS` on success.
        unsafe {
            let mut info: task_vm_info = mem::zeroed();
            let mut count = (mem::size_of::<task_vm_info>() / mem::size_of::<i32>()) as u32;
            let kr = task_info(
                mach_task_self(),
                TASK_VM_INFO,
                &mut info as *mut _ as *mut i32,
                &mut count,
            );
            if kr == KERN_SUCCESS {
                Some(info)
            } else {
                None
            }
        }
    }

    pub fn memory_footprint_bytes() -> u64 {
        query_task_vm_info()
            .map(|info| info.phys_footprint)
            .unwrap_or(0)
    }

    pub fn memory_breakdown() -> serde_json::Value {
        let Some(info) = query_task_vm_info() else {
            return serde_json::json!({});
        };

        // Copy fields out of the packed struct into locals to avoid
        // unaligned references (task_vm_info is repr(C, packed(4))).
        let total_footprint = info.phys_footprint;
        let resident = info.resident_size;
        let compressed = info.compressed;
        let internal = info.internal;
        let device = info.device;
        let gpu_memory = info.ledger_tag_graphics_footprint;
        let gpu_memory_compressed = info.ledger_tag_graphics_footprint_compressed;
        let media_memory = info.ledger_tag_media_footprint;
        let neural_memory = info.ledger_tag_neural_footprint;
        let purgeable = info.ledger_purgeable_nonvolatile;

        serde_json::json!({
            "total_footprint": total_footprint,
            "resident": resident,
            "compressed": compressed,
            "internal": internal,
            "device": device,
            "gpu_memory": gpu_memory,
            "gpu_memory_compressed": gpu_memory_compressed,
            "media_memory": media_memory,
            "neural_memory": neural_memory,
            "purgeable": purgeable,
        })
    }
}
