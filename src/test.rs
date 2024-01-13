#[cfg(test)]
mod tests {
    use crate::{create, file_types::FileTypes, filey::Filey};
    use std::{
        fs::{create_dir_all, remove_dir_all, File},
        os::unix::fs::symlink,
        path::Path,
    };

    fn init() {
        let test_dir = "test_dir";
        if !Path::new(test_dir).exists() {
            create_dir_all(test_dir).unwrap();
        }
    }

    fn quit() {
        let test_dir = "test_dir";
        if Path::new(test_dir).exists() {
            remove_dir_all(test_dir).unwrap();
        }
    }

    #[test]
    fn test_file_name() {
        assert_eq!(
            Filey::new("src/lib.rs").file_name(),
            Some("lib.rs".to_string())
        )
    }

    #[test]
    fn test_file_stem() {
        assert_eq!(
            Filey::new("src/lib.rs").file_stem(),
            Some("lib".to_string())
        )
    }

    #[test]
    fn test_parent_dir() {
        assert_eq!(
            Filey::new("src/lib.rs").parent_dir(),
            Some("src".to_string())
        )
    }

    #[test]
    fn test_create_files() {
        init();
        let file_a = Path::new("test_dir/file_a");
        Filey::new(&file_a).create(FileTypes::File).unwrap();
        assert!(file_a.exists() && file_a.is_file());
        let file_b = Path::new("test_dir/file_b");
        let file_c = Path::new("test_dir/file_c");
        create!(FileTypes::File, &file_b, &file_c);
        assert!(file_b.exists() && file_b.is_file());
        assert!(file_c.exists() && file_c.is_file());
        quit();
    }

    #[test]
    fn test_create_directories() {
        init();
        let dir_a = Path::new("test_dir/dir_a");
        Filey::new(&dir_a).create(FileTypes::Directory).unwrap();
        assert!(dir_a.exists() && dir_a.is_dir());
        let dir_b = Path::new("test_dir/dir_b");
        let dir_c = Path::new("test_dir/dir_c");
        create!(FileTypes::Directory, &dir_b, &dir_c);
        assert!(dir_b.exists() && dir_b.is_dir());
        assert!(dir_c.exists() && dir_c.is_dir());
        quit();
    }

    #[test]
    fn test_create_symlink() {
        init();
        let file_a = "test_dir/file_a";
        File::create(file_a).unwrap();
        let file_a_symlink = Path::new("test_dir/file_a_symlink");
        Filey::new(file_a).symlink(&file_a_symlink).unwrap();
        assert!(file_a_symlink.is_symlink());
        quit();
    }

    #[test]
    fn test_create_hard_link() {
        init();
        let file_a = "test_dir/file_a";
        File::create(file_a).unwrap();
        let file_a_hard_link = Path::new("test_dir/file_a_hard_link");
        Filey::new(file_a).hard_link(&file_a_hard_link).unwrap();
        assert!(file_a_hard_link.exists());
        quit();
    }

    #[test]
    fn test_file_types() {
        init();
        let file_a = "test_dir/file_a";
        File::create(file_a).unwrap();
        let file_a_symlink = "test_dir/file_a_symlink";
        symlink(file_a, file_a_symlink).unwrap();
        let dir_a = "test_dir/dir_a";
        create_dir_all(dir_a).unwrap();
        assert_eq!(FileTypes::which(file_a), Some(FileTypes::File));
        assert_eq!(FileTypes::which(dir_a), Some(FileTypes::Directory));
        assert_eq!(FileTypes::which(file_a_symlink), Some(FileTypes::Symlink));
        assert_eq!(FileTypes::which("test_dir/no_such_file_or_directory"), None);
        quit();
    }

    #[test]
    fn test_absolutized() {
        assert_eq!(
            Filey::new("test_dir/file_a")
                .absolutized()
                .unwrap()
                .to_string(),
            "/home/p14/code/filey/test_dir/file_a".to_string()
        );
    }

    #[test]
    fn test_close_user() {
        assert_eq!(
            Filey::new("test_dir/file_a")
                .absolutized()
                .unwrap()
                .close_user()
                .unwrap()
                .to_string(),
            "~/code/filey/test_dir/file_a"
        );
    }

    #[test]
    fn test_expand_user() {
        assert_eq!(
            Filey::new("test_dir/file_a")
                .absolutized()
                .unwrap()
                .close_user()
                .unwrap()
                .expand_user()
                .unwrap()
                .to_string(),
            "/home/p14/code/filey/test_dir/file_a"
        );
    }

    #[test]
    fn test_copy() {
        init();
        let file_a = "test_dir/file_a";
        File::create(file_a).unwrap();
        let copied_file_a = Path::new("test_dir/copied_file_a");
        Filey::new(file_a).copy(&copied_file_a).unwrap();
        assert!(copied_file_a.exists());
        quit();
    }

    #[test]
    fn test_remove() {
        init();
        let files = ["test_dir/file_a", "test_dir/file_b", "test_dir/file_c"];
        let dirs = ["test_dir/dir_a", "test_dir/dir_b", "test_dir/dir_c"];
        for i in &files {
            File::create(i).unwrap();
        }
        for i in &dirs {
            create_dir_all(i).unwrap();
        }
        for i in &files {
            let path = Path::new(i);
            Filey::new(&path).remove().unwrap();
            assert!(!path.exists());
        }
        for i in &dirs {
            let path = Path::new(i);
            Filey::new(&path).remove().unwrap();
            assert!(!path.exists());
        }
        quit();
    }

    #[test]
    fn test_move() {
        init();
        let mut file_a = Filey::new("test_dir/file_a");
        file_a.create(FileTypes::File).unwrap();
        let renamed_file_a = Path::new("test_dir/renamed_file_a");
        file_a.move_to(&renamed_file_a).unwrap();
        assert!(renamed_file_a.exists());
        let file_a_in_dir_a = Path::new("test_dir/dir_a/renamed_file_a");
        create_dir_all("test_dir/dir_a").unwrap();
        file_a.move_to(&file_a_in_dir_a).unwrap();
        assert!(file_a_in_dir_a.exists());
        quit();
    }
}
