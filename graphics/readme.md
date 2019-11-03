# Graphics

This crate has two goals:
1) Create a safe, strongly-typed, less error-prone interface to a graphics backend.
2) Cache graphics state on the CPU to minimize graphics calls.

## Implementation Details

The `Context` object is the centerpiece of this crate. It's primary goal is to track GPU state to elide state changes when possible. To that end, it creates, tracks and destroys graphics resources.