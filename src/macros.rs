/// Concatenates file(s) to String.
///
/// # Examples
/// ```
/// # use filey::{Filey, catenate};
/// # use std::error::Error;
/// # use std::fs;
/// #
/// # fn cat() -> Result<(), Box<Error>> {
/// fs::write("h.rs", "fn main {")?;
/// fs::write("el.rs", r#"    println!("Hello, World!");"#)?;
/// fs::write("lo.rs", "}")?;
///
/// let s = catenate!("h.rs", "el.rs", "lo.rs");
/// println!("{}", s);
/// // fn main() {
/// //     println!("Hello, World!");
/// // }
/// # Ok(())
/// # }
/// # fn main() {
/// # cat().unwrap();
/// # }
/// ```
#[macro_export]
macro_rules! catenate {
    ( $( $path:expr ),* ) => {
        {
            use std::{io::Read, path::Path, fs::File};

            let mut buffer = String::new();
            $(
                if Path::new($path).is_file() {
                let mut s = String::new();
                let mut f = File::open($path).unwrap();
                f.read_to_string(&mut s).unwrap();
                buffer.push_str(&s);
                buffer.push('\n');
                }
            )*
            buffer
        }
    }
}

/// Creates file(s) or directory(s).
///
/// # Examples
/// ```
/// # use filey::{Filey, FileTypes, create};
/// #
/// create!(FileTypes::File, "src/draw_ui.rs", "src/app_state.rs", "run.rs");
/// ```
#[macro_export]
macro_rules! create {
    ( $file_type:expr $(, $path:expr )* $(,)?) => {
        {
            $(
                let f = Filey::new($path);
                if !f.exists() {
                    f.create($file_type).unwrap();
                }
            )*
        }
    }
}

/// Removes file(s) or directory(s).
///
/// # Examples
/// ```
/// # use filey::{Filey, remove};
/// remove!("old_dir", "unnecessary.jpg");
/// ```
#[macro_export]
macro_rules! remove {
    ( $( $path:expr ), *) => {
        {
            $(
                let f = Filey::new($path);
                if f.exists() {
                    f.remove().unwrap();
                }
            )*
        }
    }
}
