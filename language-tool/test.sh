git clone https://github.com/tree-sitter/tree-sitter-rust tree-sitter-rust

mkdir languages
tree-sitter build tree-sitter-rust -o languages/rust.dll

ls languages

cargo run