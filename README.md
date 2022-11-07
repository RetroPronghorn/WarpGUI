<h1 align="center">
  <a href="https://satellite.im" target="_blank">
  <img src="extra/images/logo.png" width=200 height=200/><br>
  Uplink
  </a>
</h1>

<h4 align="center">Privacy First, Modular, P2P messaging client built atop Warp.</h4>

<br/>

Uplink is written in pure Rust with a UI in [Dioxus](https://github.com/DioxusLabs) (which is also written in Rust). It was developed to be a new foundation for the basic implementation of Warp features in a universal application.

The goal should be to build a hyper-customizable application that can run anywhere and support extensions.

## Features

// TODO

## Contributing

Guidelines for contributing are located in the [`CONTRIBUTING.md`](CONTRIBUTING.md).

## Prerequisites

You'll need to install the required dependancies for your system in order to build and develop on Uplink. See the table below for help installing them on your system.

| Distribution    | Commands                                                                             |
| --------------- | ------------------------------------------------------------------------------------ | ----------- |
| Debian & Ubuntu | apt get -y libgtk-3-dev libwebkit2gtk-4.0-dev libappindicator3-dev protobuf-compiler |
| Arch            | pacman -S gtk3 cmake protobuf                                                        |
| MacOS           | brew install protobuf, xcode-select --install, curl https://sh.rustup.rs -sSf        | sh -s -- -y |
| Windows         | ¯\\\_(ツ)\_/¯                                                                        |

## Building from source

### Linux

// TODO

### Windows

¯\\\_(ツ)\_/¯

### Mac

// TODO

## Running

To run the app in dev mode simply run `cargo run`.

The executable has a few command line options that might come in handy:

- `--path` which changes the folder used for storage, currently it's .warp but later it will go somewhere else
- `--title` which comes in handy if you have more than one window open

### Extra Options

You can specify a window title and a custom path for storage which is useful for local AB testing.
`cargo run -- --title "User 1" --path .user1`

> Note: You can also pass these options directly to the binary by omitting the `--`.

## Local Testing

To spawn multiple instances of the app execute the test script `test/physical/launch_physical_test.sh`. This will spawn two named windows with local cache files in a `.temp` directory.

Please also read through the [Physical Testing Checklist](https://github.com/Satellite-im/Uplink/blob/dev/test/CHECKLIST.md) and ensure functionality before submitting a PR.

## Resetting Data

It's often necessary to reset your account for development, to do so just delete any `.temp` and `.warp` files within this project. Note that on most \*nix based systems the .warp file is stored in your home directory unless otherwise configured.

## Contributions

All contributions are welcome! Please keep in mind we're still a relatively small team and any work done to make sure contributions don't cause bugs or issues in the application is much appreciated.
