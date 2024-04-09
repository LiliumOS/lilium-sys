//! Almost all system calls in Lilium return a `SysResult`
//! This is a signed integer type, equal to the register width of the platform, which can either be non-negative (indicating a succesful operation),
//!  or negative, which indicates an Error.
//! An `OK` (0) result is the most common success result, with positive values having per-syscall meaning documented on the syscall.
//! The negative results are divided into the 5 main lilium subsystems, and general errors.
//! (Note that syscalls belonging to a particular subsystem are not restricted to only errors from that subsystem)
//! 
//! ## Error Groups
//! When multiple error conditions are simultaneously present, which error is returned is not specified.
//! Generally, these errors would have an effective order that prevents this (for example, a String Pointer argument that points to unmapped memory would necessarily return `INVALID_MEMORY`,
//!  rather than `INVALID_STRING`, because the condition for `INVALID_STRING` depends on first accessing the memory)
//! Some System calls may group multiple errors together, such that errors in all previous groups are reported before any errors in subsequent groups

/// Operation failed due to insufficient permissions
#define PERMISSION (-1)
/// A handle argument was expected and one of the following occured:
/// * The given handle was null, and the operation does not have specific behaviour on a null input
/// * The given handle was from a different thread of execution then the current, and this error was detected by the kernel
/// * The given handle was previously closed, and this error was detected by the kernel
/// * The given handle is of an incorrect type for the operation
#define INVALID_HANDLE (-2)
/// A pointer argument refers to memory that is not valid to access for the current thread:
/// * A pointer argument was null, and a null pointer was not accepted by the operation
/// * A pointer argument points to a non-existant mapping
/// * A pointer argument refers to kernel memory
/// * A pointer argument refers to a mapping that was removed, or part of a mapping that was truncated, and no new mapping was created
/// * A pointer argument refers to a handle, and this error was detected by the kernel
/// * A pointer argument referred to a valid mapping but the operation to be performed was invalid on memory in that mapping (i.e. a write operation to a read-only page, or trying to execute a non-executable page)
/// * A pointer argument has an alignment constraint that was violated, and this error is detected by the kernel. 
/// * A pointer argument is valid for fewer bytes than was expected, and this error was detected by the kernel.
/// * A pointer argument referred to valid memory that is reserved for the kernel (such as memory being modified by an asynchronous IO operation), and this error was detected by the kernel.
///
/// Checks for alignment, validity, reservation (for handles or userspace memory reserved for the kernel) are a best effort basis.
/// The kernel generally does not know the precise extent of memory validity and can only be granular to the page boundary.
/// As such, it is possible for a pointer to exceed the bounds of an object it is intended to point into without this error being detected by the kernel,
/// Such an operation causes userspace undefined behaviour
#define INVALID_MEMORY (-3)
/// An attempt was made to perform an operation on an object that is busy or otherwise cannot be acquired.
#define BUSY (-4)
/// An attempt was made to perform an operation that does not exist, or perform an operation on an object that does not allow that operation.
/// Note that this differs from `PERMISSION` in that `PERMISSION` is returned when the current thread is not allowed, but `INVALID_OPERATION` is returned when the operation cannot be performed regardless of permissions.
#define INVALID_OPERATION (-5)
/// An operation expected a string but recieved non-UTF-8 text.
#define INVALID_STRING (-6)
/// An operation expected a mutable string or slice, but the length field indicated fewer elements then the operation attempted to write.
/// The length field is updated to the expected length and the operation may be retried after extending the available memory accordinly
///
/// When this error is returned, and the syscall accepts multiple mutable strings or slices, the behaviour is kernel and syscall dependant, but is either:
/// * The function stops processing immediately after setting the length field for the failing string, and no further strings are modified,
/// * The function continues processing up to a certain number of failures (which can be unbounded), and updates at least all string/slices with insufficient length fields.
/// 
/// The function typically does not report how many insufficient length fields were updated in total.
///
/// Regardless of the failure behaviour, if multiple mutable strings/slices are encountered,
/// the order they are checked and updated for insufficient length conditions is not specified
#define INSUFFICIENT_LENGTH (-7)
/// A thread attempted to acquire a resource but its active security context imposes a limit on that resource that has been exhausted by threads sharing the limit.
#define RESOURCE_LIMIT_EXHAUSTED (-8)
/// An operation was performed on an object that is an incorrect state for that operation, or an argument was in an invalid state for the operation
#define INVALID_STATE (-9)
/// An extended option specifier was provided to an operation that was invalid, for example:
/// * The option has a unrecognized type, and the option was not marked as ignorable
/// * The option sets any reserved (undefined) flag bits
/// * Any reserved field of the option header is not set to 0
#define INVALID_OPTION (-10)

