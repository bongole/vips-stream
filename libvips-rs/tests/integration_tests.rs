#[cfg(test)]
mod integration_tests {
    use std::{
        fs::{metadata, File},
        io::{Read, Write},
        os::unix::prelude::MetadataExt,
    };

    use tempfile::NamedTempFile;

    #[test]
    fn test_init() {
        let b = libvips_rs::init();
        assert_eq!(0, b);
    }

    #[test]
    fn test_concurrency() {
        libvips_rs::init();

        libvips_rs::set_concurrency(1);
        assert_eq!(1, libvips_rs::concurrency());

        libvips_rs::set_concurrency(0);
        assert!(0 != libvips_rs::concurrency());
    }

    #[test]
    fn test_set_simd() {
        libvips_rs::init();

        libvips_rs::set_simd_enabled(true);
        assert_eq!(true, libvips_rs::is_simd_enabled());

        libvips_rs::set_simd_enabled(false);
        assert_eq!(false, libvips_rs::is_simd_enabled());
    }

    #[test]
    fn test_version() {
        libvips_rs::init();
        let version = libvips_rs::version();
        assert!(!"".eq(&version));
    }

    #[test]
    fn test_source_custom_set_on_read() {
        libvips_rs::init();
        let mut src = libvips_rs::new_source_custom();

        let file_path = format!("{}/tests/assets/test.jpg", env!("CARGO_MANIFEST_DIR"));
        let mut file = File::open(file_path).unwrap();

        src.set_on_read(move |buf| file.read(buf).unwrap() as i64 );

        let _vi = libvips_rs::new_image_from_source(src);

        assert!(0 < _vi.vips_source.read_position());
    }

    #[test]
    fn test_target_custom() {
        libvips_rs::init();
        let mut src = libvips_rs::new_source_custom();
        let mut target = libvips_rs::new_target_custom();

        let file_path = format!("{}/tests/assets/test.jpg", env!("CARGO_MANIFEST_DIR"));
        let mut file = File::open(file_path).unwrap();

        src.set_on_read(move |buf| file.read(buf).unwrap() as i64 );

        let mut tmpfile = NamedTempFile::new().unwrap();
        let tmpfile_path = tmpfile.path().to_str().unwrap().to_string();
        target.set_on_write(move |buf| tmpfile.write(buf).unwrap() as i64 );

        let vi = libvips_rs::new_image_from_source(src);
        let r = vi.write_to_target(&target, ".png");

        let tmpfile_metadata = metadata(tmpfile_path).unwrap();

        assert!(r);
        assert!(target.is_finished());
        assert!(0 < tmpfile_metadata.size());
    }

    #[test]
    fn test_thumbnail() {
        libvips_rs::init();
        let mut src = libvips_rs::new_source_custom();
        let mut target = libvips_rs::new_target_custom();

        let file_path = format!("{}/tests/assets/test.jpg", env!("CARGO_MANIFEST_DIR"));
        let mut file = File::open(file_path).unwrap();

        src.set_on_read(move |buf| file.read(buf).unwrap() as i64 );

        let mut tmpfile = NamedTempFile::new().unwrap();
        let tmpfile_path = tmpfile.path().to_str().unwrap().to_string();
        target.set_on_write(move |buf| tmpfile.write(buf).unwrap() as i64 );

        let mut vi = libvips_rs::new_image_from_source(src);
        vi.thumbnail(300).thumbnail(200);
        let r = vi.write_to_target(&target, ".png");

        let tmpfile_metadata = metadata(tmpfile_path).unwrap();

        assert!(r);
        assert!(target.is_finished());
        assert!(0 < tmpfile_metadata.size());
    }
}
