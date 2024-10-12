# fetch-paper-api
It is a simple command tool, written in rust, designed to download jar file from papermc.io/api/v2 with different projects, versions and builds.

## Brief Guide
### Requirements
- a good network
- [cargo](https://www.rust-lang.org/tools/install) 

### Install
Simply type `cargo install fetch-paper` in your terminal.

### Example(TL;NR)  
> I want to download build 100 of papermc 1.18.1, using:  
`fetchpaper paper -v 1.18.1 -b 100`

> I want to download latest build of velocity 3.1.1, and save to ~/Downloads, using:  
`fetchpaper velocity -p ~/Downloads -v 3.1.1`

> I want to download latest version and latest build of velocity, using:  
`fetchpaper paper`

### Usage
```
fetchpaper [OPTIONS] <PROJECT>

Arguments:
  <PROJECT>  project_id

Options:
  -p, --path <VER>     path to download file, default will use "./target.jar" [default: ./target.jar]
  -v, --version <VER>  version id, default will use latest
  -b, --build <BUILD>  build id, default will use latest
      --skip-checksum  
  -h, --help           Print help
```

## Python version
Python version is end-of-support now!  
Rust provides powerful cargo tools to do all of those annoying versions and requirements things. Cheers!
