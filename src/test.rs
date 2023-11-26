#[cfg(test)]
mod tests {
    use crate::{
        catenate, create, remove, file_operations::Filey, file_types::FileTypes, unit_of_information::UnitOfInfo,
    };
    use std::io::{Read, Write};
    use std::fs::File;
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
            f.absolutized().unwrap().to_string().as_str(),
            "/home/p14/code/filey/test/test.txt"
        );
        let f2 = Filey::new("~/code/filey/test/test.txt");
        assert_eq!(
            f.absolutized().unwrap().to_string().as_str(),
            "/home/p14/code/filey/test/test.txt"
        );
        assert_eq!(
            f2.expand_user().unwrap().to_string().as_str(),
            "/home/p14/code/filey/test/test.txt"
        );
        let f3 = Filey::new("/home/p14/code/filey/test/test.txt");
        assert_eq!(
            f3.close_user().unwrap().as_str(),
            "~/code/filey/test/test.txt"
        );
        let f4 = Filey::new("test/test_symlink.txt");
        assert_eq!(f4.exists(), true);
        assert_eq!(f4.canonicalized().unwrap().path(), f3.path());
        f3.move_to("test/a").unwrap();
        let f5 = Filey::new("test/a/test.txt");
        assert_eq!(f5.exists(), true);
        f5.move_to(&f.path()).unwrap();
        f.remove().unwrap();
        assert_eq!(f.exists(), false);
        File::create(&f.path()).unwrap();
        assert_eq!(
            UnitOfInfo::convert(1073741824, UnitOfInfo::MiB) as u64,
            1024
        );
        assert_eq!(UnitOfInfo::format(1024).as_str(), "1KiB");
        assert_eq!(UnitOfInfo::format(1048576).as_str(), "1MiB");
        assert_eq!(UnitOfInfo::format(1073741824).as_str(), "1GiB");
        assert_eq!(UnitOfInfo::format(1099511627776).as_str(), "1TiB");
        assert_eq!(UnitOfInfo::format(1125899906842624).as_str(), "1PiB");
        assert_eq!(UnitOfInfo::format(1152921504606846976).as_str(), "1EiB");
        let s = catenate!(
            "src/unit_of_information.rs",
            "src/file_operations.rs"
        );
        create!(FileTypes::File, "a.txt", "b.txt", "c.txt");
        create!(FileTypes::Directory, "d", "e", "f");
        remove!("a.txt", "b.txt", "c.txt", "d", "e", "f");
        println!("{}", s);
        let mut file = Filey::new("kiss.txt");
        file.write(b"Keep it simple, stupid.").unwrap();
        let mut buffer = String::new();
        let mut reader = File::open("kiss.txt").unwrap();
        reader.read_to_string(&mut buffer).unwrap();
        println!("{}", buffer);
        file.remove().unwrap();
    }
}
