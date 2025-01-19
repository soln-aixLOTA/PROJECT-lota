const std = @import("std");

// Mock NVML types and functions for development/testing
const MockNvml = struct {
    pub const nvmlDevice_t = *anyopaque;
    pub const nvmlReturn_t = c_uint;
    pub const NVML_SUCCESS: nvmlReturn_t = 0;
    pub const NVML_ERROR_UNINITIALIZED: nvmlReturn_t = 1;
    pub const NVML_ERROR_INVALID_ARGUMENT: nvmlReturn_t = 2;
    pub const NVML_ERROR_NOT_SUPPORTED: nvmlReturn_t = 3;
    pub const NVML_ERROR_NO_PERMISSION: nvmlReturn_t = 4;
    pub const NVML_ERROR_ALREADY_INITIALIZED: nvmlReturn_t = 5;
    pub const NVML_ERROR_NOT_FOUND: nvmlReturn_t = 6;
    pub const NVML_ERROR_INSUFFICIENT_POWER: nvmlReturn_t = 7;
    pub const NVML_ERROR_INSUFFICIENT_RESOURCES: nvmlReturn_t = 8;
    pub const NVML_ERROR_UNKNOWN: nvmlReturn_t = 999;

    pub const nvmlMemory_t = extern struct {
        total: u64,
        free: u64,
        used: u64,
    };

    pub fn nvmlInit() nvmlReturn_t {
        return NVML_SUCCESS;
    }

    pub fn nvmlDeviceGetCount(count: *c_uint) nvmlReturn_t {
        count.* = 1; // Mock a single GPU
        return NVML_SUCCESS;
    }

    pub fn nvmlDeviceGetHandleByIndex(index: c_uint, device: *nvmlDevice_t) nvmlReturn_t {
        if (index >= 1) {
            return NVML_ERROR_INVALID_ARGUMENT;
        }
        // Create a mock device handle
        const mock_handle = @as(*anyopaque, @ptrFromInt(0x12345678));
        device.* = mock_handle;
        return NVML_SUCCESS;
    }

    pub fn nvmlDeviceGetMemoryInfo(device: nvmlDevice_t, memory: *nvmlMemory_t) nvmlReturn_t {
        _ = device;
        memory.* = .{
            .total = 8 * 1024 * 1024 * 1024, // 8GB
            .free = 6 * 1024 * 1024 * 1024,  // 6GB
            .used = 2 * 1024 * 1024 * 1024,  // 2GB
        };
        return NVML_SUCCESS;
    }

    pub fn nvmlShutdown() nvmlReturn_t {
        return NVML_SUCCESS;
    }
};

// Use mock NVML for development/testing
const nvml = MockNvml;

pub const NvmlError = error{
    InitializationFailed,
    DeviceNotFound,
    InvalidHandle,
    InsufficientMemory,
    NotSupported,
    NoPermission,
    UnknownError,
};

pub const GpuDevice = struct {
    handle: nvml.nvmlDevice_t,
    index: u32,

    pub fn getMemoryInfo(self: *const GpuDevice) !struct { total: u64, free: u64, used: u64 } {
        var memory: nvml.nvmlMemory_t = undefined;
        const result = nvml.nvmlDeviceGetMemoryInfo(self.handle, &memory);
        if (result != nvml.NVML_SUCCESS) {
            return switch (result) {
                nvml.NVML_ERROR_UNINITIALIZED => NvmlError.InitializationFailed,
                nvml.NVML_ERROR_INVALID_ARGUMENT => NvmlError.InvalidHandle,
                nvml.NVML_ERROR_NOT_SUPPORTED => NvmlError.NotSupported,
                nvml.NVML_ERROR_NO_PERMISSION => NvmlError.NoPermission,
                else => NvmlError.UnknownError,
            };
        }
        return .{
            .total = memory.total,
            .free = memory.free,
            .used = memory.used,
        };
    }
};

// Initialize NVML
pub export fn nvml_init() i32 {
    const result = nvml.nvmlInit();
    return if (result == nvml.NVML_SUCCESS) 0 else -1;
}

// Get device count
pub export fn nvml_get_device_count() i32 {
    var count: c_uint = undefined;
    const result = nvml.nvmlDeviceGetCount(&count);
    return if (result == nvml.NVML_SUCCESS) @intCast(count) else -1;
}

// Get device by index
pub export fn nvml_get_device_by_index(index: i32) ?*GpuDevice {
    var handle: nvml.nvmlDevice_t = undefined;
    const result = nvml.nvmlDeviceGetHandleByIndex(@intCast(index), &handle);
    if (result != nvml.NVML_SUCCESS) {
        return null;
    }

    const device = std.heap.c_allocator.create(GpuDevice) catch {
        return null;
    };

    device.* = .{
        .handle = handle,
        .index = @intCast(index),
    };

    return device;
}

const MemoryInfo = extern struct {
    total: u64,
    free: u64,
    used: u64,
};

export fn nvml_get_device_memory_info(_: c_int) MemoryInfo {
    return MemoryInfo{
        .total = 8 * 1024 * 1024 * 1024, // 8GB
        .free = 4 * 1024 * 1024 * 1024,  // 4GB
        .used = 4 * 1024 * 1024 * 1024,  // 4GB
    };
}

test "gpu memory info" {
    // Initialize NVML
    try std.testing.expectEqual(@as(i32, 0), nvml_init());
    defer _ = nvml.nvmlShutdown();

    // Get device count
    const count = nvml_get_device_count();
    try std.testing.expect(count >= 0);

    if (count > 0) {
        // Get first device
        const device = nvml_get_device_by_index(0);
        try std.testing.expect(device != null);

        // Get memory info
        const memory = nvml_get_device_memory_info(0);
        try std.testing.expect(memory.total > 0);
        try std.testing.expect(memory.used <= memory.total);
        try std.testing.expect(memory.free <= memory.total);
        try std.testing.expect(memory.used + memory.free == memory.total);
    }
}
