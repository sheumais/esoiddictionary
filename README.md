## What is this?

This project was created to allow inspection of ability data that has been extracted from The Elder Scrolls Online game files.

> [!NOTE]
> This website downloads around 20mb of data. It should be cached by your browser but please be careful on mobile data plans.

## How do I look at the data?

You can just use website: https://sheumais.github.io/esoiddictionary/

## How did you get the data?

To extract the data for yourself follow these steps:
1. Download [ESOExtractData](https://en.uesp.net/wiki/ESO_Mod:EsoExtractData)
2. Run the following command, substituting in the appropriate file paths for your system/game.
```shell
EsoExtractData.exe "C:\Program Files (x86)\Steam\steamapps\common\Zenimax Online\The Elder Scrolls Online\depot\eso.mnf" .\export\ --extractsubfile combined --archive 0
```
3. In the `export\000\` folder, locate the largest `(Number)_Uncompressed.EsoFileData` file. (Example: `1257315_Uncompressed.EsoFileData`)

This file contains all the data, and is compressed and then copied into [/static/data.bin.zst](static/data.bin.zst) each patch.

All ability data remains the copyrighted intellectual propery of Zenimax Online Studios.

## I want to interact with the data using code

Due to the nature of the file (a database dump from ZOS) the format can and will change between patches slightly.

An implementation of a parser is updated by Dave from UESP when necessary and can be found on [GitHub](https://github.com/uesp/uesp-esoapps/blob/master/TestSkillFormat/TestSkillFormat/TestSkillFormat.cpp).

I got Claude to port the file format definition to a Rust struct which is available as an (unstable) [crate](https://github.com/sheumais/esoskilldataformat).

The format of the file is also documented on [UESP.net](https://en.uesp.net/wiki/ESO_Mod:Skill_Data_Format).

## How can I run it locally?

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

## How does this website work?

The game ability data, despite being ~100mb on disk, can be compressed down to <8mb.

This compressed form of the data is bundled with the web assembly file, allowing extremely quick navigation of the data.

I run a short [script](https://gist.github.com/sheumais/5281defb65a5ba8dc938ed84b160959a) to generate the byte indexes of each skill and put that into [/static/index.bin](static/index.bin).

These indexes are used to get the data from the decompressed file quickly. The indexes map into the original 100mb data, which exists in the browser's memory after being downloaded (hence the short load before the website opens despite the data being cached).

The website is written in Rust using the [Yew framework](https://yew.rs/).