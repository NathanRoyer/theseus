//! Theseus is a new OS written from scratch in Rust, with the goals of runtime composability and state spill freedom.    
//! 
//! # Structure of Theseus
//! The Theseus kernel is composed of many small modules, each contained within a single Rust crate, and built all together as a cargo workspace. 
//! All crates in Theseus are listed in the sidebar to the left, click on a crate name to read more about what that module does and the functions and types it provides.
//! Each module is a separate project that lives in its own crate, with its own "Cargo.toml" manifest file that specifies that module's dependencies and features. 
//! 
//! Theseus is essentially a "bag of modules" without any source-level hierarchy, as you can see every crate by flatly listing the contents of the `kernel` directory. 
//! However, there are two special "metamodules" that warrant further explanation: the `nano_core` and the `captain`.
//!
//! ## Key Modules
//! #### `nano_core`
//! The `nano_core` is the aptly-name tiny core module that contains the first code to run
//! The `nano_core` is very simple, and only does the following things:
//! 
//! 1. Bootstraps the OS after the bootloade is finished, and initializes simple things like logging.
//! 2. Establishes a simple virtual memory subsystem so that other modules can be loaded.
//! 3. Loads the core library module, the `captain` module, and then calls [`captain::init()`](../captain/fn.init.html) as a final step.
//! 4. That's it! The `nano_core` gives complete control to the `captain` and takes no other action.
//!
//! In general, you shouldn't ever need to change the `nano_core` ... **ever**. That's because the `nano_core` doesn't contain any specific program logic, it just sets up an initial environment so that other things can run.
//! If you want to change how the OS starts up and initializes, you should change the code in the `captain` instead.
//!
//! #### `captain`
//! The `captain` steers the ship of Theseus, meaning that it contains the logic that initializes and connects all the other module crates in the proper order and with the proper flow of data between modules. 
//! Currently, the default `captain` in Theseus loads a bunch of crates, then initializes ACPI and APIC to discover multicore configurations, sets up interrupt handlers, spawns a console thread and createsa queue to send keyboard presses to the console, boots up other cores (APs), unmaps the initial identity-mapped pages, and then finally spawns some test userspace processes (liable to change).     
//! At the end, the `captain` must enable interrupts to allow the system to schedule other Tasks. It then falls into an idle loop that does nothing except yields the processor to another Task.    
//!
//! **Note**: the `captain` makes copious usage of conditional compilation based on the `loadable` feature. You don't need to worry about using this feature when initially developing a new module; it's only necessary once you want to make it runtime-loadable.    
//! It is easiest to read what the `captain` does if you only read the instructions within the NOT-loadable code blocks, i.e., within `#[cfg(not(feature = "loadable"))]`.
//! Soon, a macro will cleanup the structure of `loadable` code such that it is much easier to read. 
//! 


//! # Basic Overview of Each Crate
//! One-line summaries of what each crate includes (may be incomplete):
//! 
//! * `acpi`: ACPI (Advanced Configuration and Power Interface) support for Theseus, including multicore discovery.
//! * `apic`: APIC (Advanced Programmable Interrupt Controller) support for Theseus (x86 only), including apic/xapic and x2apic.
//! * `ap_start`: High-level initialization code that runs on each AP (core) after it has booted up
//! * `ata_pio`: Support for ATA hard disks (IDE/PATA) using PIO (not DMA), and not SATA.
//! * `captain`: The main driver of Theseus. Controls the loading and initialization of all subsystems and other crates.
//! * `console`: A console implementation that allows simple printing to the screen.
//! * `console_types`: A temporary way to move the console typedefs out of the console crate.
//! * `dbus`: Simple dbus-like IPC support for Theseus (incomplete).
//! * `driver_init`: Code for handling the sequence required to initialize each driver.
//! * `e1000`: Support for the e1000 NIC and driver.
//! * `exceptions`: Exceptions handling in Theseus, mostly early exception handlers that are mere placeholders.
//! * `gdt`: GDT (Global Descriptor Table) support (x86 only) for Theseus.
//! * `interrupts`: Interrupt configuration and handlers for Theseus. 
//! * `ioapic`: IOAPIC (I/O Advanced Programmable Interrupt Controller) support (x86 only) for Theseus.
//! * `keyboard`: The keyboard driver.
//! * `memory`: The virtual memory subsystem.
//! * `mod_mgmt`: Module management, including parsing, loading, linking, unloading, and metadata management.
//! * `pci`: Basic PCI support for Theseus, x86 only.
//! * `pic`: PIC (Programmable Interrupt Controller), support for a legacy interrupt controller that isn't used much.
//! * `pit_clock`: PIT (Programmable Interval Timer) support for Theseus, x86 only.
//! * `scheduler`: The scheduler and runqueue management.
//! * `spawn`: Functions and wrappers for spawning new Tasks, both kernel threads and userspace processes.
//! * `syscall`: Initializes the system call support, and provides basic handling and dispatching of syscalls in Theseus.
//! * `task`: Task types and structure definitions, a Task is a thread of execution.
//! * `tsc`: TSC (TimeStamp Counter) support for performance counters on x86. Basically a wrapper around rdtsc.
//! * `tss`: TSS (Task State Segment support (x86 only) for Theseus.
//!
//! 



