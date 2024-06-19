# Memory hacking library in Rust

## Latest breaking changes

- Rename `memwar::mem::Allocation::deref_chain` to `memwar::mem::Allocation::deref_chain_with_base`

## Example usage

```toml
# Cargo.toml

[dependencies]
memwar = { git = "https://github.com/zofiaclient/memwar", package = "memwar" }
```

```rust
use anyhow::{Result, bail, anyhow};

fn sandbox() -> Result<()> {
    let wpinf = match process::get_process_by_name("Game.exe")
        .map_err(|e| anyhow!("Failed to find Game.exe! Last OS error: {e}"))?
    {
        Some((wpinf, _)) => wpinf,
        None => bail!(
            "Failed to get window information! Last OS error: {}",
            GetLastError()
        ),
    };

    let h_process = process::open_process_handle(wpinf.pid())
        .map_err(|e| anyhow!("Failed to open handle to Game.exe! Last OS error: {e}"))?;

    let base_addr = module::get_mod_base(wpinf.pid(), "GameAssembly.dll");

    if base_addr.is_null() {
        bail!("Failed to get GameAssembly.dll base address!")
    }
    
    // Some more logic..

    let alloc = Allocation::existing(h_process, base_addr);
    
    let player_speed_addr = alloc
        .deref_chain(offsets::LOCAL_PLAYER, offsets::PLAYER_SPEED)
        .map_err(|e| anyhow!("Failed to dereference pointer chain! Last OS error: {e}"))?;

    let player_speed = alloc
        .read_f32(player_speed_addr)
        .map_err(|e| anyhow!("Failed to read from Game.exe! Last OS error: {e}"))?;
}
```

## Planned features

- Manual mapping of payload DLLs
