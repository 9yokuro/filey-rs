# filey-rs
A collection of utilities to make file operations more convenient.

# Install
Run the following Cargo command in your project directory:
```
cargo add filey
```
Or add the following line to your Cargo.toml:
```
filey = "0.3.4"
```

# Examples
Print concatenated file(s)
```
use filey::{Filey, catenate};

let treasure_map = catenate!("map1", "map2", "map3", "map4");
println!("{}", treasure_map);
```

Move a file to git repository and create symbolic link.
```
use filey::Filey;

let vimrc = Filey::new("~/.vimrc").expand_user()?;
let f = vimrc.move_to("dotfiles/")?;
f.symlink(&vimrc.path())?;
```
