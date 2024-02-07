# filey-rs
A collection of utilities to make file operations more convenient.

# Install
Run the following Cargo command in your project directory:
```
cargo add filey
```
Or add the following line to your Cargo.toml:
```
filey = "1.3.0"
```

# Examples
Move a file to git repository and create symbolic link.
```
use filey::Filey;

let mut vimrc = Filey::new("~/.vimrc").expand_user()?;
let mut f = vimrc.move_to("dotfiles/")?;
f.symlink(&vimrc.path())?;
```
