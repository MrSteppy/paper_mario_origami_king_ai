# Paper Mario: The Origami King AI

Paper Mario: The Origami King has fights which take place in a round arena consisting of four rings which can be turned 
and shifted. An integral part of a fight is to align the enemies on those rings in a favorable order. 

Which can be hard. This project aims to always find an optimal order in the least amount of turns.

## Modules
The project consists of the following modules:

### game_logic

The code which simulates the game and the solving algorithm.
Works as standalone CLI app.

#### Command overview
A command overview can be found typing "?" or "help".

| example         | description                                                                                          |
|-----------------|------------------------------------------------------------------------------------------------------|
| c2 124          | Set enemies in column 2 on row 1, 2 and 4                                                            |
| c3 3 H          | Set the enemy in column 3, row 3 to require to be killed by hammer                                   |
| c6 1234 J       | Set the enemies in column 6 to be killed by jumping                                                  |
| - c3 1          | Remove the enemy on column 3, row 1                                                                  |
| g 4             | Tell the ai that there are 4 groups of enemies. This can usually be omitted.                         |
| solve in 3      | Find the optimal solution in max 3 turns                                                             |
| solve           | Find the optimal solution in as few turns as possible. Can be slower than `solve in 3`               |
| solve fast      | Solve the arena in as few turns as possible and be happy with any solution, even if it isn't optimal |
| solve fast in 5 | Find a possibly non-optimal solution in max 5 turns                                                  |
| +hammer         | Tell the ai you have a throwable hammer which can be used                                            |
| -hammer         | Tell the ai you don't have a throwable hammer to use                                                 |
| e r2 5          | Manually execute the move `r2 5`                                                                     |
| clear           | Reset the arena                                                                                      |

#### Running the CLI
```commandline
cargo run --release --package game_logic
```

### gui

A graphical frontend. Produces a desktop executable binary and a library which can be used by the android app. 

#### Running the desktop app
```bash
cargo run --release --package gui
```

### android_app

A separate crate just used to build the android app. This is necessary since the tool that builds the app can't work 
with a binary crate (main.rs in gui).

Please see [Building for Android](#building-for-android) for details.

## Terminology

### Rings (Rows)

The arena consists of four rings (rows) numbered `r1` to `r4`, where smaller numbers denote the inner rings.

### Columns

There are 12 columns, starting at `c1` up to `c12`. The right column at the top is `c1`, from there on the number
increases clockwise.

### Moves

A `Move` turns a ring or shifts a column. 
It consists of a target (row or column to turn/shift) and an amount.
The move `r1 3` rotates the innermost ring by 3 clockwise.
The move `c4 2` shifts the fourth column away from the center by 2.

Negative amounts turn counterclockwise or shift towards the middle.

## Building for android

This section describes how to build an apk from the [android_app](#android_app) module. 

### Setup

Install [cargo apk](https://github.com/rust-mobile/cargo-apk):
```bash
cargo install cargo-apk
```

Install rustup targets needed to compile for android:
```bash
rustup target add armv7-linux-androideabi
rustup target add aarch64-linux-android
rustup target add i686-linux-android
rustup target add x86_64-linux-android
```

Install the Android SDK and the NDK. The recommended way to do this, is to install Android Studio, and let it download the SDK for you.
After that go to `Settings > Languages & Frameworks > Android SDK` and switch to the `SDK Tools` tab. There select `NDK (Side by side)`.
Click on `Apply` and it should also download that. 

Set the `ANDROID_HOME` environment variable pointing to `$HOME/Android/Sdk` (or wherever you installed the SDK)
and the `ANDROID_NDK_HOME` variable pointing to `$ANDROID_HOME/ndk/<your ndk version>`.

Last but not least, for release builds you need a `keystore.jks` file in the `android_app` module.
You can generate one with Android Studio:

Go to `Build > Generate signed App Bundle/APK`, select `APK` and press `next`.
Now click on `Create new` and fill in the needed information. The password has to be `steptech`.

### Building the apk

If you've done everything right, the following script should produce an .apk file under
`target/release/apk`:

```bash
cd android_app
cargo apk build --release
```

For a quicker debug build omit the --release flag
```bash
cd android_app
cargo apk build
```