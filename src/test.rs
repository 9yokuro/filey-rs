#[cfg(test)]
mod tests {
    use crate::{
        file_types::FileTypes,
        file_operations::Filey,
    };
    use std::fs;
    #[test]
    fn it_works() {
        assert_eq!(FileTypes::which("test").unwrap(), FileTypes::Directory);
        assert_eq!(FileTypes::which("test/test.txt").unwrap(), FileTypes::File);
        assert_eq!(
            FileTypes::which("test/test_symlink.txt").unwrap(),
            FileTypes::Symlink
        );

        let f = Filey::new("test/test.txt");
        assert_eq!(
            "test/test.txt",
            f.path().to_string_lossy().to_string().as_str()
        );
        assert_eq!(f.file_name().unwrap().as_str(), "test.txt");
        assert_eq!(f.file_stem().unwrap().as_str(), "test");
        assert_eq!(
            f.parent_dir()
                .unwrap()
                .to_string_lossy()
                .to_string()
                .as_str(),
            "test"
        );
        assert_eq!(
            f.absolutized()
                .unwrap()
                .to_string_lossy()
                .to_string()
                .as_str(),
            "/home/p14/code/filey/test/test.txt"
        );
        let f2 = Filey::new("~/code/filey/test/test.txt");
        assert_eq!(
            f.absolutized()
                .unwrap()
                .to_string_lossy()
                .to_string()
                .as_str(),
            "/home/p14/code/filey/test/test.txt"
        );
        assert_eq!(
            f2.expand_user()
                .unwrap()
                .to_string_lossy()
                .to_string()
                .as_str(),
            "/home/p14/code/filey/test/test.txt"
        );
        let f3 = Filey::new("/home/p14/code/filey/test/test.txt");
        assert_eq!(
            f3.close_user().unwrap().as_str(),
            "~/code/filey/test/test.txt"
        );
        let f4 = Filey::new("test/test_symlink.txt");
        assert_eq!(f4.exists(), true);
        assert_eq!(f4.canonicalized().unwrap(), f3.path());
        f3.move_to("test/a").unwrap();
        let f5 = Filey::new("test/a/test.txt");
        assert_eq!(f5.exists(), true);
        f5.move_to(&f.path()).unwrap();
        f.remove().unwrap();
        assert_eq!(f.exists(), false);
        fs::File::create(&f.path()).unwrap();
    }
}
