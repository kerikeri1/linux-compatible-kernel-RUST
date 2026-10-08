use std::{env, fs, path::Path, process::{exit, Command}};

fn main() {
    match env::args().nth(1).as_deref() {
        Some("build") => build(),
        Some("run") => run(false),
        Some("debug") => run(true),
        _ => {
            eprintln!("usage: cargo xtask <build|run|debug>");
            exit(1);
        }
    }
}

fn sh(cmd: &mut Command) {
    let status = cmd.status().expect("failed to spawn command");
    if !status.success() {
        exit(1);
    }
}

fn build() {
    sh(Command::new("cargo").args([
        "build", "-p", "kernel", "--target", "x86_64-unknown-none",
    ]));
}

fn fetch_limine() {
    if !Path::new("limine").exists() {
        sh(Command::new("git").args([
            "clone",
            "https://github.com/limine-bootloader/limine.git",
            "--branch=v9.x-binary",
            "--depth=1",
        ]));
    }
}

fn make_image() {
    build();
    fetch_limine();
    fs::create_dir_all("target/image/EFI/BOOT").unwrap();
    fs::create_dir_all("target/image/boot").unwrap();
    fs::copy("limine/BOOTX64.EFI", "target/image/EFI/BOOT/BOOTX64.EFI").unwrap();
    fs::copy("kernel/limine.conf", "target/image/limine.conf").unwrap();
    fs::copy(
        "target/x86_64-unknown-none/debug/kernel",
        "target/image/boot/kernel",
    )
    .unwrap();
}

fn run(debug: bool) {
    make_image();
    let mut qemu = Command::new("qemu-system-x86_64");
    qemu.args([
        "-M", "q35", "-m", "512M",
        "-serial", "stdio", "-display", "none",
        "-bios", "/usr/share/ovmf/OVMF.fd",
        "-drive", "format=raw,file=fat:rw:target/image",
    ]);
    if debug {
        qemu.args(["-s", "-S"]); // GDB su localhost:1234, CPU ferma all'avvio
    }
    sh(&mut qemu);
}
