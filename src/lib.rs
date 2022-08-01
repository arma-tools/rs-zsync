mod util;

pub mod file_maker;
pub mod meta_file;

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::file_maker::FileMaker;
    use crate::meta_file::MetaFile;

    #[test]
    fn test() {
        let file = String::from("test-data/grad_rebreatherOnLand.pbo");

        println!("File: {}", file);

        let mut file_zsync = file.clone();
        file_zsync.push_str(".zsync");

        let mut mf = MetaFile::new();
        assert!(mf.parse_zsync(Path::new(&file_zsync)).is_ok());

        let mut filemaker = FileMaker::new(&mf);
        let progress = filemaker.map_matcher(Path::new(&file));
        println!("Caluclated File Completion: {}%", progress);

        assert_eq!(progress as u32, 98);
        assert_eq!(filemaker.remaining_size(progress) as u32, 16274);

        let parts = filemaker.file_maker();
        let first_part = &parts[0];
        assert_eq!(first_part.start_offset, 0);
        assert_eq!(first_part.end_offset, 8192);
        assert_eq!(first_part.block_length, 8192);
        assert_eq!(first_part.offset, 0);

        let second_part = &parts[1];
        assert_eq!(second_part.start_offset, 1155072);
        assert_eq!(second_part.end_offset, 1163264);
        assert_eq!(second_part.block_length, 422);
        assert_eq!(second_part.offset, 0);
    }
}
