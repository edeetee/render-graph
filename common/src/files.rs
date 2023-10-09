use std::{
    fs::read_dir,
    path::{PathBuf},
};





pub fn map_file_tree<O, DirFn: FnMut(PathBuf, Vec<O>) -> O, FileFn: FnMut(PathBuf) -> O>(
    path: PathBuf,
    dir_fn: &mut DirFn,
    file_fn: &mut FileFn,
) -> O {
    if path.is_dir() {
        let branch_inner = read_dir(&path)
            .unwrap()
            .into_iter()
            .map(|dir| map_file_tree(dir.unwrap().path(), dir_fn, file_fn))
            .collect();

        dir_fn(path, branch_inner)
    } else {
        // let info = LeafItem::try_from_path(path)?;
        file_fn(path)
    }
}