/// An operation was performed that required allocating memory for either the process or the kernel, and the allocation failed for a reason other than a specified resource limit, such as:
/// * The available physical memory on the system is exhausted and insufficient memory could be reacquired for the process,
/// * The available virtual memory region the kernel attempted to allocate for a resource was full,
/// * Allocation for any page tables used to allocate virtual memory failed (note that this cause in particular may be the result of exhausting the threads `AllocateThreadKMem` resource limit)
#define INSUFFICIENT_MEMORY (-11)

/// Indicates that a system call number is invalid/unrecognized by the kernel, a system call operation is not supported in the current kernel build configuration, 
/// or platform restrictions prevented performing a given system call
/// 
/// This differs from `INVALID_OPERATION` in that it specifically detects issues with the SCI function itself, rather than a specific operation requested to be performed by the SCI function.
///
/// Note that some cases of platform restrictions may return `INVALID_OPERATION` instead.
#define UNSUPPORTED_KERNEL_FUNCTION (-12)

/// An enumeration operation was performed, but the enumeration state indicates a finished enumeration operation.
#define FINISHED_ENUMERATE (-32)

// subsystem 1 (threads) Error Codes

/// A blocking operation was performed and was not resumed before the blocking timeout expired
#define TIMEOUT (-0x100)
/// A blocking operation was performed and the thread was interrupted
#define INTERRUPTED (-0x101)
/// An operation was performed on a thread that terminated due to an non-recoverable error, such as:
/// * Recieving a `SIGSEGV` upon executing the initial function because it was not accessible to the spawned at the time the kernel scheduled that thread
/// * The thread was terminated by `DestroyThread`
/// * The process owning the thread was terminated by `SIGKILL`, while that thread is not being debugged.
#define KILLED (-0x102)

// subsystem 2 (io) Error Codes

/// An operation was performed on an object that does not support the operation, or via a handle that does not support the operation, for example:
/// * An I/O operation of a type that is not supported by the handle's characteristics (`IOWrite` on a non-writable handle, `IOSeek` or a random-access operation on a handle that is neither seekable nor random-access)
/// * A device of the wrong type was used in a specialized I/O operation (IE. `GetClockOffset` applied to a filesystem device)
/// * A property was queried or modified that does not apply to the object
/// * A device was attempted to be created of a type that requires some operation but that operation is unsupported or invalid (Creating a block device from a non-random access `IOHandle`)
/// * A device was opened as the wrong type.
/// * A file was opened as writable on a filesystem mounted as read-only
#define UNSUPPORTED_OPERATION (-0x200)
/// An operation was performed that would block on a handle that is configured to perform asynchronous operation, and the operation was scheduled in the background
#define PENDING (-0x203)
/// An operation attempted to locate or access an object that does not exist, or locate an object through a path that does not exist or is not accessible.
#define DOES_NOT_EXIST (-0x204)
/// An operation required an object not being present or inaccessible found the object
#define ALREADY_EXISTS (-0x205)
/// An operation attempted to refer to a device that is not referrable by the current thread or does not exist.
#define UNKNOWN_DEVICE (-0x206)
/// An operation was performed that would block on a handle that is configured to not perform blocking operations.
#define WOULD_BLOCK (-0x207)
/// An write operation would cause a device to exceed it's storage capacity or configured storage limit
#define DEVICE_FULL (-0x208)
/// An operation attempted to access a device that is not able to respond to the operation
#define DEVICE_UNAVAILABLE (-0x209)
/// An operation that refers to a path encountered a loop in resolving symbolic links
#define LINK_RESOLUTION_LOOP (-0x20A)
/// An operation was performed on an object that was closed remotely, such as:
/// * An write operation was performed on a pipe or FIFO object and the read end of the pipe was closed
/// * A read or write operation to an IPC Connection or a socket, and the remote end of the connection was closed
/// * A read or write operation to a socket, and the connection was interrupted
#define CLOSED_REMOTELY (-0x20B)

/// An operation was performed on an connection object, and the connection was interrupted or broken externally
#define CONNECTION_INTERRUPTED (-0x20C)


// subsystem 3 (process) Error Codes

/// A process that was joined was terminated by signal
#define SIGNALED (-0x300)

/// A mapping was referred to by a system call that would cause the mapping to become inaccessible, such as:
/// * A Secure or encrypted mapping is referred to an `IOHandle` used to create a new process, and the new process uses `FLAG_REPLACE_IMAGE`
#define MAPPING_INACCESSIBLE (-0x301)

/// Minimum privileges were required by a spawned process, and those privileges were not acquired
#define PRIVILEGE_CHECK_FAILED (-0x302)


// subsystem 4 (debug) Error Codes
