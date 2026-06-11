use std::io;
use std::path::Path;

use zip::ZipArchive;

pub fn unzip_file(zip_path: &Path, dest: &Path) -> io::Result<()> {
    let file = std::fs::File::open(zip_path)?;
    let mut archive = ZipArchive::new(file).map_err(zip_to_io)?;
    std::fs::create_dir_all(dest)?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(zip_to_io)?;
        let rel = match entry.enclosed_name() {
            Some(p) => p,
            None => continue,
        };
        let out_path = dest.join(rel);
        if entry.is_dir() {
            std::fs::create_dir_all(&out_path)?;
        } else {
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let mut out = std::fs::File::create(&out_path)?;
            io::copy(&mut entry, &mut out)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Some(mode) = entry.unix_mode() {
                    std::fs::set_permissions(&out_path, std::fs::Permissions::from_mode(mode))?;
                }
            }
        }
    }
    Ok(())
}

pub fn copy_dir(src: &Path, dst: &Path) -> io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir(&from, &to)?;
        } else {
            if let Some(parent) = to.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

#[cfg(unix)]
pub fn rewrite_shortcut(shortcut: &Path, target: &Path) -> io::Result<()> {
    if shortcut.symlink_metadata().is_ok() {
        std::fs::remove_file(shortcut)?;
    }
    if let Some(parent) = shortcut.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::os::unix::fs::symlink(target, shortcut)
}

#[cfg(windows)]
pub fn rewrite_shortcut(shortcut: &Path, target: &Path) -> io::Result<()> {
    if let Some(parent) = shortcut.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let lnk_path = if shortcut.extension().and_then(|e| e.to_str()) == Some("lnk") {
        shortcut.to_path_buf()
    } else {
        shortcut.with_extension("lnk")
    };
    if lnk_path.exists() {
        let _ = std::fs::remove_file(&lnk_path);
    }
    let link = mslnk::ShellLink::new(target)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("shortcut: {e}")))?;
    link.create_lnk(&lnk_path)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("shortcut: {e}")))
}

fn zip_to_io(e: zip::result::ZipError) -> io::Error {
    io::Error::new(io::ErrorKind::Other, format!("zip: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::now_ms;
    use std::io::Write;
    use zip::write::SimpleFileOptions;
    use zip::CompressionMethod;

    fn make_zip(path: &Path) {
        let file = std::fs::File::create(path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        let opts = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
        zip.add_directory("bin/", opts).unwrap();
        zip.start_file("bin/app", opts).unwrap();
        zip.write_all(b"hello-binary").unwrap();
        zip.start_file("readme.txt", opts).unwrap();
        zip.write_all(b"notes").unwrap();
        zip.finish().unwrap();
    }

    #[test]
    fn unzip_then_copy_preserves_previous_version() {
        let root = std::env::temp_dir().join(format!("dn-install-{}", now_ms()));
        let zip_path = root.join("artifact.zip");
        let staging = root.join("Updates").join("v1.1.0");
        let install_old = root.join("Install").join("v1.0.0");
        let install_new = root.join("Install").join("v1.1.0");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::create_dir_all(&install_old).unwrap();
        std::fs::write(install_old.join("marker"), b"old").unwrap();

        make_zip(&zip_path);
        unzip_file(&zip_path, &staging).unwrap();
        copy_dir(&staging, &install_new).unwrap();

        assert_eq!(
            std::fs::read_to_string(install_new.join("bin/app")).unwrap(),
            "hello-binary"
        );
        assert_eq!(
            std::fs::read_to_string(install_new.join("readme.txt")).unwrap(),
            "notes"
        );
        assert!(install_old.join("marker").exists());

        std::fs::remove_dir_all(&root).ok();
    }

    #[cfg(unix)]
    #[test]
    fn shortcut_points_to_target() {
        let root = std::env::temp_dir().join(format!("dn-shortcut-{}", now_ms()));
        let target_a = root.join("v1.0.0").join("bin");
        let target_b = root.join("v1.1.0").join("bin");
        std::fs::create_dir_all(&target_a).unwrap();
        std::fs::create_dir_all(&target_b).unwrap();
        let shortcut = root.join("shortcutLauncher");

        rewrite_shortcut(&shortcut, &target_a).unwrap();
        assert_eq!(std::fs::read_link(&shortcut).unwrap(), target_a);

        rewrite_shortcut(&shortcut, &target_b).unwrap();
        assert_eq!(std::fs::read_link(&shortcut).unwrap(), target_b);

        std::fs::remove_dir_all(&root).ok();
    }
}
