# linux-compatible-kernel-RUST

A from scratch Linux-compatible kernel in Rust (x86-64).
Goal: run unmodified Linux userland, from busybox to Debian.

> Work in progress. Boots, but does not run userspace yet.

## Status

Currently boots via Limine in QEMU and prints to the serial port.

## Build and run

Requirements: Rust (via [rustup](https://rustup.rs)), `qemu-system-x86`, `ovmf`, `git`.

On Debian/Kali/Ubuntu:

```
sudo apt install qemu-system-x86 ovmf git
```

Then:

```
git clone https://github.com/kerikeri1/linux-compatible-kernel-RUST.git
cd linux-compatible-kernel-RUST
cargo xtask run
```

You should see `Hello from ferrux!` on the terminal.
Exit QEMU with `Ctrl+A` then `X`.

## Milestones

- [x] 0.1 Boot with Limine, serial output
- [ ] 0.2 GDT, IDT, exception handlers
- [ ] 0.3 Physical memory manager, paging, kernel heap
- [ ] 0.4 Scheduler and context switch
- [ ] 0.5 Userspace (ring 3) and syscalls
- [ ] 0.6 ELF loader, static musl hello world
- [ ] 0.7 busybox shell (initramfs, fork/exec, pipes, TTY)
- [ ] 0.8 Signals and job control
- [ ] 0.9 Threads, futex, mmap
- [ ] 1.0 ext2 on virtio-blk, networking, dynamic glibc, Debian

## Project layout

```
kernel/   the kernel (target x86_64-unknown-none)
xtask/    build tooling (cargo xtask run / debug)
docs/     notes and design docs
```

## License

To be decided.
