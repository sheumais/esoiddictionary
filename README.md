## What is this?

This repository (and associated website) is designed to allow easy inspection of ability data that has been extracted from The Elder Scrolls Online files.

## How do I look at the data?

You can just use website: https://sheumais.github.io/esoiddictionary/

## How can I run it locally for faster queries?

Download and install [Rust](https://rustup.rs/)

Install [trunk](https://trunk-rs.github.io/trunk/)
```sh
cargo install --locked trunk
```

Add the WebAssembly target
```sh
rustup target add wasm32-unknown-unknown
```

Serve locally
```sh
trunk serve --public-url /esoiddictionary/
```

## How did you get the data?

To extract the data, for yourself, as it appears at [/static/data.bin](static/data.bin) follow these steps:
1. Download [ESOExtractData](https://en.uesp.net/wiki/ESO_Mod:EsoExtractData)
2. Run the following command, substituting in the appropriate file paths for your system/game.
```shell
EsoExtractData.exe "C:\Program Files (x86)\Steam\steamapps\common\Zenimax Online\The Elder Scrolls Online\depot\eso.mnf" .\export\ --extractsubfile combined --archive 0
```
3. In the `export\000\` folder, locate the largest `(Number)_Uncompressed.EsoFileData` file. (Example: `1257315_Uncompressed.EsoFileData`)

This file contains all the data, and is copied directly into the /static/data.bin file each patch

## I want to interact with the data using code

Due to the nature of the file (a database dump from ZOS) the format can and will change between patches slightly.

An implementation of a parser is updated by Dave from UESP when necessary and can be found on GitHub [here](https://en.uesp.net/wiki/ESO_Mod:Skill_Data_Format).

I got Claude to port the file format definition to a Rust struct which is available as an (unstable) [crate](https://github.com/sheumais/esoskilldataformat).

The format of the file is also documented [here](https://en.uesp.net/wiki/ESO_Mod:Skill_Data_Format).

## How does this website work?

The data is copied directly from the game into [/static/data.bin](static/data.bin). I do not touch this file in any way.

I run a short [script](https://gist.github.com/sheumais/5281defb65a5ba8dc938ed84b160959a) to generate the byte indexes of each skill and put that into [/static/index.bin](static/index.bin).

These indexes are used to request specific abilities via a byte range header in the [http request](src/fetch.rs). (This is necessary to avoid having every visitor download a 100mb file of ability data)

Once the client receives the data, it parses it into the format specified by the external [crate](https://github.com/sheumais/esoskilldataformat), and [displays](src/id.rs) the data.

The website is written in Rust using the [Yew framework](https://yew.rs/).