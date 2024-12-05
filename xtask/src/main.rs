use std::env::args;
use std::process::Command;
use std::os::unix::process::CommandExt;
use std::sync::Mutex;

static DEFERRED: Mutex<Vec<&str>> = Mutex::new(vec![]);

fn build() -> bool {
    /*    
        [build]
        target = "kern/os-dev-target.json"

        [unstable]
        build-std-features = ["compiler-builtins-mem"]
        build-std = ["core", "compiler_builtins"]

        [profile.dev]
        panic = "abort" 

        [profile.release]
        panic = "abort" 
     */


    match Command::new("cargo")
        .args(["+nightly", "build", "--bin", "kern",
            "--config", "build.target = \"kern/os-dev-target.json\"",
            "--config", "unstable.build-std-features = [\"compiler-builtins-mem\"]",
            "--config", "unstable.build-std = [\"core\", \"compiler_builtins\"]",
            "--config", "profile.dev.panic = \"abort\"",
            "--config", "profile.release.panic = \"abort\"",
            //"--config", "rustflags=[\"-C\", \"\"]"
        ])
        .status()
        .expect("failed a call to cargo")
        .code() {
        Some(0) => println!("cargo build succeeded"),
        Some(status_code) => { println!("cargo build failed with status code {status_code}"); return false },
        None => { println!("cargo build did not terminate with status code (was terminated by signal)"); return false },
    }

    match Command::new("sudo")
        .args(["losetup", "-P", "/dev/loop0", "kern/disk.img"])
        .status()
        .expect("failed a call to losetup")
        .code() {
            Some(0) => println!("losetup created"),
            Some(_) | None => { println!("losetup failed"); return false },
        }

    match DEFERRED.lock() {
        Ok(mut v) => v.push("losetup"),
        Err(_) => {
            println!("Failed to defer the losetup\nDeleting it now");
            let _ = Command::new("losetup").args(["-d", "/dev/loop0"]).spawn();
            return false;
        }
    }

    if std::fs::create_dir("./temporary_mount").is_err() {
        println!("Failed to create a temporary mount");
        return false;
    }

    println!("Successfully created the folder");
    
    match DEFERRED.lock() {
        Ok(mut v) => v.push("mkdir"),
        Err(_) => {
            println!("Failed to defer the folder deletion\nDeleting it now");
            std::fs::remove_dir("./temporary_mount").expect("Failed to remove the directory");
            return false;
        }
    }

    match Command::new("sudo")
        .args(["mount","/dev/loop0p1", "./temporary_mount"])
        .status()
        .expect("failed a call to mount")
        .code() {
        Some(0) => println!("Mounted the disk image loopback device to ./temporary_mount"),
        None | Some(_) => { println!("Failed to mount the disk image loopback"); return false }
    }

    match DEFERRED.lock() {
        Ok(mut v) => v.push("mount"),
        Err(_) => {
            println!("Failed to defer the unmounting\nUnmounting now");
            let _ = Command::new("umount")
                .args(["./temporary_mount"])
                .spawn();
            return false;
        }
    }

    match Command::new("sudo")
        .args(["cp", "./target/os-dev-target/debug/kern", "./temporary_mount/kernel_entry"])
        .status()
        .expect("failed a call to cp")
        .code() {
        Some(0) => println!("Copied the kernel entry over"),
        None | Some(_) => { println!("Failed to copy the kernel entry over"); return false }
    }

    true
}

fn deferred_build() -> bool {
    let res = build();
    let mut defer_list = DEFERRED.lock().expect("Failed to defer");
    defer_list.reverse();

    //println!("{defer_list:#?}");

    for op in defer_list.iter() { 
        match *op {
            "losetup" => {
                let _ = Command::new("sudo").args(["losetup","-d", "/dev/loop0"])
                    .output().expect("Failed to call deferred losetup -d");
                println!("Deferred losetup process");
            },
            "mkdir" => {
                let _ = std::fs::remove_dir("./temporary_mount")
                    .expect("Failed to process deferred rmdir");
                println!("Deferred directory removal");
            },
            "mount" => {
                let _ = Command::new("sudo").args(["umount", "./temporary_mount"])
                    .output().expect("Failed to process deferred unmount");
                println!("Deferred unmount process");
            },
            other => { println!("Unexpectedly found '{other}' in the defer list"); return false }
        }
    }

    res
}

fn test() {
    if !deferred_build() {
        println!("The build failed, not proceeding with test");
        return
    };

    Command::new("qemu-system-x86_64")
        .args(["-s", "-S", "-display", "gtk,zoom-to-fit=on",
            "--bios", "/nix/store/y0c428fwc7z8bp5m36c3d00qcn3qyx8g-OVMF-202402-fd/FV/OVMF.fd", "-drive",
            "file=kern/disk.img,format=raw,index=0,media=disk"])
        .spawn()
        .expect("Failed to open qemu");

    Command::new("gdb")
        .args(["-ex", "target remote localhost:1234", "target/os-dev-target/debug/kern"])
        .exec();
    /*
    Command::new("alacritty")
        .args(["-e", "bash", "--rcfile",
            "<(echo \". ~/bashrc; gdb -ex 'target remote localhost:1234' target/os-dev-target/debug/kern\")"])
        .spawn()
        .expect("Failed to open terminal window with gdb"); */
}

fn main() {
    match args().nth(1).expect("An argument must be provided to xtask").as_str() {
        "build" => { deferred_build(); },
        "test" => { test(); },
        cmd => { println!("Did not recognise command {cmd}"); }
    };
}
