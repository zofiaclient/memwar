# Writing a-salt-cube

## Prerequisites

- A decent knowledge of RustLang

## Outline

### Writing cheat requirements

1. Finding game modules
2. Finding pointers and offsets (we'll use Cheat Engine)

### Coding

1. Create command line interface to modify cheat properties during execution
2. Open handles to the target process and module
3. Creating threads to handle cheat logic in parallel
4. Dereference pointer chains and write cheat values

## Finding our pointers

Open AssaultCube, and enter the singleplayer mode.

First, I'm going to open up Cheat Engine and attach it to the Assault Cube process.<br>
From here, we can enter our initial health value (100) into the **value** field and start our first scan.

From my first scan, I got 996 results. This is not ideal, as we need to find a small, consistent pool of addresses that
hold our health value.

So, let's run another scan after taking some damage from a bot friend.

One result! Wow, that was quick. For larger games, you will encounter more addresses finding pointers this way, and it
may be of use then to look into tools designed for reversing game data.

Let's click on our address and head down to the red arrow to save it in our address list.
After we have found our address(es), we need to run a pointer scan to see what points to our health address.

Right-click on the address from the address table and press `Pointer scan for this address`, then simply press ok in the
scan options prompt.

After our scan has completed we should get fairly large amount of entries. Not to worry though, this doesn't really
matter as we're going to automatically sort the offsets.

Click on each offset category to sort to the pointer with the least amount and smallest of offsets.
We're going to want to look for a module we can read from, followed by a small series of offsets. For AssaultCube, I
found the health pointer to be at `"ac_client.exe"+0017E0A8 [0xEC]`

I guess I never explained what a pointer, an address, or an offset really is.

A pointer is made of up of a base address, and offsets. This stores the address to a location in memory, and the offsets
are a series of other pointers, relative to our base pointer.

For example, if I have pointer `"game.exe"+0x100 [0x0]`, we would first read from the module "game.exe" to find its
address. Then, we read at address `0x100` to get a pointer to, let's say our local player. We read the pointer `0x0` (0)
bytes away to get our final address, pointing to I don't know, the player id, which we can then use to later modify the
value of the player id.

Addresses are our way of accessing values held in memory. When we invoke our system API to read or write process memory,
we need to give it an address, a location of where the data is stored at.

```
# 0 - 60
0x0..0x40 => Example Executable Info, 

# 100
0x64 => Player
  # 100
  | 0x64 => Player Id
  
  # 104 - 114
  | 0x68..0x72 => Player Name
```

Offsets are just distance from one address to another, take the following example:

```rust
// "game.exe" + 0x10
#[repr(C)]
struct Player {
    id: u32, // offset: 0x0
    name: [u8; 10], // offset: 0x4 (an u32 is four bytes in size, so our next field would be four bytes away)
}
```

## Creating our project

Run `cargo init <NAME>` to create your Cargo project.

We will be using one of my Windows-only libraries to assist us in developing the cheat. I know, right? If you're like
me, I hate when tutorials do that too. Feel free to look at the library's source to see how we interact with processes
and memory, and write your own instead. It's a fun task, really, so don't get discouraged.

Let's add the `memwar` library to the Cargo.toml, along with [anyhow](https://crates.io/crates/anyhow) to help with error
handling, and my crate `cnsl` for reading user prompts from our command line:

```toml
[dependencies]
memwar = { git = "https://github.com/imajindevon/memwar" }
anyhow = "1.0.86"
cnsl = "0.1.3"
```

## Writing the main function

We outlined our program logic previously, but now we need to outline the
processes:

```text
main cli thread - accepts user input to change values
health cheat thread - modifies ingame health value
```

In our main function, I'm going to write some code to find our process and open a handle to it, returning an error if it
fails.

Although, we need to account for the fact that most of memwar's functions are unsafe. So let's run all of our code in a
separate, unsafe function.

```rust
// main.rs

use memwar::process;
use anyhow::{anyhow, Result};

unsafe fn run() -> Result<()> {
    let pid = process::get_process_by_name("ac_client.exe")
        .map_err(|e| anyhow!("Failed to get window! OS error: {e}"))?
        .ok_or_else(|| anyhow!("Failed to find ac_client.exe!"))?;
    
    let h_process = process::open_process_handle(pid)
        .map_err(|e| anyhow!("Failed to open a handle to AssaultCube.exe! OS error: {e}"))?;
    
    Ok(())
}

fn main() -> Result<()> {
    unsafe {
        run()
    }
}
```

Now run your program with `cargo run`, and see if you can catch an error.
If so, don't worry. Just remember Google is your best friend as a developer, so make sure to use it to your full
advantage.

We all make simple mistakes! I got the following error when I first ran the program:

```
Error: Failed to find AssaultCube.exe!
```

After looking through the task manager, I was able to see the problem. The AssaultCube process is named `ac_client.exe`.
After fixing our code, we can see the program returns without an error.

## Writing our `tasks.rs` module

NOTE: As of memwar v0.1.1, a Task structure is available for you to develop your cheat thread around.

Create a module named `tasks.rs`.
Our tasks module will contain a Tasks struct that will hold all of our tasks.

NOTE: AssaultCube is a 32-bit process. Our following logic will contain code that will NOT work if you build to a
64-bit target. To fix this, append this option to your Cargo build command to compile to a 32-bit target:
`--target i686-pc-windows-msvc`

Instead of writing the health value once, we will continuously update the value with our modified value in a loop. This
is also a decent exercise to get you familiar with threads.

```rust
// tasks.rs

unsafe fn new_health_task(alloc: SendAlloc) -> Task<i32, u32> {
    todo!()
}

pub struct Tasks {
    health_task: Task<i32, u32>,
}

impl Tasks {
    pub fn health_task(&self) -> &Task<i32, u32> {
        &self.health_task
    }

    pub unsafe fn from_alloc(alloc: SendAlloc) -> Self {
        Self {
            health_task: new_health_task(alloc),
        }
    }
}
```

## Defining constants in the `pointers.rs` module

Now that we have our half-finished implementation of our tasks module, we need to store a few constants. These constants
will store the game module's address of our base pointer, and the offsets of the chain of later pointers.

Remember this? `"ac_client.exe"+0017E0A8 [0xEC]` In this format, `0017E0A8` is the address of our base pointer,
(only in ac_client.exe), and after adding it to the base address of module "ac_client.exe", we need to dereference one
more pointer. We need to add offset `0xEC` to get the address of our final health pointer. It is safe to assume that
address `0017E0A8` gave us our local player address, which we can take advantage of later on.

```rust
// pointers.rs

pub const LOCAL_PLAYER: usize = 0x0017E0A8;

/// Value type: i32
pub const OFFS_LOCAL_PLAYER_HEALTH: [usize; 1] = [0xEC];
```

## Finishing `new_health_task`

For this approach I used a `Sender<u32>` and `Receiver<u32>` to send and receive the health value across threads.
I also used a `Sender<u32>` and `Receiver<u32>` to send and receive errors that occurred in the thread. This will be
useful for debugging broken pointers and operations.

```rust
unsafe fn new_health_task(alloc: SendAlloc) -> Task<i32, u32> {
    let (health_sender, health_receiver) = mpsc::channel();
    let (error_sender, error_receiver) = mpsc::channel();
    todo!()
}
```

We use an `AtomicBool` to modify and read if the cheat is enabled.

```rust
unsafe fn new_health_task(alloc: SendAlloc) -> Task<i32, u32> {
    // ...
    let is_enabled = Arc::<AtomicBool>::default();
    let is_enabled_sent = is_enabled.clone();
    todo!()
}
```

We need to check if our cheat is loaded, or if a new modified health value has been sent from the CLI.

```rust
unsafe fn new_health_task(alloc: SendAlloc) -> Task<i32, u32> {
    // ...
    thread::spawn(move || {
        let mut health = None;

        loop {
            // Try and read an updated modified health value from the CLI thread.
            match health_receiver.recv_timeout(Duration::from_millis(100)) {
                // The CLI thread has sent an updated health value.
                Ok(v) => health = Some(v),
                // The CLI thread has not sent an updated health value, so we will continue to
                // use the value previously stored.
                Err(RecvTimeoutError::Timeout) => (),

                // The CLI thread has disconnected, so we exit the thread.
                Err(RecvTimeoutError::Disconnected) => return,
            };

            // If the cheat is not enabled, continue the loop.
            if !is_enabled_sent.load(Ordering::Relaxed) {
                continue;
            }
            
            // TODO
        }
    });
    todo!()
}
```

Finally, if the cheat is enabled, we can write our modified health value.

```rust
unsafe fn new_health_task(alloc: SendAlloc) -> Task<i32, u32> {
    // ...
    thread::spawn(move || {
        let mut health = None;

        loop {
            // ...

            if let Some(health) = health {
                let alloc = Allocation::from(alloc);

                let p_health = match alloc
                    .deref_chain_with_base(pointers::LOCAL_PLAYER as _, pointers::OFFS_LOCAL_PLAYER_HEALTH)
                {
                    Ok(v) => v,
                    Err(e) => {
                        let _ = error_sender.send(e);
                        continue;
                    }
                };

                if let Err(err) = alloc.write_i32(p_health, health) {
                    let _ = error_sender.send(err);
                }
            }
        }
    });
    Task::new(health_sender, is_enabled, error_receiver)
}
```

## Finishing our main function

Now that we have the main cheat logic out of the way, we will revisit our main function and set up a command line
interface for our cheat user, along with getting the base address of `ac_client.exe`.

```rust
// main.rs

use anyhow::bail;

unsafe fn run() -> Result<()> {
    // ...
    let p_base = module::get_mod_base(pid, "ac_client.exe")
        .map_err(|e| anyhow!("Failed to create snapshot of process! OS error: {e}"))?;

    if p_base.is_null() {
        bail!("Failed to find ac_client.exe module!")
    }

    let alloc = SendAlloc::new(CVoidPtr(h_process), CVoidPtr(p_base));
    let tasks = Tasks::from_alloc(alloc);
    
    cli(tasks) // We will write this function next
}
```

Our CLI function should look something like this, but adjust it to your taste:

```rust
// main.rs

unsafe fn cli(tasks: Tasks) -> Result<()> {
    println!("Type help to get a list of commands");

    loop {
        let input = readln!("$ ");
        let trim = input.trim();

        if trim == "help" {
            println!("help\ntoggle_health");
            println!("health");
            println!(" \\ value: i32")
        }

        if trim == "toggle_health" {
            tasks.health_task().toggle_enabled();
        }

        if trim == "health" {
            println!("New health value:");

            let health = loop {
                let health_value = readln!("$ ");

                match health_value.parse() {
                    Ok(v) => break v,
                    Err(e) => eprintln!("{e}"),
                }
            };

            tasks.health_task().send_data(health)?;

            match tasks.health_task().read_error() {
                Ok(err) => {
                    eprintln!("Thread raised error {err}")
                }
                Err(TryRecvError::Empty) => (),
                Err(TryRecvError::Disconnected) => bail!("Thread disconnected! Aborting.")
            }
        }
    }
}
```

Wow. You did it! You wrote your first external cheat. Or, we hope. Let's test it and see if we get any errors or
unexpected results.

Loading up our program gives us no errors:

```
Type help to get a list of commands
$
```

If we enter `health` and our desired health value:

```
$ health
New health value:
$ 1000
```

And let us not forget to toggle our cheat (🤦🏿‍♂️)..

```
$ toggle_health
```

We can see our updated health value ingame!