//! # Theseus's Build Process
//! Theseus uses the [cargo virtual workspace](https://doc.rust-lang.org/cargo/reference/manifest.html#the-workspace-section) feature to group all of the crates together into a single meta project, which significantly speeds up build times.     
//! 
//! The top-level Makefile basically just calls the kernel Makefile, copies the kernel build files into a top-level build directory, and then calls `grub-mkrescue` to generate a bootable .iso image.     
//!
//! The kernel Makefile (`kernel/Makefile`) actually builds all of the Rust code using [`xargo`](https://github.com/japaric/xargo), a cross-compiler toolchain that wraps the default Rust `cargo`.
//! The only special action it takes is to build the `nano_core` separately and fully link it against the architecture-specific assembly code in `nano_core/boot` into a static binary.    
//! 
//! ### Debug vs. Release mode
//! There is a special file `kernel/Config.mk` that contains configuration options used in the `kernel/Makefile`. 
//! Among other things that are well-documented in that file, this lets you switch between Rust's **debug** and **release** modes by setting the `BUILD_MODE` variable.     
//! As with most languages, **release** mode in Rust is *way* faster, but takes much longer to compile and is difficult to debug. Unless you're evaluauting the performance of Theseus, it's best to stick with **debug** mode.
//! 


//! ## Proper Runtime Module Loading
//! By default, Theseus is built into a single kernel binary just like a regular OS, in which all crates are linked into a single static library and then zipped up into a bootable .iso file. 
//! However, the actual research into runtime composability dictates that all modules (except the `nano_core`) are loaded at runtime, and not linked into a single static kernel binary. 
//!
//! To enable this, use the `make loadable` command to enable the `loadable` feature, which does the following:
//!
//! * Builds each crate into its own separate object file, which are not all linked together like in other OSes.
//! * Copies each crate's object file into the top-level build directory's module subdirectory (`build/grub-isofiles/modules`) such that each module is a separate object file in the final .iso image. 
//!   That allows the running instance of Theseus to see all the modules currently available just by asking the bootloader (without needing a filesystem), and to load them individually.
//! * Sets the `loadable` config option, which as seen in the `nano_core` and `captain` code, will enable the `#![cfg(loadable)]` code blocks that load each crate. 
//! 
//! 


//! # Booting and Flow of Execution
//! The kernel takes over from the bootloader (GRUB, or another multiboot2-compatible bootloader) in `nano_core/src/boot/arch_x86_64/boot.asm:start` and is running in *32-bit protected mode*. 
//! After initializing paging and other things, the assembly file `boot.asm` jumps to `long_mode_start`, which runs 64-bit code in long mode. 
//! Then, it jumps to `start_high`, so now we're running in the higher half because Theseus is a [higher-half kernel](https://wiki.osdev.org/Higher_Half_Kernel). 
//! We then set up a new Global Descriptor Table (GDT), segmentation registers, and finally call the Rust code entry point [`nano_core_start()`](../nano_core/fn.nano_core_start.html) with the address of the multiboot2 boot information structure as the first argument (in register RDI).
//!
//! After calling `nano_core_start`, the assembly files are no longer used, and `nano_core_start` should never return. 



//! # Adding New Functionality to Theseus
//! The easiest way to add new functionality is just to create a new crate by duplicating an existing crate and changing the details in its new `Cargo.toml` file.
//! At the very least, you'll need to change the `name` entry under the `[package]` heading at the top of the `Cargo.toml` file, and you'll most likely need to change the dependencies for your new crate.     
//!
//! If your new crate needs to be initialized, you can invoke it from the [`captain::init()`](../captain/fn.init.html) function, although there may be more appropriate places to do so, such as the [`driver_init::init()`](../driver_init/fn.init.html) function for drivers.
//! 



