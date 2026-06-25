# crater

A minimal container runtime for Linux, written in Rust. crater creates and runs
isolated processes using Linux namespaces, OverlayFS, virtual networking, and
cgroups v2 — a small-scale take on what Docker does under the hood.

> Requires Linux and root privileges (namespaces, mounts, cgroups, and netlink
> all need elevated permissions). Built against Rust edition 2024.

## Install

```sh
cargo build --release
# binary at target/release/crater
```

## Usage

```sh
# Create a container from an image, naming it and giving it a command
crater create -i <image> -n <name> -- <program> [args...]

# Run a container (foreground, or -d to detach)
crater run <name> [-d]

# Stop a running container (SIGTERM)
crater stop <name>

# List images or containers
crater list -i        # images
crater list -c        # containers

# Delete an image or container
crater delete -i <image>
crater delete -c <container>
```

## Built features

**Container lifecycle.** Create, run, stop, list, and delete. Each container
tracks a state of `Created`, `Running`, or `Dead`, along with its PID. Run
supports both foreground and detached (`-d`) mode; detaching uses `fork` +
`setsid` to fully background the process.

**Metadata store.** Container definitions (image, command args, state, PID) are
serialized to JSON under `/crater/metadata/<name>.json`.

**Namespace isolation.** Containers are unshared into new UTS and PID namespaces,
then the child additionally unshares NET and MNT namespaces. The container
hostname is set to `container-host`.

**Filesystem isolation (OverlayFS).** Each container gets an overlay mount with
the image as the read-only `lowerdir`, plus per-container `state` (upperdir) and
`overlay` (workdir) directories. The merged view becomes the new root via
`pivot_root`, and the old root is detached.

**Virtual networking.** A host bridge (`crater-br`) is created once; each
container gets a veth pair. The host end is attached to the bridge, the container
end is moved into the container's network namespace and assigned `10.0.0.2/24`
with a default route to `10.0.0.1`. Loopback is brought up inside the container.

**Resource limits (cgroups v2).** A `crater` cgroup caps CPU at 20% (`20000/100000`),
memory at 100 MB, and enables the `pids` controller. Each container process is
attached to this cgroup.

## Storage layout

| Path                        | Contents                                  |
|-----------------------------|-------------------------------------------|
| `/crater/images`            | Images (see below)                        |
| `/crater/metadata`          | Per-container metadata JSON               |
| `/crater/filesystem-state`  | Per-container overlay state/overlay/merged dirs |

## Image format

> **Not built yet.** There is currently no image format, no packaging, and no
> pulling from a registry. Right now an "image" is simply a directory on the
> filesystem under `/crater/images/<name>` containing a root filesystem. crater
> uses that directory directly as the read-only OverlayFS `lowerdir`. To add an
> image, place an unpacked root filesystem at `/crater/images/<name>` yourself.

A real image format (packaging, distribution, and a defined on-disk layout) is
planned but not yet implemented.
