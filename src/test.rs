#[cfg(test)]
mod tests {
    use super::super::*;
    use std::fs;
    #[test]
    fn it_works() {
        assert_eq!(FileTypes::which("test"), FileTypes::Directory);
        assert_eq!(FileTypes::which("test/test.txt"), FileTypes::File);
        assert_eq!(
            FileTypes::which("test/test_symlink.txt"),
            FileTypes::Symlink
        );

        let fileinfo = FileInfo::new("test/test.txt").unwrap();
        fileinfo
            .write("test/test.txt", fileinfo.to_string())
            .unwrap();
        assert_eq!(fileinfo, fileinfo.read("test/test.txt").unwrap());

        let f = FileOperations::new("test/test.txt");
        assert_eq!(
            "test/test.txt",
            f.pathbuf().to_string_lossy().to_string().as_str()
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
            "/home/p14/code/file_operations_rs/test/test.txt"
        );
        let f2 = FileOperations::new("~/code/file_operations_rs/test/test.txt");
        assert_eq!(
            f.absolutized()
                .unwrap()
                .to_string_lossy()
                .to_string()
                .as_str(),
            "/home/p14/code/file_operations_rs/test/test.txt"
        );
        assert_eq!(
            f2.expand_user()
                .unwrap()
                .to_string_lossy()
                .to_string()
                .as_str(),
            "/home/p14/code/file_operations_rs/test/test.txt"
        );
        let f3 = FileOperations::new("/home/p14/code/file_operations_rs/test/test.txt");
        assert_eq!(
            f3.close_user().unwrap().as_str(),
            "~/code/file_operations_rs/test/test.txt"
        );
        let f4 = FileOperations::new("test/test_symlink.txt");
        assert_eq!(f4.exists(), true);
        assert_eq!(f4.canonicalized().unwrap(), f3.pathbuf());
        f3.move_to("test/a").unwrap();
        let f5 = FileOperations::new("test/a/test.txt");
        assert_eq!(f5.exists(), true);
        f5.move_to(f.pathbuf()).unwrap();
        f.remove().unwrap();
        assert_eq!(f.exists(), false);
        fs::File::create(f.pathbuf()).unwrap();
        f.write(f.pathbuf(), "file_operations_rs").unwrap();
        assert_eq!(f.read(f.pathbuf()).unwrap().as_str(), "file_operations_rs");
    }
}

