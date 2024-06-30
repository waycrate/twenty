# Twenty

20-20-20 rule enforcer for compositors implementing ext-session-lock.
Twenty makes sure that you look 20 ft away every 20 minutes for 20 seconds to relax your eyes.

Uses [waycrate/exwlshelleventloop/iced_sessionlock](https://github.com/waycrate/exwlshelleventloop/tree/master/iced_sessionlock)
for locking the screen.

### Installing

You can compile it using `cargo` or via [baker](https://github.com/rv178/baker).

via baker

```
bake setup
bake
sudo bake install
```

A binary will be copied to `./bin/twenty`

via cargo 

```
cargo build --release
```

A binary will be copied to `./target/release/`

### Usage

[] indicates optional arguments.

#### Initializing the program.

```
twenty --init [light/dark]
```
The lock screen defaults to dark mode unless specified otherwise.

#### Killing the program

```
twenty --kill
```

### Uninstalling

```
sudo bake uninstall
```

#### Authored by [rv178](https://github.com/rv178) and [shivkr6](https://github.com/shivkr6)
