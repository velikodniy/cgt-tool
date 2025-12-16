## 1. CLI Implementation

- [x] 1.1 Update `Commands::Parse` to accept `Vec<PathBuf>` instead of `Option<PathBuf>` for file argument
- [x] 1.2 Update `Commands::Report` to accept `Vec<PathBuf>` instead of `PathBuf` for file argument
- [x] 1.3 Implement file concatenation helper function in `main.rs`
- [x] 1.4 Update `parse` command handler to concatenate multiple files before parsing
- [x] 1.5 Update `report` command handler to concatenate multiple files before parsing
- [x] 1.6 Update PDF output default filename logic (`report.pdf` for multiple inputs)
- [x] 1.7 Add check to refuse overwriting existing files for default PDF output

## 2. Testing

- [x] 2.1 Add CLI integration test for `parse` with multiple files
- [x] 2.2 Add CLI integration test for `report` with multiple files
- [x] 2.3 Add CLI integration test for PDF output filename with multiple inputs
- [x] 2.4 Add CLI integration test for PDF overwrite protection

## 3. Documentation

- [x] 3.1 Update README.md with multiple file usage examples for `parse`
- [x] 3.2 Update README.md with multiple file usage examples for `report`
