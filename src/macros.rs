/// Creates file(s).
///
/// # Examples
/// ```
/// # use filey::{Filey, create_file};
/// #
/// create_file!("src/draw_ui.rs", "src/app_state.rs", "run.rs");
/// ```
#[macro_export]
macro_rules! create_file {
    ( $( $path:expr ), *) => {
        {
            $(
                let f = Filey::new($path);
                if !f.exists() {
                    f.create_file().unwrap();
                }
            )*
        }
    }
}

/// Creates directory(s).
#[macro_export]
macro_rules! create_dir {
    ( $( $path:expr ), *) => {
        {
            $(
                let f = Filey::new($path);
                if !f.exists() {
                    f.create_dir().unwrap();
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
